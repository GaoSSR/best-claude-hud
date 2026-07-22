use crate::config::InputData;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{get_current_pid, ProcessesToUpdate, System};

const CHECKPOINT_SIZE: usize = 64;
const MAX_ANCESTOR_DEPTH: usize = 8;
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum EffortSelection {
    Ultracode,
    Other,
}

#[derive(Debug, Deserialize)]
struct SessionRecord {
    #[serde(rename = "sessionId")]
    session_id: String,
    #[serde(rename = "startedAt")]
    started_at: u64,
    #[serde(default)]
    cwd: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct EffortCache {
    transcript_path: String,
    started_at: u64,
    processed_bytes: u64,
    selection: Option<EffortSelection>,
    pending_prompt_id: Option<String>,
    prefix_checkpoint: Vec<u8>,
    checkpoint: Vec<u8>,
    processed_hash: u64,
    created_nanos: Option<u64>,
    modified_nanos: Option<u64>,
    changed_nanos: Option<u64>,
}

impl EffortCache {
    fn new(transcript_path: &Path, started_at: u64) -> Self {
        Self {
            transcript_path: transcript_path.to_string_lossy().into_owned(),
            started_at,
            processed_bytes: 0,
            selection: None,
            pending_prompt_id: None,
            prefix_checkpoint: Vec::new(),
            checkpoint: Vec::new(),
            processed_hash: FNV_OFFSET_BASIS,
            created_nanos: None,
            modified_nanos: None,
            changed_nanos: None,
        }
    }

    fn matches(&self, transcript_path: &Path, started_at: u64, metadata: &fs::Metadata) -> bool {
        let file_len = metadata.len();
        let created_nanos = system_time_nanos(metadata.created().ok());
        let modified_nanos = system_time_nanos(metadata.modified().ok());
        let changed_nanos = metadata_changed_nanos(metadata);
        let metadata_proves_unchanged = self.modified_nanos.is_some()
            && self.changed_nanos.is_some()
            && self.modified_nanos == modified_nanos
            && self.changed_nanos == changed_nanos;

        self.transcript_path == transcript_path.to_string_lossy()
            && self.started_at == started_at
            && self.processed_bytes <= file_len
            && (self.created_nanos.is_none()
                || created_nanos.is_none()
                || self.created_nanos == created_nanos)
            && read_prefix_checkpoint(transcript_path, self.prefix_checkpoint.len()).as_deref()
                == Some(self.prefix_checkpoint.as_slice())
            && read_checkpoint(transcript_path, self.processed_bytes, self.checkpoint.len())
                .as_deref()
                == Some(self.checkpoint.as_slice())
            && (self.processed_bytes != file_len
                || metadata_proves_unchanged
                || hash_prefix(transcript_path, self.processed_bytes) == Some(self.processed_hash))
    }
}

enum TranscriptEvent {
    EffortCommand { prompt_id: String },
    CommandOutput { prompt_id: String, content: String },
    Other,
}

pub fn resolve_effort_label(input: &InputData) -> Option<String> {
    let level = input
        .effort
        .as_ref()
        .and_then(|effort| effort.level.as_deref())
        .map(str::trim)
        .filter(|level| !level.is_empty())?;

    let canonical_level = level.to_ascii_lowercase();
    if canonical_level != "xhigh" {
        return Some(match canonical_level.as_str() {
            "low" | "medium" | "high" | "max" | "ultracode" => canonical_level,
            _ => level.to_string(),
        });
    }

    let environment_override = std::env::var("CLAUDE_CODE_EFFORT_LEVEL")
        .ok()
        .filter(|value| !value.trim().is_empty());
    let session_started_at = input.session_id.as_deref().and_then(|session_id| {
        active_session_started_at(session_id, input.workspace.project_directory())
    });

    Some(resolve_xhigh_label(
        Path::new(&input.transcript_path),
        input.session_id.as_deref(),
        environment_override.as_deref(),
        session_started_at,
    ))
}

fn resolve_xhigh_label(
    transcript_path: &Path,
    session_id: Option<&str>,
    environment_override: Option<&str>,
    session_started_at: Option<u64>,
) -> String {
    if environment_override
        .map(str::trim)
        .is_some_and(|value| !value.eq_ignore_ascii_case("xhigh"))
    {
        return "xhigh".to_string();
    }

    let (Some(session_id), Some(started_at)) = (session_id, session_started_at) else {
        return "xhigh".to_string();
    };
    let Some(cache_path) = effort_cache_path(session_id, started_at) else {
        return "xhigh".to_string();
    };

    match refresh_effort_cache(transcript_path, started_at, &cache_path) {
        Some(EffortSelection::Ultracode) => "ultracode".to_string(),
        _ => "xhigh".to_string(),
    }
}

fn active_session_started_at(session_id: &str, project_directory: &str) -> Option<u64> {
    let sessions_dir = dirs::home_dir()?.join(".claude").join("sessions");
    active_ancestor_session_started_at(&sessions_dir, session_id, project_directory)
}

fn active_ancestor_session_started_at(
    sessions_dir: &Path,
    session_id: &str,
    project_directory: &str,
) -> Option<u64> {
    let mut system = System::new();
    let mut pid = get_current_pid().ok()?;

    for _ in 0..MAX_ANCESTOR_DEPTH {
        let session_path = sessions_dir.join(format!("{pid}.json"));
        if let Ok(contents) = fs::read(session_path) {
            if let Ok(record) = serde_json::from_slice::<SessionRecord>(&contents) {
                if record.session_id == session_id
                    && record.cwd.as_deref() == Some(project_directory)
                {
                    return Some(record.started_at);
                }
            }
        }

        let pids = [pid];
        system.refresh_processes(ProcessesToUpdate::Some(&pids), false);
        pid = system.process(pid)?.parent()?;
    }

    None
}

fn effort_cache_path(session_id: &str, started_at: u64) -> Option<PathBuf> {
    let safe_session_id: String = session_id
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .collect();
    if safe_session_id.is_empty() {
        return None;
    }

    Some(
        dirs::cache_dir()?
            .join("best-claude-hud")
            .join(format!("effort-{safe_session_id}-{started_at}.json")),
    )
}

fn refresh_effort_cache(
    transcript_path: &Path,
    started_at: u64,
    cache_path: &Path,
) -> Option<EffortSelection> {
    refresh_effort_cache_with_stats(transcript_path, started_at, cache_path).0
}

fn refresh_effort_cache_with_stats(
    transcript_path: &Path,
    started_at: u64,
    cache_path: &Path,
) -> (Option<EffortSelection>, usize) {
    let Ok(metadata) = fs::metadata(transcript_path) else {
        return (None, 0);
    };

    let loaded_cache = read_effort_cache(cache_path)
        .filter(|cache| cache.matches(transcript_path, started_at, &metadata));
    let had_valid_cache = loaded_cache.is_some();
    let mut cache = loaded_cache.unwrap_or_else(|| EffortCache::new(transcript_path, started_at));
    let previous_cache = cache.clone();

    let bytes_read = scan_appended_transcript(transcript_path, &mut cache);
    cache.prefix_checkpoint = read_prefix_checkpoint(
        transcript_path,
        CHECKPOINT_SIZE.min(cache.processed_bytes as usize),
    )
    .unwrap_or_default();
    cache.checkpoint = read_checkpoint(
        transcript_path,
        cache.processed_bytes,
        CHECKPOINT_SIZE.min(cache.processed_bytes as usize),
    )
    .unwrap_or_default();
    if let Ok(metadata) = fs::metadata(transcript_path) {
        cache.created_nanos = system_time_nanos(metadata.created().ok());
        cache.modified_nanos = system_time_nanos(metadata.modified().ok());
        cache.changed_nanos = metadata_changed_nanos(&metadata);
    }
    if !had_valid_cache || cache != previous_cache {
        write_effort_cache(cache_path, &cache);
    }
    (cache.selection, bytes_read)
}

fn system_time_nanos(value: Option<SystemTime>) -> Option<u64> {
    value?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_nanos().min(u64::MAX as u128) as u64)
}

#[cfg(unix)]
fn metadata_changed_nanos(metadata: &fs::Metadata) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;

    let seconds = metadata.ctime();
    let nanos = metadata.ctime_nsec();
    if seconds < 0 || nanos < 0 {
        return None;
    }

    Some(((seconds as u128) * 1_000_000_000 + nanos as u128).min(u64::MAX as u128) as u64)
}

#[cfg(not(unix))]
fn metadata_changed_nanos(_metadata: &fs::Metadata) -> Option<u64> {
    None
}

fn update_hash(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn hash_prefix(path: &Path, length: u64) -> Option<u64> {
    let mut file = File::open(path).ok()?.take(length);
    let mut buffer = [0_u8; 8192];
    let mut hash = FNV_OFFSET_BASIS;

    loop {
        let read = file.read(&mut buffer).ok()?;
        if read == 0 {
            return Some(hash);
        }
        hash = update_hash(hash, &buffer[..read]);
    }
}

fn read_prefix_checkpoint(path: &Path, checkpoint_len: usize) -> Option<Vec<u8>> {
    let mut file = File::open(path).ok()?;
    let mut checkpoint = vec![0; checkpoint_len];
    file.read_exact(&mut checkpoint).ok()?;
    Some(checkpoint)
}

fn read_checkpoint(path: &Path, processed_bytes: u64, checkpoint_len: usize) -> Option<Vec<u8>> {
    let checkpoint_len = checkpoint_len.min(processed_bytes as usize);
    let mut file = File::open(path).ok()?;
    file.seek(SeekFrom::Start(processed_bytes - checkpoint_len as u64))
        .ok()?;
    let mut checkpoint = vec![0; checkpoint_len];
    file.read_exact(&mut checkpoint).ok()?;
    Some(checkpoint)
}

fn read_effort_cache(path: &Path) -> Option<EffortCache> {
    let contents = fs::read(path).ok()?;
    serde_json::from_slice(&contents).ok()
}

fn write_effort_cache(path: &Path, cache: &EffortCache) {
    let Some(parent) = path.parent() else {
        return;
    };
    if fs::create_dir_all(parent).is_err() {
        return;
    }
    let Ok(contents) = serde_json::to_vec(cache) else {
        return;
    };
    let temporary_path = path.with_extension(format!("{}.tmp", std::process::id()));
    if fs::write(&temporary_path, contents).is_ok() {
        #[cfg(windows)]
        let _ = fs::remove_file(path);
        let _ = fs::rename(&temporary_path, path);
    }
}

fn scan_appended_transcript(path: &Path, cache: &mut EffortCache) -> usize {
    let Ok(mut file) = File::open(path) else {
        return 0;
    };
    if file.seek(SeekFrom::Start(cache.processed_bytes)).is_err() {
        return 0;
    }

    let mut appended = Vec::new();
    if file.read_to_end(&mut appended).is_err() {
        return 0;
    }
    let complete_len = appended
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map(|position| position + 1)
        .unwrap_or(0);

    for line in appended[..complete_len].split(|byte| *byte == b'\n') {
        if line.is_empty() {
            continue;
        }
        apply_transcript_event(parse_transcript_event(line, cache.started_at), cache);
    }

    cache.processed_hash = update_hash(cache.processed_hash, &appended[..complete_len]);
    cache.processed_bytes += complete_len as u64;
    appended.len()
}

fn parse_transcript_event(line: &[u8], started_at_ms: u64) -> TranscriptEvent {
    let Ok(entry) = serde_json::from_slice::<Value>(line) else {
        return TranscriptEvent::Other;
    };
    let Some(timestamp) = entry.get("timestamp").and_then(Value::as_str) else {
        return TranscriptEvent::Other;
    };
    let Ok(timestamp) = DateTime::parse_from_rfc3339(timestamp) else {
        return TranscriptEvent::Other;
    };
    let timestamp_ms = timestamp.timestamp_millis();
    if timestamp_ms < 0 || (timestamp_ms as u64) < started_at_ms {
        return TranscriptEvent::Other;
    }
    if entry.get("type").and_then(Value::as_str) != Some("user") {
        return TranscriptEvent::Other;
    }

    let Some(prompt_id) = entry.get("promptId").and_then(Value::as_str) else {
        return TranscriptEvent::Other;
    };
    let Some(content) = entry.pointer("/message/content").and_then(Value::as_str) else {
        return TranscriptEvent::Other;
    };

    if content.starts_with("<local-command-stdout>") && content.ends_with("</local-command-stdout>")
    {
        return TranscriptEvent::CommandOutput {
            prompt_id: prompt_id.to_string(),
            content: content.to_string(),
        };
    }
    if is_effort_command(content) {
        return TranscriptEvent::EffortCommand {
            prompt_id: prompt_id.to_string(),
        };
    }

    TranscriptEvent::Other
}

fn is_effort_command(content: &str) -> bool {
    let mut lines = content.lines().map(str::trim);
    if lines.next() != Some("<command-name>/effort</command-name>")
        || lines.next() != Some("<command-message>effort</command-message>")
    {
        return false;
    }

    let Some(arguments) = lines.next() else {
        return false;
    };
    arguments.starts_with("<command-args>")
        && arguments.ends_with("</command-args>")
        && lines.all(|line| line.trim().is_empty())
}

fn apply_transcript_event(event: TranscriptEvent, cache: &mut EffortCache) {
    match event {
        TranscriptEvent::EffortCommand { prompt_id } => {
            cache.pending_prompt_id = Some(prompt_id);
        }
        TranscriptEvent::CommandOutput { prompt_id, content }
            if cache.pending_prompt_id.as_deref() == Some(prompt_id.as_str()) =>
        {
            if let Some(selection) = classify_effort_output(&content) {
                cache.selection = Some(selection);
            }
            cache.pending_prompt_id = None;
        }
        TranscriptEvent::CommandOutput { .. } | TranscriptEvent::Other => {}
    }
}

fn classify_effort_output(content: &str) -> Option<EffortSelection> {
    let stdout = content
        .strip_prefix("<local-command-stdout>")?
        .strip_suffix("</local-command-stdout>")?
        .trim();

    if stdout.starts_with("Invalid argument:")
        || stdout.contains("exceeds your organization's limit")
        || stdout.starts_with("Higher effort levels are restricted by your organization")
        || stdout.contains("overrides effort this session")
        || stdout.starts_with("Ultracode needs dynamic workflows enabled")
    {
        return Some(EffortSelection::Other);
    }

    if stdout.starts_with("Set effort level to ultracode")
        || stdout.starts_with("Current effort level: ultracode")
    {
        return Some(EffortSelection::Ultracode);
    }

    if stdout.starts_with("Set effort level to ")
        || stdout.starts_with("Current effort level: ")
        || stdout.starts_with("Effort level set to auto")
    {
        return Some(EffortSelection::Other);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_path(suffix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "best-claude-hud-effort-{}-{nanos}-{sequence}-{suffix}",
            std::process::id()
        ))
    }

    fn timestamp(value: &str) -> u64 {
        DateTime::parse_from_rfc3339(value)
            .unwrap()
            .timestamp_millis() as u64
    }

    fn command(prompt_id: &str, timestamp: &str, level: &str) -> String {
        format!(
            r#"{{"type":"user","timestamp":"{timestamp}","promptId":"{prompt_id}","message":{{"content":"<command-name>/effort</command-name>\n            <command-message>effort</command-message>\n            <command-args>{level}</command-args>"}}}}
"#
        )
    }

    fn output(prompt_id: &str, timestamp: &str, content: &str) -> String {
        format!(
            r#"{{"type":"user","timestamp":"{timestamp}","promptId":"{prompt_id}","message":{{"content":"<local-command-stdout>{content}</local-command-stdout>"}}}}
"#
        )
    }

    fn write_transcript(contents: &str) -> PathBuf {
        let path = temp_path("transcript.jsonl");
        fs::write(&path, contents).unwrap();
        path
    }

    fn resolve_with_cache(transcript_path: &Path, cache_path: &Path, started_at: u64) -> String {
        match refresh_effort_cache(transcript_path, started_at, cache_path) {
            Some(EffortSelection::Ultracode) => "ultracode".to_string(),
            _ => "xhigh".to_string(),
        }
    }

    #[test]
    fn distinguishes_ultracode_from_regular_xhigh() {
        let contents = format!(
            "{}{}{}{}",
            command("xhigh", "2026-07-22T11:31:00Z", "xhigh"),
            output(
                "xhigh",
                "2026-07-22T11:31:00Z",
                "Set effort level to xhigh: Deeper reasoning"
            ),
            command("ultra", "2026-07-22T11:32:00Z", "ultracode"),
            output(
                "ultra",
                "2026-07-22T11:32:00Z",
                "Set effort level to ultracode (this session only): xhigh + dynamic workflow orchestration"
            )
        );
        let transcript = write_transcript(&contents);
        let cache = temp_path("cache.json");

        assert_eq!(
            resolve_with_cache(&transcript, &cache, timestamp("2026-07-22T11:30:00Z")),
            "ultracode"
        );

        fs::remove_file(transcript).unwrap();
        fs::remove_file(cache).unwrap();
    }

    #[test]
    fn latest_successful_effort_change_wins() {
        let contents = format!(
            "{}{}{}{}",
            command("ultra", "2026-07-22T11:31:00Z", "ultracode"),
            output(
                "ultra",
                "2026-07-22T11:31:00Z",
                "Set effort level to ultracode (this session only): xhigh + dynamic workflow orchestration"
            ),
            command("xhigh", "2026-07-22T11:32:00Z", "xhigh"),
            output(
                "xhigh",
                "2026-07-22T11:32:00Z",
                "Set effort level to xhigh: Deeper reasoning"
            )
        );
        let transcript = write_transcript(&contents);
        let cache = temp_path("cache.json");

        assert_eq!(
            resolve_with_cache(&transcript, &cache, timestamp("2026-07-22T11:30:00Z")),
            "xhigh"
        );

        fs::remove_file(transcript).unwrap();
        fs::remove_file(cache).unwrap();
    }

    #[test]
    fn resumed_session_does_not_inherit_old_ultracode_selection() {
        let contents = format!(
            "{}{}",
            command("ultra", "2026-07-22T11:31:00Z", "ultracode"),
            output(
                "ultra",
                "2026-07-22T11:31:00Z",
                "Set effort level to ultracode (this session only): xhigh + dynamic workflow orchestration"
            )
        );
        let transcript = write_transcript(&contents);
        let cache = temp_path("cache.json");

        assert_eq!(
            resolve_with_cache(&transcript, &cache, timestamp("2026-07-22T11:40:00Z")),
            "xhigh"
        );

        fs::remove_file(transcript).unwrap();
        fs::remove_file(cache).unwrap();
    }

    #[test]
    fn arbitrary_stdout_cannot_spoof_ultracode() {
        let transcript = write_transcript(&output(
            "shell",
            "2026-07-22T11:32:00Z",
            "Set effort level to ultracode (this session only): xhigh + dynamic workflow orchestration",
        ));
        let cache = temp_path("cache.json");

        assert_eq!(resolve_with_cache(&transcript, &cache, 0), "xhigh");

        fs::remove_file(transcript).unwrap();
        fs::remove_file(cache).unwrap();
    }

    #[test]
    fn stdout_with_embedded_effort_tag_cannot_create_pending_command_state() {
        let nested_stdout = output(
            "spoof",
            "2026-07-22T11:31:00Z",
            "<command-name>/effort</command-name>",
        );
        let contents = format!(
            "{}{}",
            nested_stdout,
            output(
                "spoof",
                "2026-07-22T11:32:00Z",
                "Set effort level to ultracode (this session only): xhigh + dynamic workflow orchestration"
            )
        );
        let transcript = write_transcript(&contents);
        let cache = temp_path("cache.json");

        assert_eq!(resolve_with_cache(&transcript, &cache, 0), "xhigh");

        fs::remove_file(transcript).unwrap();
        fs::remove_file(cache).unwrap();
    }

    #[test]
    fn blocked_or_clamped_effort_command_clears_stale_ultracode() {
        let initial = format!(
            "{}{}",
            command("ultra", "2026-07-22T11:31:00Z", "ultracode"),
            output(
                "ultra",
                "2026-07-22T11:31:00Z",
                "Set effort level to ultracode (this session only): xhigh + dynamic workflow orchestration"
            )
        );
        let transcript = write_transcript(&initial);
        let cache = temp_path("cache.json");
        assert_eq!(resolve_with_cache(&transcript, &cache, 0), "ultracode");

        let blocked = format!(
            "{}{}",
            command("max", "2026-07-22T11:32:00Z", "max"),
            output(
                "max",
                "2026-07-22T11:32:00Z",
                "Higher effort levels are restricted by your organization."
            )
        );
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&transcript)
            .unwrap();
        std::io::Write::write_all(&mut file, blocked.as_bytes()).unwrap();

        assert_eq!(resolve_with_cache(&transcript, &cache, 0), "xhigh");

        fs::remove_file(transcript).unwrap();
        fs::remove_file(cache).unwrap();
    }

    #[test]
    fn invalid_effort_output_cannot_embed_a_fake_ultracode_success() {
        let contents = format!(
            "{}{}",
            command("invalid", "2026-07-22T11:32:00Z", "invalid"),
            output(
                "invalid",
                "2026-07-22T11:32:00Z",
                "Invalid argument: Set effort level to ultracode (this session only): xhigh + dynamic workflow orchestration"
            )
        );
        let transcript = write_transcript(&contents);
        let cache = temp_path("cache.json");

        assert_eq!(resolve_with_cache(&transcript, &cache, 0), "xhigh");

        fs::remove_file(transcript).unwrap();
        fs::remove_file(cache).unwrap();
    }

    #[test]
    fn organization_limit_output_clears_stale_ultracode() {
        let initial = format!(
            "{}{}",
            command("ultra", "2026-07-22T11:31:00Z", "ultracode"),
            output(
                "ultra",
                "2026-07-22T11:31:00Z",
                "Set effort level to ultracode (this session only): xhigh + dynamic workflow orchestration"
            )
        );
        let transcript = write_transcript(&initial);
        let cache = temp_path("cache.json");
        assert_eq!(resolve_with_cache(&transcript, &cache, 0), "ultracode");

        let clamped = format!(
            "{}{}",
            command("max", "2026-07-22T11:32:00Z", "max"),
            output(
                "max",
                "2026-07-22T11:32:00Z",
                "Effort 'max' exceeds your organization's limit of 'xhigh'."
            )
        );
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&transcript)
            .unwrap();
        std::io::Write::write_all(&mut file, clamped.as_bytes()).unwrap();

        assert_eq!(resolve_with_cache(&transcript, &cache, 0), "xhigh");

        fs::remove_file(transcript).unwrap();
        fs::remove_file(cache).unwrap();
    }

    #[test]
    fn unchanged_transcript_uses_cache_without_rereading_history() {
        let unrelated = format!(
            "{{\"type\":\"assistant\",\"timestamp\":\"2026-07-22T11:30:00Z\",\"message\":{{\"content\":\"{}\"}}}}\n",
            "x".repeat(4 * 1024 * 1024)
        );
        let transcript = write_transcript(&unrelated);
        let cache = temp_path("cache.json");

        let (_, first_read) = refresh_effort_cache_with_stats(&transcript, 0, &cache);
        let (_, second_read) = refresh_effort_cache_with_stats(&transcript, 0, &cache);

        assert!(first_read >= 4 * 1024 * 1024);
        assert_eq!(second_read, 0);

        fs::remove_file(transcript).unwrap();
        fs::remove_file(cache).unwrap();
    }

    #[test]
    fn incrementally_reads_only_new_effort_events() {
        let transcript = write_transcript("{}\n");
        let cache = temp_path("cache.json");
        let (_, first_read) = refresh_effort_cache_with_stats(&transcript, 0, &cache);
        assert_eq!(first_read, 3);

        let appended = format!(
            "{}{}",
            command("ultra", "2026-07-22T11:32:00Z", "ultracode"),
            output(
                "ultra",
                "2026-07-22T11:32:00Z",
                "Set effort level to ultracode (this session only): xhigh + dynamic workflow orchestration"
            )
        );
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&transcript)
            .unwrap();
        std::io::Write::write_all(&mut file, appended.as_bytes()).unwrap();

        let (selection, second_read) = refresh_effort_cache_with_stats(&transcript, 0, &cache);
        assert_eq!(selection, Some(EffortSelection::Ultracode));
        assert_eq!(second_read, appended.len());

        fs::remove_file(transcript).unwrap();
        fs::remove_file(cache).unwrap();
    }

    #[test]
    fn truncated_transcript_invalidates_stale_ultracode_cache() {
        let contents = format!(
            "{}{}",
            command("ultra", "2026-07-22T11:31:00Z", "ultracode"),
            output(
                "ultra",
                "2026-07-22T11:31:00Z",
                "Set effort level to ultracode (this session only): xhigh + dynamic workflow orchestration"
            )
        );
        let transcript = write_transcript(&contents);
        let cache = temp_path("cache.json");
        assert_eq!(resolve_with_cache(&transcript, &cache, 0), "ultracode");

        fs::write(&transcript, "{}\n").unwrap();
        assert_eq!(resolve_with_cache(&transcript, &cache, 0), "xhigh");

        fs::remove_file(transcript).unwrap();
        fs::remove_file(cache).unwrap();
    }

    #[test]
    fn same_length_transcript_replacement_invalidates_stale_cache() {
        let prefix = format!(
            "{{\"type\":\"assistant\",\"timestamp\":\"2026-07-22T11:30:00Z\",\"message\":{{\"content\":\"{}\"}}}}\n",
            "p".repeat(256)
        );
        let suffix = format!(
            "{{\"type\":\"assistant\",\"timestamp\":\"2026-07-22T11:33:00Z\",\"message\":{{\"content\":\"{}\"}}}}\n",
            "s".repeat(256)
        );
        let contents = format!(
            "{}{}{}{}",
            prefix,
            command("ultra", "2026-07-22T11:31:00Z", "ultracode"),
            output(
                "ultra",
                "2026-07-22T11:31:00Z",
                "Set effort level to ultracode (this session only): xhigh + dynamic workflow orchestration"
            ),
            suffix
        );
        let transcript = write_transcript(&contents);
        let cache = temp_path("cache.json");
        assert_eq!(resolve_with_cache(&transcript, &cache, 0), "ultracode");

        let mut replacement = contents.into_bytes();
        let replace_start = CHECKPOINT_SIZE;
        let replace_end = replacement.len() - CHECKPOINT_SIZE;
        replacement[replace_start..replace_end].fill(b' ');
        std::thread::sleep(std::time::Duration::from_millis(5));
        fs::write(&transcript, replacement).unwrap();
        let mut cached = read_effort_cache(&cache).unwrap();
        let metadata = fs::metadata(&transcript).unwrap();
        cached.modified_nanos = system_time_nanos(metadata.modified().ok());
        write_effort_cache(&cache, &cached);

        assert_eq!(resolve_with_cache(&transcript, &cache, 0), "xhigh");

        fs::remove_file(transcript).unwrap();
        fs::remove_file(cache).unwrap();
    }

    #[test]
    fn incomplete_trailing_line_is_processed_after_completion() {
        let complete_command = command("ultra", "2026-07-22T11:31:00Z", "ultracode");
        let partial_command = complete_command.trim_end_matches('\n');
        let transcript = write_transcript(partial_command);
        let cache = temp_path("cache.json");
        assert_eq!(resolve_with_cache(&transcript, &cache, 0), "xhigh");

        let appended = format!(
            "\n{}",
            output(
                "ultra",
                "2026-07-22T11:31:00Z",
                "Set effort level to ultracode (this session only): xhigh + dynamic workflow orchestration"
            )
        );
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&transcript)
            .unwrap();
        std::io::Write::write_all(&mut file, appended.as_bytes()).unwrap();

        assert_eq!(resolve_with_cache(&transcript, &cache, 0), "ultracode");

        fs::remove_file(transcript).unwrap();
        fs::remove_file(cache).unwrap();
    }

    #[test]
    fn xhigh_environment_override_allows_verified_ultracode() {
        let contents = format!(
            "{}{}",
            command("ultra", "2026-07-22T11:31:00Z", "ultracode"),
            output(
                "ultra",
                "2026-07-22T11:31:00Z",
                "Set effort level to ultracode (this session only): xhigh + dynamic workflow orchestration"
            )
        );
        let transcript = write_transcript(&contents);
        let session_id = format!(
            "test-xhigh-override-{}",
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        assert_eq!(
            resolve_xhigh_label(&transcript, Some(&session_id), Some("xhigh"), Some(0)),
            "ultracode"
        );
        fs::remove_file(transcript).unwrap();
        if let Some(cache_path) = effort_cache_path(&session_id, 0) {
            if cache_path.exists() {
                fs::remove_file(cache_path).unwrap();
            }
        }
    }

    #[test]
    fn incompatible_environment_override_prevents_ultracode_misreporting() {
        let transcript = write_transcript("");
        assert_eq!(
            resolve_xhigh_label(&transcript, Some("session"), Some("max"), Some(0)),
            "xhigh"
        );
        fs::remove_file(transcript).unwrap();
    }

    #[test]
    fn cache_path_isolated_by_process_start() {
        let first = effort_cache_path("session", 1000).unwrap();
        let second = effort_cache_path("session", 2000).unwrap();
        assert_ne!(first, second);
    }

    #[test]
    fn active_session_boundary_uses_the_current_process_registry_record() {
        let sessions_dir = temp_path("sessions");
        fs::create_dir_all(&sessions_dir).unwrap();
        let pid = get_current_pid().unwrap();
        fs::write(
            sessions_dir.join(format!("{pid}.json")),
            r#"{"sessionId":"current-session","startedAt":1234,"cwd":"/tmp/Kimi-Test"}"#,
        )
        .unwrap();

        assert_eq!(
            active_ancestor_session_started_at(&sessions_dir, "current-session", "/tmp/Kimi-Test"),
            Some(1234)
        );
        assert_eq!(
            active_ancestor_session_started_at(
                &sessions_dir,
                "current-session",
                "/tmp/Other-Project"
            ),
            None
        );

        fs::remove_dir_all(sessions_dir).unwrap();
    }
}
