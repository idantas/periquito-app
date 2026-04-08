use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct HistoryStats {
    pub total_good: usize,
    pub total_corrections: usize,
    pub total_evaluated: usize,
    pub accuracy: Option<u32>,
    pub rolling_accuracy: Option<u32>,
}

impl Default for HistoryStats {
    fn default() -> Self {
        Self {
            total_good: 0,
            total_corrections: 0,
            total_evaluated: 0,
            accuracy: None,
            rolling_accuracy: None,
        }
    }
}

fn history_file() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".english-learning")
        .join("history.jsonl")
}

pub fn load() -> HistoryStats {
    let path = history_file();
    let data = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(_) => return HistoryStats::default(),
    };

    let mut good = 0usize;
    let mut corrections = 0usize;
    let mut evaluated_types: Vec<String> = Vec::new();

    for line in data.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(tip_type) = obj.get("type").and_then(|t| t.as_str()) {
                match tip_type {
                    "good" => {
                        good += 1;
                        evaluated_types.push("good".to_string());
                    }
                    "correction" => {
                        corrections += 1;
                        evaluated_types.push("correction".to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    let total = good + corrections;
    let accuracy = if total > 0 {
        Some((good * 100 / total) as u32)
    } else {
        None
    };

    // Rolling accuracy: last 50
    let rolling = {
        let recent: Vec<&String> = evaluated_types.iter().rev().take(50).collect();
        if recent.is_empty() {
            None
        } else {
            let recent_good = recent.iter().filter(|t| t.as_str() == "good").count();
            Some((recent_good * 100 / recent.len()) as u32)
        }
    };

    HistoryStats {
        total_good: good,
        total_corrections: corrections,
        total_evaluated: total,
        accuracy,
        rolling_accuracy: rolling,
    }
}
