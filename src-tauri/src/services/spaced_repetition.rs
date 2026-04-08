use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::distractor_engine;

/// Leitner box intervals in seconds
const INTERVALS: [i64; 5] = [
    3600,      // box 1: 1 hour
    86400,     // box 2: 1 day
    259200,    // box 3: 3 days
    604800,    // box 4: 7 days
    1209600,   // box 5: 14 days (mastered)
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizItem {
    pub id: String,
    pub incorrect_sentence: String,
    pub correct_sentence: String,
    pub explanation: String,
    pub category: String,
    #[serde(rename = "box")]
    pub leitner_box: u8,
    pub next_review_date: DateTime<Utc>,
    pub total_reviews: u32,
    pub correct_count: u32,
}

impl QuizItem {
    pub fn new(incorrect: String, correct: String, explanation: String, category: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            incorrect_sentence: incorrect,
            correct_sentence: correct,
            explanation,
            category,
            leitner_box: 1,
            next_review_date: Utc::now(),
            total_reviews: 0,
            correct_count: 0,
        }
    }

    pub fn is_due(&self) -> bool {
        Utc::now() >= self.next_review_date
    }

    pub fn record_answer(&mut self, correct: bool) {
        self.total_reviews += 1;
        if correct {
            self.correct_count += 1;
            self.leitner_box = (self.leitner_box + 1).min(5);
        } else {
            self.correct_count = 0;
            self.leitner_box = (self.leitner_box).max(2) - 1; // min box 1
        }
        let interval = INTERVALS[(self.leitner_box - 1) as usize];
        self.next_review_date = Utc::now() + chrono::Duration::seconds(interval);
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizQuestion {
    pub item: QuizItem,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizResult {
    pub correct: bool,
    pub correct_answer: String,
    pub explanation: String,
    pub leitner_box: u8,
    pub correct_count: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewStats {
    pub total_items: usize,
    pub due_count: usize,
    pub mastered_count: usize,
}

fn reviews_file() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".english-learning")
        .join("reviews.json")
}

fn load_items() -> Vec<QuizItem> {
    let path = reviews_file();
    match std::fs::read_to_string(&path) {
        Ok(data) => {
            let items: Vec<QuizItem> = serde_json::from_str(&data).unwrap_or_default();
            // Corruption detection: if all items are box 1 with 0 correct, reset
            if items.len() > 5 && items.iter().all(|i| i.leitner_box == 1 && i.correct_count == 0) {
                log::warn!("Detected corrupted reviews, resetting");
                return Vec::new();
            }
            items
        }
        Err(_) => Vec::new(),
    }
}

fn save_items(items: &[QuizItem]) {
    let path = reviews_file();
    let dir = path.parent().unwrap();
    let _ = std::fs::create_dir_all(dir);
    if let Ok(json) = serde_json::to_string_pretty(items) {
        let _ = std::fs::write(&path, json);
    }
}

/// Sync corrections from history.jsonl into review items
pub fn sync_from_history() {
    let corrections = distractor_engine::load_from_history();
    let mut items = load_items();

    let existing: std::collections::HashSet<String> = items
        .iter()
        .map(|i| i.incorrect_sentence.to_lowercase().trim().to_string())
        .collect();

    let mut added = 0;
    for c in &corrections {
        let key = c.wrong.to_lowercase().trim().to_string();
        if !existing.contains(&key) && !c.right.is_empty() {
            items.push(QuizItem::new(
                c.wrong.clone(),
                c.right.clone(),
                c.why.clone(),
                c.category.clone(),
            ));
            added += 1;
        }
    }

    if added > 0 {
        log::info!("Synced {} new corrections into review queue", added);
        save_items(&items);
    }
}

/// Get the next due quiz item with options
pub fn next_quiz() -> Option<QuizQuestion> {
    sync_from_history();
    let items = load_items();
    let corrections = distractor_engine::load_from_history();

    // Find next due item, prioritize lower boxes
    let mut due: Vec<&QuizItem> = items.iter().filter(|i| i.is_due()).collect();
    due.sort_by_key(|i| i.leitner_box);

    let item = (*due.first()?).clone();
    let options = distractor_engine::generate_options(&item, &corrections);

    Some(QuizQuestion { item, options })
}

/// Submit an answer and return the result
pub fn submit_answer(item_id: &str, answer: &str) -> Option<QuizResult> {
    let mut items = load_items();
    let item = items.iter_mut().find(|i| i.id == item_id)?;

    let correct = answer == item.correct_sentence;
    item.record_answer(correct);

    let result = QuizResult {
        correct,
        correct_answer: item.correct_sentence.clone(),
        explanation: item.explanation.clone(),
        leitner_box: item.leitner_box,
        correct_count: item.correct_count,
    };

    save_items(&items);

    // Award XP
    let stats = super::history_stats::load();
    let accuracy = stats.accuracy.unwrap_or(0);
    let tip_type = if correct { "good" } else { "correction" };
    super::level_manager::add_xp(tip_type, accuracy);

    Some(result)
}

/// Get review queue stats
pub fn get_stats() -> ReviewStats {
    let items = load_items();
    ReviewStats {
        total_items: items.len(),
        due_count: items.iter().filter(|i| i.is_due()).count(),
        mastered_count: items.iter().filter(|i| i.leitner_box >= 5).count(),
    }
}
