use std::sync::Arc;
use tauri::AppHandle;
use tauri::Emitter;
use tokio::sync::{broadcast, Mutex};

use crate::models::hook_event::HookEvent;
use crate::models::periquito_state::PeriquitoState;
use crate::models::session_data::EnglishTip;
use crate::services::session_store::SessionStore;

#[derive(Clone, serde::Serialize)]
pub struct StatePayload {
    pub unified_state: PeriquitoState,
    pub effective_session_id: Option<String>,
    pub active_session_count: usize,
    pub is_any_analyzing: bool,
}

#[derive(Clone, serde::Serialize)]
pub struct TipsPayload {
    pub all_tips: Vec<EnglishTip>,
}

pub fn start(
    app: AppHandle,
    store: Arc<Mutex<SessionStore>>,
    mut rx: broadcast::Receiver<HookEvent>,
) {
    // Event handler loop
    let app_clone = app.clone();
    let store_clone = store.clone();
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let is_session_end = event.event == "SessionEnd";
            let session_id = event.session_id.clone();

            // Check if we should analyze English
            let should_analyze = event.event == "UserPromptSubmit";
            let is_interactive;
            let user_prompt;

            {
                let mut store = store_clone.lock().await;
                let session = store.process(&event);
                session.reset_sleep_timer();
                is_interactive = session.is_interactive;
                user_prompt = event.user_prompt.clone();

                if should_analyze && is_interactive && user_prompt.is_some() {
                    session.is_analyzing_english = true;
                }
            }

            if is_session_end {
                let mut store = store_clone.lock().await;
                store.remove_session(&session_id);
            }

            emit_state(&app_clone, &store_clone).await;

            // Spawn English analysis in background
            if should_analyze && is_interactive {
                if let Some(prompt) = user_prompt {
                    let store_for_analysis = store_clone.clone();
                    let app_for_analysis = app_clone.clone();
                    let sid = session_id.clone();

                    tokio::spawn(async move {
                        let result = super::emotion_analyzer::analyze(&prompt).await;

                        {
                            let mut store = store_for_analysis.lock().await;
                            if let Some(session) = store.sessions.get_mut(&sid) {
                                session.emotion_state.record_emotion(
                                    &result.emotion,
                                    result.intensity,
                                );
                                session.record_english_tip(
                                    &result.tip_type,
                                    result.tip.as_deref(),
                                    result.category.as_deref(),
                                    &prompt,
                                );
                                session.is_analyzing_english = false;
                            }

                            // Increment XP and play sound
                            if result.tip_type == "good" || result.tip_type == "correction" {
                                let stats = super::history_stats::load();
                                let accuracy = stats.accuracy.unwrap_or(0);
                                super::level_manager::add_xp(&result.tip_type, accuracy);
                                super::sound_service::play_for_tip(&result.tip_type);
                            }
                        }

                        emit_state(&app_for_analysis, &store_for_analysis).await;
                    });
                }
            }
        }
    });

    // Emotion decay timer (every 60s)
    let app_clone = app.clone();
    let store_clone = store.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            {
                let mut store = store_clone.lock().await;
                store.decay_all_emotions();
                store.check_sleep_timers();
            }
            emit_state(&app_clone, &store_clone).await;
        }
    });
}

async fn emit_state(app: &AppHandle, store: &Arc<Mutex<SessionStore>>) {
    let store = store.lock().await;
    let payload = StatePayload {
        unified_state: store.effective_state(),
        effective_session_id: store.effective_session_id(),
        active_session_count: store.active_session_count(),
        is_any_analyzing: store.is_any_analyzing(),
    };
    let _ = app.emit("state-update", &payload);

    let tips_payload = TipsPayload {
        all_tips: store.all_tips(),
    };
    let _ = app.emit("tips-update", &tips_payload);
}
