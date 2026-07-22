use super::{Segment, SegmentData};
use crate::config::{InputData, Message, ModelConfig, SegmentId, TranscriptEntry};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Default)]
pub struct ContextWindowSegment;

impl ContextWindowSegment {
    pub fn new() -> Self {
        Self
    }

    /// Get context limit for the specified model
    fn get_context_limit_for_model(model_id: &str) -> u32 {
        let model_config = ModelConfig::load();
        model_config.get_context_limit(model_id)
    }
}

impl Segment for ContextWindowSegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        let model_context_limit = Self::get_context_limit_for_model(&input.model.id) as u64;
        let official_tokens = input
            .context_window
            .as_ref()
            .and_then(|context| context.total_input_tokens)
            .filter(|tokens| *tokens > 0);
        let official_percentage = input
            .context_window
            .as_ref()
            .and_then(|context| context.used_percentage)
            .filter(|percentage| percentage.is_finite() && *percentage > 0.0);
        let transcript_tokens = if official_tokens.is_none() {
            parse_transcript_usage(&input.transcript_path).map(u64::from)
        } else {
            None
        };

        let context_used_token = official_tokens.or(transcript_tokens).unwrap_or(0);
        let context_limit = input
            .context_window
            .as_ref()
            .and_then(|context| context.context_window_size)
            .filter(|limit| *limit > 0)
            .unwrap_or(model_context_limit);
        let context_used_rate = official_percentage.unwrap_or_else(|| {
            if context_used_token == 0 || context_limit == 0 {
                0.0
            } else {
                (context_used_token as f64 / context_limit as f64) * 100.0
            }
        });
        let percentage_display = if context_used_rate.fract() == 0.0 {
            format!("{:.0}%", context_used_rate)
        } else {
            format!("{:.1}%", context_used_rate)
        };
        let tokens_display = format_tokens(context_used_token);

        let mut metadata = HashMap::new();
        metadata.insert("tokens".to_string(), context_used_token.to_string());
        metadata.insert("percentage".to_string(), context_used_rate.to_string());
        metadata.insert("limit".to_string(), context_limit.to_string());
        metadata.insert("model".to_string(), input.model.id.clone());
        metadata.insert(
            "source".to_string(),
            if official_tokens.is_some() || official_percentage.is_some() {
                "statusline_context_window"
            } else if transcript_tokens.is_some() {
                "transcript"
            } else {
                "empty"
            }
            .to_string(),
        );

        Some(SegmentData {
            primary: format!("{} · {} tokens", percentage_display, tokens_display),
            secondary: String::new(),
            secondary_color: None,
            metadata,
        })
    }

    fn id(&self) -> SegmentId {
        SegmentId::ContextWindow
    }
}

fn parse_transcript_usage<P: AsRef<Path>>(transcript_path: P) -> Option<u32> {
    let path = transcript_path.as_ref();

    if path.exists() {
        try_parse_transcript_file(path)
    } else {
        None
    }
}

fn try_parse_transcript_file(path: &Path) -> Option<u32> {
    let lines = read_lines(path)?;

    // Check if the last line is a summary
    if let Some(entry) = parse_entry(lines.last()?) {
        if entry.r#type.as_deref() == Some("summary") {
            // Handle summary case: find usage by leafUuid
            if let Some(leaf_uuid) = &entry.leaf_uuid {
                let project_dir = path.parent()?;
                return find_usage_by_leaf_uuid(leaf_uuid, project_dir);
            }
        }
    }

    // Prefer the latest completed assistant message. Streaming intermediate
    // entries may contain incomplete usage snapshots.
    let mut fallback_usage = None;
    for line in lines.iter().rev() {
        if let Some(entry) = parse_entry(line) {
            if entry.r#type.as_deref() == Some("assistant") {
                if let Some(message) = &entry.message {
                    if let Some(tokens) = extract_usage(message, true) {
                        return Some(tokens);
                    }
                    if fallback_usage.is_none() {
                        fallback_usage = extract_usage(message, false);
                    }
                }
            }
        }
    }

    fallback_usage
}

fn find_usage_by_leaf_uuid(leaf_uuid: &str, project_dir: &Path) -> Option<u32> {
    // Search for the leafUuid across all session files in the project directory
    let entries = fs::read_dir(project_dir).ok()?;

    for entry in entries {
        let entry = entry.ok()?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
            continue;
        }

        if let Some(usage) = search_uuid_in_file(&path, leaf_uuid) {
            return Some(usage);
        }
    }

    None
}

fn search_uuid_in_file(path: &Path, target_uuid: &str) -> Option<u32> {
    let lines = read_lines(path)?;

    // Find the message with target_uuid
    for line in &lines {
        if let Some(entry) = parse_entry(line) {
            if let Some(uuid) = &entry.uuid {
                if uuid == target_uuid {
                    // Found the target message, check its type
                    if entry.r#type.as_deref() == Some("assistant") {
                        // Direct assistant message with usage
                        if let Some(message) = &entry.message {
                            if let Some(tokens) = extract_usage(message, false) {
                                return Some(tokens);
                            }
                        }
                    } else if entry.r#type.as_deref() == Some("user") {
                        // User message, need to find the parent assistant message
                        if let Some(parent_uuid) = &entry.parent_uuid {
                            return find_assistant_message_by_uuid(&lines, parent_uuid);
                        }
                    }
                    break;
                }
            }
        }
    }

    None
}

fn find_assistant_message_by_uuid(lines: &[String], target_uuid: &str) -> Option<u32> {
    for line in lines {
        if let Some(entry) = parse_entry(line) {
            if let Some(uuid) = &entry.uuid {
                if uuid == target_uuid && entry.r#type.as_deref() == Some("assistant") {
                    if let Some(message) = &entry.message {
                        if let Some(tokens) = extract_usage(message, false) {
                            return Some(tokens);
                        }
                    }
                }
            }
        }
    }

    None
}

fn read_lines(path: &Path) -> Option<Vec<String>> {
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    Some(reader.lines().map_while(Result::ok).collect())
}

fn parse_entry(line: &str) -> Option<TranscriptEntry> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    serde_json::from_str(line).ok()
}

fn extract_usage(message: &Message, require_complete: bool) -> Option<u32> {
    if require_complete && message.stop_reason.is_none() {
        return None;
    }

    let tokens = message
        .usage
        .clone()
        .map(|usage| usage.normalize().display_tokens())?;

    // Claude Code writes a completed-looking, all-zero usage placeholder when
    // the user interrupts a response. Ignore it so the previous valid context
    // snapshot remains visible until another API response arrives.
    (tokens > 0).then_some(tokens)
}

fn format_tokens(tokens: u64) -> String {
    if tokens >= 1000 {
        let k_value = tokens as f64 / 1000.0;
        if k_value.fract() == 0.0 {
            format!("{}k", k_value as u64)
        } else {
            format!("{:.1}k", k_value)
        }
    } else {
        tokens.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn write_temp_transcript(contents: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sequence = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "best-claude-hud-{}-{nanos}-{sequence}.jsonl",
            std::process::id()
        ));
        std::fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn transcript_parser_prefers_latest_complete_usage() {
        let path = write_temp_transcript(
            r#"{"type":"assistant","message":{"usage":{"input_tokens":1000}}}
{"type":"assistant","message":{"usage":{"input_tokens":2500},"stop_reason":"end_turn"}}
{"type":"assistant","message":{"usage":{"input_tokens":9999}}}
"#,
        );

        let usage = try_parse_transcript_file(&path);
        std::fs::remove_file(path).unwrap();

        assert_eq!(usage, Some(2500));
    }

    #[test]
    fn transcript_parser_ignores_zero_usage_from_interrupted_response() {
        let path = write_temp_transcript(
            r#"{"type":"assistant","message":{"usage":{"input_tokens":2566,"cache_read_input_tokens":275456,"output_tokens":557},"stop_reason":"tool_use"}}
{"type":"assistant","message":{"usage":{"input_tokens":0,"output_tokens":0,"cache_creation_input_tokens":0,"cache_read_input_tokens":0},"stop_reason":"stop_sequence"}}
{"type":"user","message":{"content":[{"type":"text","text":"[Request interrupted by user]"}]}}
"#,
        );

        let usage = try_parse_transcript_file(&path);
        std::fs::remove_file(path).unwrap();

        assert_eq!(usage, Some(278_579));
    }

    #[test]
    fn context_segment_prefers_official_statusline_data() {
        let path = write_temp_transcript(
            r#"{"type":"assistant","message":{"usage":{"input_tokens":99999},"stop_reason":"end_turn"}}"#,
        );
        let input: InputData = serde_json::from_value(serde_json::json!({
            "model": { "id": "kimi-k2.7-code", "display_name": "Kimi K2.7" },
            "workspace": { "current_dir": "/tmp/project" },
            "transcript_path": path,
            "context_window": {
                "total_input_tokens": 24000,
                "context_window_size": 262144,
                "used_percentage": 9.2
            }
        }))
        .unwrap();

        let data = ContextWindowSegment::new().collect(&input).unwrap();
        std::fs::remove_file(&input.transcript_path).unwrap();

        assert_eq!(data.primary, "9.2% · 24k tokens");
        assert_eq!(
            data.metadata.get("source").map(String::as_str),
            Some("statusline_context_window")
        );
    }

    #[test]
    fn zero_statusline_snapshot_falls_back_to_pre_interrupt_usage() {
        let path = write_temp_transcript(
            r#"{"type":"assistant","message":{"usage":{"input_tokens":2566,"cache_read_input_tokens":275456,"output_tokens":557},"stop_reason":"tool_use"}}
{"type":"assistant","message":{"usage":{"input_tokens":0,"output_tokens":0,"cache_creation_input_tokens":0,"cache_read_input_tokens":0},"stop_reason":"stop_sequence"}}
"#,
        );
        let input: InputData = serde_json::from_value(serde_json::json!({
            "model": { "id": "k3[1m]", "display_name": "K3" },
            "workspace": { "current_dir": "/tmp/project" },
            "transcript_path": path,
            "context_window": {
                "total_input_tokens": 0,
                "context_window_size": 1000000,
                "used_percentage": 0
            }
        }))
        .unwrap();

        let data = ContextWindowSegment::new().collect(&input).unwrap();
        std::fs::remove_file(&input.transcript_path).unwrap();

        assert_eq!(data.primary, "27.9% · 278.6k tokens");
        assert_eq!(
            data.metadata.get("source").map(String::as_str),
            Some("transcript")
        );
    }

    #[test]
    fn empty_new_session_still_displays_zero_usage() {
        let missing = std::env::temp_dir().join("best-claude-hud-empty-session.jsonl");
        let input: InputData = serde_json::from_value(serde_json::json!({
            "model": { "id": "k3[1m]", "display_name": "K3" },
            "workspace": { "current_dir": "/tmp/project" },
            "transcript_path": missing,
            "context_window": {
                "total_input_tokens": 0,
                "context_window_size": 1000000,
                "used_percentage": 0
            }
        }))
        .unwrap();

        let data = ContextWindowSegment::new().collect(&input).unwrap();

        assert_eq!(data.primary, "0% · 0 tokens");
        assert_eq!(
            data.metadata.get("source").map(String::as_str),
            Some("empty")
        );
    }

    #[test]
    fn missing_transcript_does_not_use_project_history() {
        let missing = std::env::temp_dir().join("best-claude-hud-missing.jsonl");
        assert_eq!(parse_transcript_usage(missing), None);
    }

    #[test]
    fn new_session_does_not_inherit_tokens_from_old_sibling_transcripts() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("best-claude-hud-session-{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();

        let old_transcript = dir.join("old-session.jsonl");
        std::fs::write(
            &old_transcript,
            r#"{"type":"assistant","message":{"usage":{"input_tokens":88888},"stop_reason":"end_turn"}}"#,
        )
        .unwrap();

        let new_transcript = dir.join("new-terminal-session.jsonl");
        let usage = parse_transcript_usage(&new_transcript);

        std::fs::remove_file(old_transcript).unwrap();
        std::fs::remove_dir(dir).unwrap();

        assert_eq!(usage, None);
    }
}
