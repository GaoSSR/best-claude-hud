// Configuration editor component

use crate::config::SegmentId;

pub struct EditorComponent {
    pub editing_segment: Option<SegmentId>,
}

impl Default for EditorComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorComponent {
    pub fn new() -> Self {
        Self {
            editing_segment: None,
        }
    }

    pub fn edit_segment(&mut self, segment_id: SegmentId) {
        self.editing_segment = Some(segment_id);
    }

    pub fn stop_editing(&mut self) {
        self.editing_segment = None;
    }

    pub fn is_editing(&self, segment_id: SegmentId) -> bool {
        self.editing_segment == Some(segment_id)
    }
}
