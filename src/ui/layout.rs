// Layout utilities for TUI

use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout;

impl AppLayout {
    /// Create the main layout with title, preview, style selector, main content, and help
    pub fn main_layout(area: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Preview
                Constraint::Length(3), // Style selector
                Constraint::Min(10),   // Main content
                Constraint::Length(3), // Help
            ])
            .split(area)
            .to_vec()
    }

    /// Create the horizontal split for segment list and settings panel
    pub fn content_layout(area: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Segment list
                Constraint::Percentage(70), // Settings panel
            ])
            .split(area)
            .to_vec()
    }
}
