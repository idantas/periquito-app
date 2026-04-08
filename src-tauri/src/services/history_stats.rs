use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

const DAILY_MINIMUM: usize = 3;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryStats {
    pub total_good: usize,
    pub total_corrections: usize,
    pub total_evaluated: usize,
    pub accuracy: Option<u32>,
    pub rolling_accuracy: Option<u32>,
    pub current_streak: u32,
    pub best_streak: u32,
    pub today_count: usize,
    pub daily_minimum: usize,
}

impl Default for HistoryStats {
    fn default() -> Self {
        Self {
            total_good: 0,
            total_corrections: 0,
            total_evaluated: 0,
            accuracy: None,
            rolling_accuracy: None,
            current_streak: 0,
            best_streak: 0,
            today_count: 0,
            daily_minimum: DAILY_MINIMUM,
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
    let mut day_counts: HashMap<String, usize> = HashMap::new();

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

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
                    _ => continue,
                }

                // Extract date for streak computation
                if let Some(date_str) = obj.get("date").and_then(|d| d.as_str()) {
                    let day = &date_str[..10.min(date_str.len())]; // "YYYY-MM-DD"
                    *day_counts.entry(day.to_string()).or_insert(0) += 1;
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

    let rolling = {
        let recent: Vec<&String> = evaluated_types.iter().rev().take(50).collect();
        if recent.is_empty() {
            None
        } else {
            let recent_good = recent.iter().filter(|t| t.as_str() == "good").count();
            Some((recent_good * 100 / recent.len()) as u32)
        }
    };

    let today_count = day_counts.get(&today).copied().unwrap_or(0);

    // Compute streaks
    let (current_streak, best_streak) = compute_streaks(&day_counts, &today);

    HistoryStats {
        total_good: good,
        total_corrections: corrections,
        total_evaluated: total,
        accuracy,
        rolling_accuracy: rolling,
        current_streak,
        best_streak,
        today_count,
        daily_minimum: DAILY_MINIMUM,
    }
}

fn compute_streaks(day_counts: &HashMap<String, usize>, today: &str) -> (u32, u32) {
    if day_counts.is_empty() {
        return (0, 0);
    }

    // Sort all dates
    let mut dates: Vec<&String> = day_counts.keys().collect();
    dates.sort();

    // Build list of qualifying days (>= DAILY_MINIMUM prompts)
    let qualifying: Vec<chrono::NaiveDate> = dates
        .iter()
        .filter(|d| day_counts.get(**d).copied().unwrap_or(0) >= DAILY_MINIMUM)
        .filter_map(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .collect();

    if qualifying.is_empty() {
        return (0, 0);
    }

    // Best streak: longest consecutive run
    let mut best = 1u32;
    let mut run = 1u32;
    for i in 1..qualifying.len() {
        if qualifying[i] - qualifying[i - 1] == chrono::Duration::days(1) {
            run += 1;
            best = best.max(run);
        } else {
            run = 1;
        }
    }

    // Current streak: walk backwards from today/yesterday
    let today_date = chrono::NaiveDate::parse_from_str(today, "%Y-%m-%d")
        .unwrap_or_else(|_| chrono::Local::now().date_naive());

    let today_qualifies = day_counts
        .get(today)
        .copied()
        .unwrap_or(0)
        >= DAILY_MINIMUM;

    let start = if today_qualifies {
        today_date
    } else {
        // Today doesn't count yet — check if yesterday was part of a streak
        today_date - chrono::Duration::days(1)
    };

    let mut current = 0u32;
    let mut check = start;
    loop {
        let key = check.format("%Y-%m-%d").to_string();
        if day_counts.get(&key).copied().unwrap_or(0) >= DAILY_MINIMUM {
            current += 1;
            check -= chrono::Duration::days(1);
        } else {
            break;
        }
    }

    best = best.max(current);

    (current, best)
}
