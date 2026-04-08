use std::path::Path;
use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;
use tokio::sync::broadcast;

use crate::models::hook_event::HookEvent;

pub const SOCKET_PATH: &str = "/tmp/periquito.sock";

pub fn start(tx: broadcast::Sender<HookEvent>) {
    tokio::spawn(async move {
        // Remove old socket if it exists
        if Path::new(SOCKET_PATH).exists() {
            let _ = std::fs::remove_file(SOCKET_PATH);
        }

        let listener = match UnixListener::bind(SOCKET_PATH) {
            Ok(l) => l,
            Err(e) => {
                log::error!("Failed to bind socket: {}", e);
                return;
            }
        };

        // Make socket world-writable so the hook can connect
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(
                SOCKET_PATH,
                std::fs::Permissions::from_mode(0o777),
            );
        }

        log::info!("Listening on {}", SOCKET_PATH);

        loop {
            match listener.accept().await {
                Ok((mut stream, _)) => {
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        let mut buf = Vec::new();
                        if let Err(e) = stream.read_to_end(&mut buf).await {
                            log::warn!("Failed to read from client: {}", e);
                            return;
                        }
                        if buf.is_empty() {
                            return;
                        }
                        match serde_json::from_slice::<HookEvent>(&buf) {
                            Ok(event) => {
                                log_event(&event);
                                let _ = tx.send(event);
                            }
                            Err(e) => {
                                log::warn!("Failed to parse event: {}", e);
                            }
                        }
                    });
                }
                Err(e) => {
                    log::error!("Accept error: {}", e);
                }
            }
        }
    });
}

pub fn cleanup() {
    if Path::new(SOCKET_PATH).exists() {
        let _ = std::fs::remove_file(SOCKET_PATH);
    }
}

fn log_event(event: &HookEvent) {
    match event.event.as_str() {
        "SessionStart" => log::info!("Session started: {}", event.session_id),
        "SessionEnd" => log::info!("Session ended: {}", event.session_id),
        "PreToolUse" => {
            log::info!("Tool: {}", event.tool.as_deref().unwrap_or("unknown"));
        }
        "PostToolUse" => {
            let tool = event.tool.as_deref().unwrap_or("unknown");
            let success = event.status != "error";
            log::info!("Result: {} {}", if success { "✓" } else { "✗" }, tool);
        }
        "UserPromptSubmit" => {
            let prompt = event.user_prompt.as_deref().unwrap_or("");
            let truncated: String = prompt.chars().take(60).collect();
            log::info!("Prompt: {}", truncated);
        }
        "Stop" | "SubagentStop" => log::info!("Done"),
        _ => {}
    }
}
