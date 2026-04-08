use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HookEvent {
    pub session_id: String,
    pub cwd: String,
    pub event: String,
    pub status: String,
    pub pid: Option<i64>,
    pub tty: Option<String>,
    pub tool: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_use_id: Option<String>,
    pub user_prompt: Option<String>,
    pub permission_mode: Option<String>,
    pub interactive: Option<bool>,
}
