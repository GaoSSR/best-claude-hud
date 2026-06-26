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
        // Handle both Unix and Windows separators by trying both
        let unix_name = path.split('/').next_back().unwrap_or("");
        let windows_name = path.split('\\').next_back().unwrap_or("");

        // Choose the name that indicates actual path splitting occurred
        let result = if windows_name.len() < path.len() {
            // Windows path separator was found
            windows_name
        } else if unix_name.len() < path.len() {
            // Unix path separator was found
            unix_name
        } else {
            // No separator found, use the whole path
            path
        };

        if result.is_empty() {
            "root".to_string()
        } else {
            result.to_string()
        }
    }
}

impl Segment for DirectorySegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        let current_dir = &input.workspace.current_dir;

        // Handle cross-platform path separators manually for better compatibility
        let dir_name = Self::extract_directory_name(current_dir);

        // Store the full path in metadata for potential use
        let mut metadata = HashMap::new();
        metadata.insert("full_path".to_string(), current_dir.clone());

        Some(SegmentData {
            primary: dir_name,
            secondary: String::new(),
            metadata,
        })
    }

    fn id(&self) -> SegmentId {
        SegmentId::Directory
    }
}
