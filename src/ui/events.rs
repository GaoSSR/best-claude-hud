// Event handling utilities

use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    Quit,
    Save,
    MoveUp,
    MoveDown,
    Edit,
    Toggle,
    SwitchPanel,
    OpenColorPicker,
    OpenIconSelector,
    Unknown,
}

pub fn handle_key_event(key: KeyEvent) -> AppEvent {
    match key.code {
        KeyCode::Char('q') => AppEvent::Quit,
        KeyCode::Char('s') => AppEvent::Save,
        KeyCode::Up => AppEvent::MoveUp,
        KeyCode::Down => AppEvent::MoveDown,
        KeyCode::Enter => AppEvent::Edit,
        KeyCode::Char(' ') => AppEvent::Toggle,
        KeyCode::Tab => AppEvent::SwitchPanel,
        KeyCode::Char('c') => AppEvent::OpenColorPicker,
        KeyCode::Char('i') => AppEvent::OpenIconSelector,
        _ => AppEvent::Unknown,
    }
}
