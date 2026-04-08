use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::models::parrot_level::ParrotLevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelData {
    pub current_level: ParrotLevel,
    pub xp: u32,
    pub last_active_date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelInfo {
    pub level: ParrotLevel,
    pub level_name: String,
    pub emoji: String,
    pub xp: u32,
    pub xp_threshold: u32,
    pub next_level_xp: Option<u32>,
    pub xp_progress: f64,
}

impl Default for LevelData {
    fn default() -> Self {
        Self {
            current_level: ParrotLevel::Egg,
            xp: 0,
            last_active_date: None,
        }
    }
}

fn level_file() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".english-learning")
        .join("level.json")
}

pub fn load() -> LevelData {
    let path = level_file();
    match std::fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => LevelData::default(),
    }
}

pub fn save(data: &LevelData) {
    let path = level_file();
    let dir = path.parent().unwrap();
    let _ = std::fs::create_dir_all(dir);
    if let Ok(json) = serde_json::to_string_pretty(data) {
        let _ = std::fs::write(&path, json);
    }
}

pub fn add_xp(tip_type: &str, accuracy: u32) -> LevelData {
    let mut data = load();

    // Apply inactivity decay
    if let Some(ref last_date) = data.last_active_date {
        if let Ok(last) = chrono::NaiveDate::parse_from_str(last_date, "%Y-%m-%d") {
            let today = chrono::Utc::now().date_naive();
            let days_inactive = (today - last).num_days();
            if days_inactive > 1 {
                let decay = 0.95_f64.powi(days_inactive as i32 - 1);
                data.xp = (data.xp as f64 * decay).round() as u32;
                log::info!("XP decay applied: {} days inactive, factor {:.3}", days_inactive, decay);
            }
        }
    }

    // Add XP based on tip type
    let xp_gain = match tip_type {
        "good" => 10,
        "correction" => 5,
        _ => 0,
    };
    data.xp = data.xp.saturating_add(xp_gain);
    data.last_active_date = Some(chrono::Utc::now().format("%Y-%m-%d").to_string());

    // Check level up
    let new_level = ParrotLevel::level_for(data.xp, accuracy, data.current_level);
    if new_level > data.current_level {
        log::info!("Level up! {} -> {}", data.current_level.name(), new_level.name());
        data.current_level = new_level;
    }

    save(&data);
    data
}

pub fn get_info() -> LevelInfo {
    let data = load();
    let level = data.current_level;

    let next_level_xp = ParrotLevel::all()
        .iter()
        .find(|l| **l > level)
        .map(|l| l.xp_threshold());

    let xp_progress = match next_level_xp {
        Some(next) => {
            let current_threshold = level.xp_threshold();
            let range = next - current_threshold;
            if range > 0 {
                ((data.xp - current_threshold) as f64 / range as f64).min(1.0)
            } else {
                1.0
            }
        }
        None => 1.0, // Max level
    };

    LevelInfo {
        level,
        level_name: level.name().to_string(),
        emoji: level.emoji().to_string(),
        xp: data.xp,
        xp_threshold: level.xp_threshold(),
        next_level_xp,
        xp_progress,
    }
}
