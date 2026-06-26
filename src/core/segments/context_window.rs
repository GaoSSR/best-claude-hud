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
        // Dynamically determine context limit based on current model ID
        let context_limit = Self::get_context_limit_for_model(&input.model.id);

        let context_used_token_opt = parse_transcript_usage(&input.transcript_path);

        let context_used_token = context_used_token_opt.unwrap_or(0);
        let context_used_rate = (context_used_token as f64 / context_limit as f64) * 100.0;
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

        Some(SegmentData {
            primary: format!("{} · {} tokens", percentage_display, tokens_display),
            secondary: String::new(),
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

    message
        .usage
        .clone()
        .map(|usage| usage.normalize().display_tokens())
}

fn format_tokens(tokens: u32) -> String {
    if tokens >= 1000 {
        let k_value = tokens as f64 / 1000.0;
        if k_value.fract() == 0.0 {
            format!("{}k", k_value as u32)
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
    use std::time::{SystemTime, UNIX_EPOCH};

    fn write_temp_transcript(contents: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("best-claude-hud-{nanos}.jsonl"));
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
    fn missing_transcript_does_not_use_project_history() {
        let missing = std::env::temp_dir().join("best-claude-hud-missing.jsonl");
        assert_eq!(parse_transcript_usage(missing), None);
    }
}
