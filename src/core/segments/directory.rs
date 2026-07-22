use super::{Segment, SegmentData};
use crate::config::{InputData, SegmentId};
use std::collections::HashMap;

#[derive(Default)]
pub struct DirectorySegment;

impl DirectorySegment {
    pub fn new() -> Self {
        Self
    }

    /// Extract directory name from path, handling both Unix and Windows separators
    fn extract_directory_name(path: &str) -> String {
        let trimmed = path.trim_end_matches(['/', '\\']);
        if trimmed.is_empty() {
            "root".to_string()
        } else {
            trimmed
                .rsplit(['/', '\\'])
                .next()
                .unwrap_or(trimmed)
                .to_string()
        }
    }
}

impl Segment for DirectorySegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        let project_dir = input.workspace.project_directory();

        // Handle cross-platform path separators manually for better compatibility
        let dir_name = Self::extract_directory_name(project_dir);

        // Store the full path in metadata for potential use
        let mut metadata = HashMap::new();
        metadata.insert("full_path".to_string(), project_dir.to_string());

        Some(SegmentData {
            primary: dir_name,
            secondary: String::new(),
            secondary_color: None,
            metadata,
        })
    }

    fn id(&self) -> SegmentId {
        SegmentId::Directory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn input(current_dir: &str, project_dir: Option<&str>) -> InputData {
        let mut workspace = json!({ "current_dir": current_dir });
        if let Some(project_dir) = project_dir {
            workspace["project_dir"] = json!(project_dir);
        }

        serde_json::from_value(json!({
            "model": "test-model",
            "workspace": workspace,
            "transcript_path": "/tmp/missing.jsonl"
        }))
        .expect("statusline input should deserialize")
    }

    #[test]
    fn project_directory_stays_stable_when_current_directory_changes() {
        let segment = DirectorySegment::new();
        let project_dir = "/Users/gaossr/Coding-Project/Python-Project/Kimi-Test";
        let project_input = input(project_dir, Some(project_dir));
        let skill_input = input(
            "/Users/gaossr/Coding-Project/Python-Project/Kimi-Test/.claude/skills",
            Some(project_dir),
        );

        assert_eq!(
            segment.collect(&project_input).unwrap().primary,
            "Kimi-Test"
        );
        assert_eq!(segment.collect(&skill_input).unwrap().primary, "Kimi-Test");
    }

    #[test]
    fn missing_or_empty_project_directory_falls_back_to_current_directory() {
        let segment = DirectorySegment::new();
        let legacy_input = input("/tmp/legacy-project", None);
        let empty_project_input = input("/tmp/current-project", Some("  "));

        assert_eq!(
            segment.collect(&legacy_input).unwrap().primary,
            "legacy-project"
        );
        assert_eq!(
            segment.collect(&empty_project_input).unwrap().primary,
            "current-project"
        );
    }

    #[test]
    fn extracts_unix_and_windows_directory_names() {
        assert_eq!(
            DirectorySegment::extract_directory_name("/tmp/unix-project"),
            "unix-project"
        );
        assert_eq!(
            DirectorySegment::extract_directory_name(r"C:\Users\test\windows-project"),
            "windows-project"
        );
        assert_eq!(
            DirectorySegment::extract_directory_name("/tmp/unix-project/"),
            "unix-project"
        );
        assert_eq!(
            DirectorySegment::extract_directory_name(r"C:\Users\test\windows-project\"),
            "windows-project"
        );
        assert_eq!(
            DirectorySegment::extract_directory_name(r"C:\Users/test/mixed-project"),
            "mixed-project"
        );
    }
}
