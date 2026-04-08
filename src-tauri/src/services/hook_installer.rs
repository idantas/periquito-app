use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

fn claude_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".claude")
}

fn hooks_dir() -> PathBuf {
    claude_dir().join("hooks")
}

fn hook_script_path() -> PathBuf {
    hooks_dir().join("periquito-hook.sh")
}

fn settings_path() -> PathBuf {
    claude_dir().join("settings.json")
}

pub fn install(bundled_hook_content: &str) -> Result<bool, String> {
    let claude = claude_dir();
    if !claude.exists() {
        return Err("Claude Code not installed (~/.claude not found)".to_string());
    }

    let hooks = hooks_dir();
    fs::create_dir_all(&hooks).map_err(|e| format!("Failed to create hooks dir: {}", e))?;

    let script = hook_script_path();
    let _ = fs::remove_file(&script);
    fs::write(&script, bundled_hook_content)
        .map_err(|e| format!("Failed to write hook script: {}", e))?;
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("Failed to set permissions: {}", e))?;

    log::info!("Installed hook script to {}", script.display());
    update_settings()
}

pub fn is_installed() -> bool {
    let settings = settings_path();
    let Ok(data) = fs::read_to_string(&settings) else {
        return false;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) else {
        return false;
    };
    let Some(hooks) = json.get("hooks").and_then(|h| h.as_object()) else {
        return false;
    };
    hooks.values().any(|v| {
        if let Some(entries) = v.as_array() {
            entries.iter().any(|entry| {
                entry
                    .get("hooks")
                    .and_then(|h| h.as_array())
                    .map(|hooks| {
                        hooks.iter().any(|h| {
                            h.get("command")
                                .and_then(|c| c.as_str())
                                .map(|c| c.contains("periquito-hook.sh"))
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false)
            })
        } else {
            false
        }
    })
}

pub fn uninstall() {
    let _ = fs::remove_file(hook_script_path());

    let settings = settings_path();
    let Ok(data) = fs::read_to_string(&settings) else {
        return;
    };
    let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&data) else {
        return;
    };

    if let Some(hooks) = json.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        let keys: Vec<String> = hooks.keys().cloned().collect();
        for key in keys {
            if let Some(entries) = hooks.get_mut(&key).and_then(|v| v.as_array_mut()) {
                entries.retain(|entry| {
                    !entry
                        .get("hooks")
                        .and_then(|h| h.as_array())
                        .map(|hooks| {
                            hooks.iter().any(|h| {
                                h.get("command")
                                    .and_then(|c| c.as_str())
                                    .map(|c| c.contains("periquito-hook.sh"))
                                    .unwrap_or(false)
                            })
                        })
                        .unwrap_or(false)
                });
                if entries.is_empty() {
                    hooks.remove(&key);
                }
            }
        }
        if hooks.is_empty() {
            json.as_object_mut().unwrap().remove("hooks");
        }
    }

    if let Ok(data) = serde_json::to_string_pretty(&json) {
        let _ = fs::write(&settings, data);
    }

    log::info!("Uninstalled Periquito hooks");
}

fn update_settings() -> Result<bool, String> {
    let settings = settings_path();
    let mut json: serde_json::Value = if let Ok(data) = fs::read_to_string(&settings) {
        serde_json::from_str(&data).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let command = "~/.claude/hooks/periquito-hook.sh";
    let hook_entry = serde_json::json!([{"type": "command", "command": command}]);
    let with_matcher = serde_json::json!([{"matcher": "*", "hooks": hook_entry}]);
    let without_matcher = serde_json::json!([{"hooks": hook_entry}]);
    let pre_compact_config = serde_json::json!([
        {"matcher": "auto", "hooks": hook_entry},
        {"matcher": "manual", "hooks": hook_entry}
    ]);

    let hook_events = vec![
        ("UserPromptSubmit", &without_matcher),
        ("SessionStart", &without_matcher),
        ("PreToolUse", &with_matcher),
        ("PostToolUse", &with_matcher),
        ("PermissionRequest", &with_matcher),
        ("PreCompact", &pre_compact_config),
        ("Stop", &without_matcher),
        ("SubagentStop", &without_matcher),
        ("SessionEnd", &without_matcher),
    ];

    let hooks = json
        .as_object_mut()
        .unwrap()
        .entry("hooks")
        .or_insert(serde_json::json!({}));

    for (event, config) in hook_events {
        let has_our_hook = hooks
            .get(event)
            .and_then(|v| v.as_array())
            .map(|entries| {
                entries.iter().any(|entry| {
                    entry
                        .get("hooks")
                        .and_then(|h| h.as_array())
                        .map(|hooks| {
                            hooks.iter().any(|h| {
                                h.get("command")
                                    .and_then(|c| c.as_str())
                                    .map(|c| c.contains("periquito-hook.sh"))
                                    .unwrap_or(false)
                            })
                        })
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        if !has_our_hook {
            if let Some(existing) = hooks.get_mut(event).and_then(|v| v.as_array_mut()) {
                if let Some(config_arr) = config.as_array() {
                    existing.extend(config_arr.clone());
                }
            } else {
                hooks.as_object_mut().unwrap().insert(event.to_string(), config.clone());
            }
        }
    }

    let data = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    fs::write(&settings, data).map_err(|e| format!("Failed to write settings: {}", e))?;

    log::info!("Updated settings.json with Periquito hooks");
    Ok(true)
}
