use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

const LANGUAGE: &str = "English";

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub emotion: String,
    pub intensity: f64,
    pub tip_type: String,
    pub tip: Option<String>,
    pub category: Option<String>,
}

impl AnalysisResult {
    fn skip() -> Self {
        Self {
            emotion: "neutral".to_string(),
            intensity: 0.0,
            tip_type: "skip".to_string(),
            tip: None,
            category: None,
        }
    }
}

pub async fn analyze(prompt: &str) -> AnalysisResult {
    let trimmed = prompt.trim();
    if trimmed.len() < 5 {
        log::info!("Prompt too short, skipping analysis");
        return AnalysisResult::skip();
    }

    match analyze_with_claude(trimmed).await {
        Ok(result) => {
            log_to_history(&result, trimmed);

            match result.tip_type.as_str() {
                "good" => AnalysisResult {
                    emotion: "happy".to_string(),
                    intensity: 0.7,
                    ..result
                },
                "correction" => AnalysisResult {
                    emotion: "sad".to_string(),
                    intensity: 0.6,
                    ..result
                },
                _ => AnalysisResult::skip(),
            }
        }
        Err(e) => {
            log::error!("English analysis failed: {}", e);
            AnalysisResult::skip()
        }
    }
}

async fn analyze_with_claude(prompt: &str) -> Result<AnalysisResult, String> {
    let claude_path = find_claude().ok_or("Claude CLI not found")?;

    let analysis_prompt = format!(
        r#"You are a concise {lang} tutor for a Brazilian Portuguese speaker. \
Analyze this text written/spoken in {lang}. \
If you find grammar mistakes, unnatural phrasing, wrong word choices, or pronunciation-related spelling errors, \
respond with ONLY valid JSON (no markdown, no backticks): \
{{"type":"correction","tip":"the tip text here","category":"grammar|spelling|word_choice|phrasing|punctuation"}}. \
The tip format should be: ❌ [what they said] → ✅ [correction] — [brief why]. \
If the text is NOT in {lang}, respond with: {{"type":"skip"}}. \
If the {lang} is good, respond with a helpful tip — suggest a synonym, \
a more natural phrasing, an idiom, a phrasal verb, or a vocabulary upgrade related to what they wrote. \
Format: {{"type":"good","tip":"💡 tip text here","category":"vocabulary|idiom|phrasal_verb|synonym|expression"}}. \
Keep the tip short (under 80 chars), practical, and relevant to their text. \
Text: "{prompt}""#,
        lang = LANGUAGE,
        prompt = prompt
    );

    let mut child = Command::new(&claude_path)
        .args([
            "-p",
            &analysis_prompt,
            "--output-format",
            "stream-json",
            "--verbose",
            "--max-turns",
            "1",
        ])
        .env("PERIQUITO_ANALYSIS_RUNNING", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn claude: {}", e))?;

    let mut stdout = child.stdout.take().ok_or("No stdout")?;
    let mut raw_output = String::new();
    stdout
        .read_to_string(&mut raw_output)
        .await
        .map_err(|e| format!("Failed to read stdout: {}", e))?;

    let _ = child.wait().await;

    // Parse stream-json: extract text from assistant events
    let mut text_parts: Vec<String> = Vec::new();
    for line in raw_output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if obj.get("type").and_then(|t| t.as_str()) == Some("assistant") {
                if let Some(content) = obj
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_array())
                {
                    for block in content {
                        if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                            if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                                text_parts.push(text.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    let output = text_parts.join("").trim().to_string();
    let json_str = extract_json(&output);

    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
        let tip_type = parsed
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("skip")
            .to_string();
        let tip = parsed.get("tip").and_then(|t| t.as_str()).map(|s| s.to_string());
        let category = parsed
            .get("category")
            .and_then(|c| c.as_str())
            .map(|s| s.to_string());

        Ok(AnalysisResult {
            emotion: String::new(), // filled by caller
            intensity: 0.0,
            tip_type,
            tip,
            category,
        })
    } else {
        // Fallback detection
        if output.contains("\"type\":\"good\"") || output.contains("Good") {
            Ok(AnalysisResult {
                emotion: String::new(),
                intensity: 0.0,
                tip_type: "good".to_string(),
                tip: None,
                category: None,
            })
        } else if output.contains('❌') || output.contains('→') || output.contains("correction") {
            Ok(AnalysisResult {
                emotion: String::new(),
                intensity: 0.0,
                tip_type: "correction".to_string(),
                tip: Some(output.chars().take(200).collect()),
                category: Some("other".to_string()),
            })
        } else {
            Ok(AnalysisResult::skip())
        }
    }
}

fn find_claude() -> Option<String> {
    let home = dirs::home_dir().unwrap_or_default();
    let candidates = vec![
        "/usr/local/bin/claude".to_string(),
        "/opt/homebrew/bin/claude".to_string(),
        format!("{}/.local/bin/claude", home.display()),
        format!("{}/.claude/local/claude", home.display()),
    ];

    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Some(path.clone());
        }
    }

    // Try which
    if let Ok(output) = std::process::Command::new("/usr/bin/which")
        .arg("claude")
        .output()
    {
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !result.is_empty() && std::path::Path::new(&result).exists() {
            return Some(result);
        }
    }

    None
}

fn extract_json(text: &str) -> String {
    let mut cleaned = text.trim().to_string();

    // Strip markdown code blocks
    if cleaned.starts_with("```") {
        if let Some(pos) = cleaned.find('\n') {
            cleaned = cleaned[pos + 1..].to_string();
        }
        if cleaned.ends_with("```") {
            cleaned = cleaned[..cleaned.len() - 3].to_string();
        }
        cleaned = cleaned.trim().to_string();
    }

    // Find first { to last }
    if let Some(start) = cleaned.find('{') {
        if let Some(end) = cleaned.rfind('}') {
            cleaned = cleaned[start..=end].to_string();
        }
    }

    cleaned
}

fn log_to_history(result: &AnalysisResult, prompt: &str) {
    let history_dir = dirs::home_dir()
        .unwrap_or_default()
        .join(".english-learning");
    let _ = std::fs::create_dir_all(&history_dir);
    let history_file = history_dir.join("history.jsonl");

    let today = chrono::Utc::now().to_rfc3339();
    let mut entry = serde_json::json!({
        "type": result.tip_type,
        "date": today,
        "prompt": &prompt[..prompt.len().min(200)]
    });

    if let Some(tip) = &result.tip {
        entry["tip"] = serde_json::Value::String(tip.clone());
    }
    if let Some(cat) = &result.category {
        entry["category"] = serde_json::Value::String(cat.clone());
    }

    if let Ok(line) = serde_json::to_string(&entry) {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&history_file)
        {
            let _ = writeln!(f, "{}", line);
        }
    }

    log::info!("Logged {} to history", result.tip_type);
}

fn history_file_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".english-learning")
        .join("history.jsonl")
}
