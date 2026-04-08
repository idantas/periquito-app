use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::periquito_state::{PeriquitoState, PeriquitoTask};
use crate::services::emotion_state::EmotionState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnglishTip {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub prompt: String,
    #[serde(rename = "type")]
    pub tip_type: String,
    pub tip: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub tool: Option<String>,
    pub status: String,
    pub tool_use_id: Option<String>,
    pub description: Option<String>,
}

pub struct SessionData {
    pub id: String,
    pub cwd: String,
    pub session_number: usize,
    pub session_start_time: DateTime<Utc>,
    pub is_interactive: bool,
    pub task: PeriquitoTask,
    pub emotion_state: EmotionState,
    pub is_processing: bool,
    pub last_activity: DateTime<Utc>,
    pub recent_events: Vec<SessionEvent>,
    pub english_tips: Vec<EnglishTip>,
    pub is_analyzing_english: bool,
    pub last_user_prompt: Option<String>,
    pub permission_mode: String,
    sleep_deadline: Option<DateTime<Utc>>,
}

static LOCAL_SLASH_COMMANDS: &[&str] = &[
    "/clear", "/help", "/cost", "/status",
    "/vim", "/fast", "/model", "/login", "/logout",
];

impl SessionData {
    pub fn new(session_id: String, cwd: String, session_number: usize, is_interactive: bool) -> Self {
        let now = Utc::now();
        Self {
            id: session_id,
            cwd,
            session_number,
            session_start_time: now,
            is_interactive,
            task: PeriquitoTask::Idle,
            emotion_state: EmotionState::new(),
            is_processing: false,
            last_activity: now,
            recent_events: Vec::new(),
            english_tips: Vec::new(),
            is_analyzing_english: false,
            last_user_prompt: None,
            permission_mode: "default".to_string(),
            sleep_deadline: None,
        }
    }

    pub fn state(&self) -> PeriquitoState {
        PeriquitoState {
            task: self.task,
            emotion: self.emotion_state.current_emotion(),
        }
    }

    pub fn project_name(&self) -> String {
        self.cwd
            .rsplit('/')
            .next()
            .unwrap_or("unknown")
            .to_string()
    }

    pub fn update_task(&mut self, new_task: PeriquitoTask) {
        self.task = new_task;
        self.last_activity = Utc::now();
    }

    pub fn update_processing(&mut self, is_processing: bool) {
        self.is_processing = is_processing;
        self.last_activity = Utc::now();
    }

    pub fn record_user_prompt(&mut self, prompt: &str) {
        self.last_user_prompt = Some(prompt.chars().take(100).collect());
        self.last_activity = Utc::now();
    }

    pub fn update_permission_mode(&mut self, mode: &str) {
        self.permission_mode = mode.to_string();
    }

    pub fn record_pre_tool_use(&mut self, tool: Option<&str>, tool_use_id: Option<&str>) {
        let event = SessionEvent {
            timestamp: Utc::now(),
            event_type: "PreToolUse".to_string(),
            tool: tool.map(|s| s.to_string()),
            status: "running".to_string(),
            tool_use_id: tool_use_id.map(|s| s.to_string()),
            description: tool.map(|t| t.to_string()),
        };
        self.recent_events.push(event);
        if self.recent_events.len() > 20 {
            self.recent_events.remove(0);
        }
        self.last_activity = Utc::now();
    }

    pub fn record_post_tool_use(&mut self, tool_use_id: Option<&str>, success: bool) {
        let status = if success { "success" } else { "error" };
        if let Some(id) = tool_use_id {
            if let Some(ev) = self.recent_events.iter_mut().rev().find(|e| {
                e.tool_use_id.as_deref() == Some(id) && e.status == "running"
            }) {
                ev.status = status.to_string();
                self.last_activity = Utc::now();
                return;
            }
        }
        let event = SessionEvent {
            timestamp: Utc::now(),
            event_type: "PostToolUse".to_string(),
            tool: None,
            status: status.to_string(),
            tool_use_id: tool_use_id.map(|s| s.to_string()),
            description: None,
        };
        self.recent_events.push(event);
        if self.recent_events.len() > 20 {
            self.recent_events.remove(0);
        }
        self.last_activity = Utc::now();
    }

    pub fn record_english_tip(&mut self, tip_type: &str, tip: Option<&str>, category: Option<&str>, prompt: &str) {
        if tip_type == "skip" {
            return;
        }
        let english_tip = EnglishTip {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            prompt: prompt.chars().take(100).collect(),
            tip_type: tip_type.to_string(),
            tip: tip.map(|s| s.to_string()),
            category: category.map(|s| s.to_string()),
        };
        self.english_tips.push(english_tip);
        while self.english_tips.len() > 20 {
            self.english_tips.remove(0);
        }
        self.last_activity = Utc::now();
    }

    pub fn reset_sleep_timer(&mut self) {
        self.sleep_deadline = Some(Utc::now() + chrono::Duration::seconds(300));
    }

    pub fn check_sleep(&mut self) -> bool {
        if let Some(deadline) = self.sleep_deadline {
            if Utc::now() >= deadline {
                self.task = PeriquitoTask::Sleeping;
                self.sleep_deadline = None;
                return true;
            }
        }
        false
    }

    pub fn is_local_slash_command(prompt: Option<&str>) -> bool {
        match prompt {
            Some(p) if p.starts_with('/') => {
                let cmd = p.split_whitespace().next().unwrap_or("");
                LOCAL_SLASH_COMMANDS.contains(&cmd)
            }
            _ => false,
        }
    }

    pub fn end_session(&mut self) {
        self.is_processing = false;
        self.sleep_deadline = None;
    }
}
