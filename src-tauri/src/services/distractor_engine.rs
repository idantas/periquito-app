use rand::seq::SliceRandom;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct HistoryCorrection {
    pub wrong: String,
    pub right: String,
    pub why: String,
    pub category: String,
}

/// Parse tip format: "❌ wrong → ✅ right — explanation"
fn parse_tip(tip: &str) -> Option<(String, String, String)> {
    // Take first segment if multiple corrections separated by "; "
    let segment = tip.split("; ").next().unwrap_or(tip);

    // Find the arrow separator
    let arrow_pos = segment.find(" → ")?;
    let left = &segment[..arrow_pos];
    let right_part = &segment[arrow_pos + " → ".len()..];

    // Clean left side (remove ❌ emoji)
    let wrong = left
        .trim_start_matches('❌')
        .trim_start_matches(" ❌")
        .trim()
        .to_string();

    // Split right side by " — " for explanation
    let (correct, why) = if let Some(dash_pos) = right_part.find(" — ") {
        let c = &right_part[..dash_pos];
        let w = &right_part[dash_pos + " — ".len()..];
        (c, w.to_string())
    } else if let Some(dash_pos) = right_part.find(" - ") {
        let c = &right_part[..dash_pos];
        let w = &right_part[dash_pos + " - ".len()..];
        (c, w.to_string())
    } else {
        (right_part, String::new())
    };

    // Clean correct side (remove ✅ emoji)
    let correct = correct
        .trim_start_matches('✅')
        .trim_start_matches(" ✅")
        .trim()
        .to_string();

    if wrong.is_empty() || correct.is_empty() {
        return None;
    }

    Some((wrong, correct, why))
}

/// Load corrections from history.jsonl
pub fn load_from_history() -> Vec<HistoryCorrection> {
    let path = history_file();
    let data = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let mut corrections = Vec::new();

    for line in data.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(trimmed) {
            let tip_type = obj.get("type").and_then(|t| t.as_str()).unwrap_or("");
            if tip_type != "correction" {
                continue;
            }

            let tip = obj.get("tip").and_then(|t| t.as_str()).unwrap_or("");
            let category = obj
                .get("category")
                .and_then(|c| c.as_str())
                .unwrap_or("other")
                .to_string();

            if let Some((wrong, right, why)) = parse_tip(tip) {
                corrections.push(HistoryCorrection {
                    wrong,
                    right,
                    why,
                    category,
                });
            }
        }
    }

    corrections
}

/// Generate multiple-choice options for a quiz item
pub fn generate_options(
    item: &super::spaced_repetition::QuizItem,
    corrections: &[HistoryCorrection],
) -> Vec<String> {
    let mut rng = rand::rng();

    // Find same-category distractors
    let mut distractors: Vec<String> = corrections
        .iter()
        .filter(|c| {
            c.category == item.category
                && c.wrong != item.incorrect_sentence
                && c.right != item.correct_sentence
        })
        .map(|c| c.wrong.clone())
        .collect();

    distractors.shuffle(&mut rng);
    distractors.truncate(2);

    // Build options: correct + incorrect + distractors
    let mut options = vec![item.correct_sentence.clone(), item.incorrect_sentence.clone()];
    options.extend(distractors);

    // Deduplicate
    let mut seen = std::collections::HashSet::new();
    options.retain(|o| seen.insert(o.clone()));

    options.shuffle(&mut rng);
    options
}

fn history_file() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".english-learning")
        .join("history.jsonl")
}
