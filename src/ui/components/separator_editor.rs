use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

#[derive(Debug, Clone)]
pub struct SeparatorEditorComponent {
    pub is_open: bool,
    pub input: String,
    pub presets: Vec<SeparatorPreset>,
    pub selected_preset: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct SeparatorPreset {
    pub name: String,
    pub value: String,
    pub description: String,
}

impl Default for SeparatorEditorComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl SeparatorEditorComponent {
    pub fn new() -> Self {
        Self {
            is_open: false,
            input: String::new(),
            presets: Self::default_presets(),
            selected_preset: None,
        }
    }

    fn default_presets() -> Vec<SeparatorPreset> {
        vec![
            SeparatorPreset {
                name: "Pipe".to_string(),
                value: " | ".to_string(),
                description: "Classic pipe separator".to_string(),
            },
            SeparatorPreset {
                name: "Thin".to_string(),
                value: " │ ".to_string(),
                description: "Thin vertical line".to_string(),
            },
            SeparatorPreset {
                name: "Arrow".to_string(),
                value: "\u{e0b0}".to_string(),
                description: "Powerline arrow (seamless transition)".to_string(),
            },
            SeparatorPreset {
                name: "Space".to_string(),
                value: "  ".to_string(),
                description: "Double space".to_string(),
            },
            SeparatorPreset {
                name: "Dot".to_string(),
                value: " • ".to_string(),
                description: "Middle dot".to_string(),
            },
        ]
    }

    pub fn open(&mut self, current_separator: &str) {
        self.is_open = true;
        self.input = current_separator.to_string();
        self.selected_preset = None;

        // Check if current separator matches a preset
        for (i, preset) in self.presets.iter().enumerate() {
            if preset.value == current_separator {
                self.selected_preset = Some(i);
                break;
            }
        }
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.input.clear();
        self.selected_preset = None;
    }

    pub fn input_char(&mut self, c: char) {
        // Allow most characters for separator
        if !c.is_control() {
            self.input.push(c);
            self.selected_preset = None; // Clear preset selection when manually editing
        }
    }

    pub fn backspace(&mut self) {
        self.input.pop();
        self.selected_preset = None; // Clear preset selection when manually editing
    }

    pub fn move_preset_selection(&mut self, delta: i32) {
        let new_selection = if let Some(current) = self.selected_preset {
            let new_idx = (current as i32 + delta).clamp(0, self.presets.len() as i32 - 1) as usize;
            Some(new_idx)
        } else if delta > 0 {
            Some(0)
        } else {
            Some(self.presets.len() - 1)
        };

        self.selected_preset = new_selection;
        if let Some(idx) = new_selection {
            self.input = self.presets[idx].value.clone();
        }
    }

    pub fn get_separator(&self) -> String {
        self.input.clone()
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.is_open {
            return;
        }

        // Calculate exact size needed
        let popup_height = 15;
        let popup_width = 60;
        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        // Clear the popup area first
        f.render_widget(Clear, popup_area);

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .title("Separator Editor");
        let inner = popup_block.inner(popup_area);
        f.render_widget(popup_block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Current input
                Constraint::Min(5),    // Presets list
                Constraint::Length(3), // Actions
            ])
            .split(inner);

        // Current input field
        f.render_widget(
            Paragraph::new(format!("> {} <", self.input))
                .style(Style::default().fg(Color::Yellow))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Current Separator"),
                ),
            chunks[0],
        );

        // Presets list
        let preset_text = self
            .presets
            .iter()
            .enumerate()
            .map(|(i, preset)| {
                let marker = if Some(i) == self.selected_preset {
                    "[•]"
                } else {
                    "[ ]"
                };
                format!("{} {} - {}", marker, preset.name, preset.description)
            })
            .collect::<Vec<_>>()
            .join("\n");

        f.render_widget(
            Paragraph::new(preset_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Presets (↑↓ to select)"),
            ),
            chunks[1],
        );

        // Actions
        f.render_widget(
            Paragraph::new("[Enter] Confirm  [Esc] Cancel  [Tab] Clear")
                .block(Block::default().borders(Borders::ALL)),
            chunks[2],
        );
    }
}
