use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;
use tauri::Emitter;

use super::focus_detector;

/// How long on a procrastination app before quiz triggers (seconds)
const IDLE_THRESHOLD: u64 = 120;
/// How often to poll frontmost app (seconds)
const POLL_INTERVAL: u64 = 15;
/// Minimum time between quiz triggers (seconds)
const QUIZ_COOLDOWN: u64 = 300;

static IDLE_SINCE: AtomicU64 = AtomicU64::new(0);
static LAST_QUIZ_TRIGGER: AtomicU64 = AtomicU64::new(0);

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Clone, serde::Serialize)]
pub struct QuizTriggerPayload {
    pub reason: String,
}

/// Start the idle detection loop
pub fn start(app: AppHandle) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL)).await;
            check_and_trigger(&app);
        }
    });
}

fn check_and_trigger(app: &AppHandle) {
    let now = now_secs();

    if focus_detector::is_terminal_focused() {
        // Productive — reset idle
        IDLE_SINCE.store(0, Ordering::Relaxed);
        return;
    }

    if focus_detector::is_procrastinating() {
        let idle_since = IDLE_SINCE.load(Ordering::Relaxed);

        if idle_since == 0 {
            // Start idle timer
            IDLE_SINCE.store(now, Ordering::Relaxed);
            log::info!("User on procrastination app, starting idle timer");
            return;
        }

        let idle_duration = now - idle_since;
        let last_trigger = LAST_QUIZ_TRIGGER.load(Ordering::Relaxed);
        let since_last_quiz = now - last_trigger;

        if idle_duration >= IDLE_THRESHOLD && since_last_quiz >= QUIZ_COOLDOWN {
            // Check if there are due quizzes
            let stats = super::spaced_repetition::get_stats();
            if stats.due_count > 0 {
                log::info!(
                    "Triggering quiz: idle {}s, {} due items",
                    idle_duration,
                    stats.due_count
                );
                LAST_QUIZ_TRIGGER.store(now, Ordering::Relaxed);
                let _ = app.emit(
                    "quiz-trigger",
                    QuizTriggerPayload {
                        reason: "procrastination".to_string(),
                    },
                );
            }
        }
    } else {
        // Not procrastinating, not terminal — neutral app, reset idle
        IDLE_SINCE.store(0, Ordering::Relaxed);
    }
}
