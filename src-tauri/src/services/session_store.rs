use std::collections::HashMap;

use crate::models::hook_event::HookEvent;
use crate::models::periquito_state::{PeriquitoEmotion, PeriquitoState, PeriquitoTask};
use crate::models::session_data::{EnglishTip, SessionData};

pub struct SessionStore {
    pub sessions: HashMap<String, SessionData>,
    next_session_number: HashMap<String, usize>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            next_session_number: HashMap::new(),
        }
    }

    pub fn active_session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn effective_session_id(&self) -> Option<String> {
        if self.sessions.len() == 1 {
            return self.sessions.keys().next().cloned();
        }
        self.sorted_session_ids().first().cloned()
    }

    pub fn effective_state(&self) -> PeriquitoState {
        self.effective_session_id()
            .and_then(|id| self.sessions.get(&id))
            .map(|s| s.state())
            .unwrap_or_default()
    }

    pub fn all_tips(&self) -> Vec<EnglishTip> {
        let mut tips: Vec<EnglishTip> = self
            .sessions
            .values()
            .flat_map(|s| s.english_tips.clone())
            .collect();
        tips.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        tips
    }

    pub fn is_any_analyzing(&self) -> bool {
        self.sessions.values().any(|s| s.is_analyzing_english)
    }

    pub fn current_emotion(&self) -> PeriquitoEmotion {
        self.effective_session_id()
            .and_then(|id| self.sessions.get(&id))
            .map(|s| s.state().emotion)
            .unwrap_or(PeriquitoEmotion::Neutral)
    }

    pub fn process(&mut self, event: &HookEvent) -> &mut SessionData {
        let is_interactive = event.interactive.unwrap_or(true);
        self.get_or_create(&event.session_id, &event.cwd, is_interactive);

        let session = self.sessions.get_mut(&event.session_id).unwrap();
        let is_processing = event.status != "waiting_for_input";
        session.update_processing(is_processing);

        if let Some(mode) = &event.permission_mode {
            session.update_permission_mode(mode);
        }

        match event.event.as_str() {
            "UserPromptSubmit" => {
                if let Some(prompt) = &event.user_prompt {
                    session.record_user_prompt(prompt);
                }
                if SessionData::is_local_slash_command(event.user_prompt.as_deref()) {
                    session.update_task(PeriquitoTask::Idle);
                } else {
                    session.update_task(PeriquitoTask::Working);
                }
            }
            "PreCompact" => {
                session.update_task(PeriquitoTask::Compacting);
            }
            "SessionStart" => {
                if is_processing {
                    session.update_task(PeriquitoTask::Working);
                }
            }
            "PreToolUse" => {
                session.record_pre_tool_use(
                    event.tool.as_deref(),
                    event.tool_use_id.as_deref(),
                );
                if event.tool.as_deref() == Some("AskUserQuestion") {
                    session.update_task(PeriquitoTask::Waiting);
                } else {
                    session.update_task(PeriquitoTask::Working);
                }
            }
            "PermissionRequest" => {
                session.update_task(PeriquitoTask::Waiting);
            }
            "PostToolUse" => {
                let success = event.status != "error";
                session.record_post_tool_use(event.tool_use_id.as_deref(), success);
                session.update_task(PeriquitoTask::Working);
            }
            "Stop" | "SubagentStop" => {
                session.update_task(PeriquitoTask::Idle);
            }
            "SessionEnd" => {
                session.end_session();
            }
            _ => {
                if !is_processing && session.task != PeriquitoTask::Idle {
                    session.update_task(PeriquitoTask::Idle);
                }
            }
        }

        self.sessions.get_mut(&event.session_id).unwrap()
    }

    pub fn remove_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
        log::info!("Removed session: {}", session_id);
    }

    pub fn decay_all_emotions(&mut self) {
        for session in self.sessions.values_mut() {
            session.emotion_state.decay_all();
        }
    }

    pub fn check_sleep_timers(&mut self) {
        for session in self.sessions.values_mut() {
            session.check_sleep();
        }
    }

    fn get_or_create(&mut self, session_id: &str, cwd: &str, is_interactive: bool) {
        if self.sessions.contains_key(session_id) {
            return;
        }
        let project_name = cwd.rsplit('/').next().unwrap_or("unknown").to_string();
        let num = self.next_session_number.entry(project_name).or_insert(0);
        *num += 1;
        let session = SessionData::new(session_id.to_string(), cwd.to_string(), *num, is_interactive);
        log::info!("Created session #{}: {} at {}", num, session_id, cwd);
        self.sessions.insert(session_id.to_string(), session);
    }

    fn sorted_session_ids(&self) -> Vec<String> {
        let mut entries: Vec<(&String, &SessionData)> = self.sessions.iter().collect();
        entries.sort_by(|a, b| {
            if a.1.is_processing != b.1.is_processing {
                return b.1.is_processing.cmp(&a.1.is_processing);
            }
            b.1.last_activity.cmp(&a.1.last_activity)
        });
        entries.into_iter().map(|(id, _)| id.clone()).collect()
    }
}
