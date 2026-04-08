use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_sound")]
    pub notification_sound: String,
    #[serde(default)]
    pub is_muted: bool,
    #[serde(default = "default_font_size")]
    pub font_size: String,
    #[serde(default)]
    pub is_usage_enabled: bool,
}

fn default_sound() -> String {
    "Purr".to_string()
}

fn default_font_size() -> String {
    "regular".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            notification_sound: default_sound(),
            is_muted: false,
            font_size: default_font_size(),
            is_usage_enabled: false,
        }
    }
}

impl AppSettings {
    fn file_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".english-learning")
            .join("settings.json")
    }

    pub fn load() -> Self {
        let path = Self::file_path();
        if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::file_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let data = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, data).map_err(|e| e.to_string())
    }
}
