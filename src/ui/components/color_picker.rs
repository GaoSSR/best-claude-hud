use crate::config::AnsiColor;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{
        Block, Borders, Clear, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Frame,
};

#[derive(Debug, Clone, Copy)]
pub enum NavDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorPickerMode {
    Basic16,
    Extended256,
    RgbInput,
}

#[derive(Debug, Clone)]
pub struct ColorPickerComponent {
    pub is_open: bool,
    pub mode: ColorPickerMode,
    pub selected_basic: usize,
    pub selected_extended: usize,
    pub rgb_input: RgbInput,
    pub current_color: Option<AnsiColor>,
    pub show_extended: bool,
    pub basic_list_state: ListState,
    pub basic_scrollbar_state: ScrollbarState,
    // Cache columns per row for navigation
    pub cached_basic_cols: usize,
    pub cached_extended_cols: usize,
}

#[derive(Debug, Clone)]
pub struct RgbInput {
    pub r: String,
    pub g: String,
    pub b: String,
    pub hex: String,
    pub editing_field: RgbField,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RgbField {
    Red,
    Green,
    Blue,
    Hex,
}

impl Default for ColorPickerComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorPickerComponent {
    pub fn new() -> Self {
        Self {
            is_open: false,
            mode: ColorPickerMode::Basic16,
            selected_basic: 0,
            selected_extended: 0,
            rgb_input: RgbInput {
                r: String::new(),
                g: String::new(),
                b: String::new(),
                hex: String::new(),
                editing_field: RgbField::Red,
            },
            current_color: None,
            show_extended: false,
            basic_list_state: ListState::default().with_selected(Some(0)),
            basic_scrollbar_state: ScrollbarState::new(16),
            cached_basic_cols: 4,
            cached_extended_cols: 16,
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.mode = ColorPickerMode::Basic16;
        self.selected_basic = 0;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn toggle_extended(&mut self) {
        self.show_extended = !self.show_extended;
        if self.show_extended {
            self.mode = ColorPickerMode::Extended256;
        } else {
            self.mode = ColorPickerMode::Basic16;
        }
    }

    pub fn switch_to_rgb(&mut self) {
        self.mode = ColorPickerMode::RgbInput;
    }

    pub fn cycle_mode(&mut self) {
        self.mode = match self.mode {
            ColorPickerMode::Basic16 => ColorPickerMode::Extended256,
            ColorPickerMode::Extended256 => ColorPickerMode::RgbInput,
            ColorPickerMode::RgbInput => ColorPickerMode::Basic16,
        };

        // Update show_extended for compatibility with existing logic
        self.show_extended = matches!(self.mode, ColorPickerMode::Extended256);
    }

    pub fn move_selection(&mut self, delta: i32) {
        match self.mode {
            ColorPickerMode::Basic16 => {
                let new_selection = (self.selected_basic as i32 + delta).clamp(0, 15) as usize;
                self.selected_basic = new_selection;
                self.basic_list_state.select(Some(new_selection));
                self.current_color = Some(AnsiColor::Color16 {
                    c16: new_selection as u8,
                });
            }
            ColorPickerMode::Extended256 => {
                let new_selection = (self.selected_extended as i32 + delta).clamp(0, 255) as usize;
                self.selected_extended = new_selection;
                self.current_color = Some(AnsiColor::Color256 {
                    c256: new_selection as u8,
                });
            }
            ColorPickerMode::RgbInput => {
                // Handle RGB input field navigation
                match self.rgb_input.editing_field {
                    RgbField::Red if delta > 0 => self.rgb_input.editing_field = RgbField::Green,
                    RgbField::Green if delta > 0 => self.rgb_input.editing_field = RgbField::Blue,
                    RgbField::Blue if delta > 0 => self.rgb_input.editing_field = RgbField::Hex,
                    RgbField::Green if delta < 0 => self.rgb_input.editing_field = RgbField::Red,
                    RgbField::Blue if delta < 0 => self.rgb_input.editing_field = RgbField::Green,
                    RgbField::Hex if delta < 0 => self.rgb_input.editing_field = RgbField::Blue,
                    _ => {}
                }
            }
        }
    }

    pub fn move_direction(&mut self, direction: NavDirection) {
        match self.mode {
            ColorPickerMode::Basic16 => {
                // Use cached columns per row for consistent navigation
                let cols_per_row = self.cached_basic_cols;
                let current_row = self.selected_basic / cols_per_row;
                let current_col = self.selected_basic % cols_per_row;

                let new_selection = match direction {
                    NavDirection::Up => {
                        if current_row > 0 {
                            let new_index = (current_row - 1) * cols_per_row + current_col;
                            new_index.min(15)
                        } else {
                            self.selected_basic // 已在顶部，不移动
                        }
                    }
                    NavDirection::Down => {
                        let total_rows = 16_usize.div_ceil(cols_per_row);
                        if current_row + 1 < total_rows {
                            let new_index = (current_row + 1) * cols_per_row + current_col;
                            new_index.min(15)
                        } else {
                            self.selected_basic // 已在底部，不移动
                        }
                    }
                    NavDirection::Left => {
                        if self.selected_basic > 0 {
                            self.selected_basic - 1
                        } else {
                            15 // 循环到最后一个
                        }
                    }
                    NavDirection::Right => {
                        if self.selected_basic < 15 {
                            self.selected_basic + 1
                        } else {
                            0 // 循环到第一个
                        }
                    }
                };

                self.selected_basic = new_selection;
                self.basic_list_state.select(Some(new_selection));
                self.current_color = Some(AnsiColor::Color16 {
                    c16: new_selection as u8,
                });
            }
            ColorPickerMode::Extended256 => {
                // Use cached columns per row for consistent navigation
                let cols_per_row = self.cached_extended_cols;
                let current_row = self.selected_extended / cols_per_row;
                let current_col = self.selected_extended % cols_per_row;

                let new_selection = match direction {
                    NavDirection::Up => {
                        if current_row > 0 {
                            let new_index = (current_row - 1) * cols_per_row + current_col;
                            new_index.min(255)
                        } else {
                            self.selected_extended // 已在顶部，不移动
                        }
                    }
                    NavDirection::Down => {
                        let total_rows = 256_usize.div_ceil(cols_per_row);
                        if current_row + 1 < total_rows {
                            let new_index = (current_row + 1) * cols_per_row + current_col;
                            new_index.min(255)
                        } else {
                            self.selected_extended // 已在底部，不移动
                        }
                    }
                    NavDirection::Left => {
                        if self.selected_extended > 0 {
                            self.selected_extended - 1
                        } else {
                            255 // 循环到最后一个
                        }
                    }
                    NavDirection::Right => {
                        if self.selected_extended < 255 {
                            self.selected_extended + 1
                        } else {
                            0 // 循环到第一个
                        }
                    }
                };

                self.selected_extended = new_selection;
                self.current_color = Some(AnsiColor::Color256 {
                    c256: new_selection as u8,
                });
            }
            ColorPickerMode::RgbInput => {
                // RGB输入模式：左右切换字段
                match direction {
                    NavDirection::Left => {
                        self.rgb_input.editing_field = match self.rgb_input.editing_field {
                            RgbField::Green => RgbField::Red,
                            RgbField::Blue => RgbField::Green,
                            RgbField::Hex => RgbField::Blue,
                            RgbField::Red => RgbField::Hex, // 循环
                        };
                    }
                    NavDirection::Right => {
                        self.rgb_input.editing_field = match self.rgb_input.editing_field {
                            RgbField::Red => RgbField::Green,
                            RgbField::Green => RgbField::Blue,
                            RgbField::Blue => RgbField::Hex,
                            RgbField::Hex => RgbField::Red, // 循环
                        };
                    }
                    _ => {} // 上下键在RGB模式中不做任何事
                }
            }
        }
    }

    pub fn input_char(&mut self, c: char) {
        if self.mode != ColorPickerMode::RgbInput {
            return;
        }

        match self.rgb_input.editing_field {
            RgbField::Red => {
                if self.rgb_input.r.len() < 3 && c.is_ascii_digit() {
                    self.rgb_input.r.push(c);
                }
            }
            RgbField::Green => {
                if self.rgb_input.g.len() < 3 && c.is_ascii_digit() {
                    self.rgb_input.g.push(c);
                }
            }
            RgbField::Blue => {
                if self.rgb_input.b.len() < 3 && c.is_ascii_digit() {
                    self.rgb_input.b.push(c);
                }
            }
            RgbField::Hex => {
                if self.rgb_input.hex.len() < 6 && c.is_ascii_hexdigit() {
                    self.rgb_input.hex.push(c.to_ascii_uppercase());
                }
            }
        }
        self.update_rgb_color();
    }

    pub fn backspace(&mut self) {
        if self.mode != ColorPickerMode::RgbInput {
            return;
        }

        match self.rgb_input.editing_field {
            RgbField::Red => {
                self.rgb_input.r.pop();
            }
            RgbField::Green => {
                self.rgb_input.g.pop();
            }
            RgbField::Blue => {
                self.rgb_input.b.pop();
            }
            RgbField::Hex => {
                self.rgb_input.hex.pop();
            }
        }
        self.update_rgb_color();
    }

    fn update_rgb_color(&mut self) {
        if !self.rgb_input.hex.is_empty() && self.rgb_input.hex.len() == 6 {
            // Parse hex color
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&self.rgb_input.hex[0..2], 16),
                u8::from_str_radix(&self.rgb_input.hex[2..4], 16),
                u8::from_str_radix(&self.rgb_input.hex[4..6], 16),
            ) {
                self.current_color = Some(AnsiColor::Rgb { r, g, b });
                return;
            }
        }

        // Parse RGB values
        if let (Ok(r), Ok(g), Ok(b)) = (
            self.rgb_input.r.parse::<u8>(),
            self.rgb_input.g.parse::<u8>(),
            self.rgb_input.b.parse::<u8>(),
        ) {
            self.current_color = Some(AnsiColor::Rgb { r, g, b });
        }
    }

    pub fn get_selected_color(&self) -> Option<AnsiColor> {
        self.current_color.clone()
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        if !self.is_open {
            return;
        }

        let popup_area = centered_rect(70, 75, area);

        // Clear the popup area first
        f.render_widget(Clear, popup_area);

        let popup_block = Block::default().borders(Borders::ALL).title("Color Picker");
        let inner = popup_block.inner(popup_area);
        f.render_widget(popup_block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Mode selector (需要3行：上边框+内容+下边框)
                Constraint::Min(8),    // Color picker content
                Constraint::Length(3), // Preview
                Constraint::Length(3), // Actions
            ])
            .split(inner);

        // Mode selector - show all three modes
        let mode_text = match self.mode {
            ColorPickerMode::Basic16 => "[•] Basic (ANSI 16)  [ ] Extended (256)  [ ] RGB",
            ColorPickerMode::Extended256 => "[ ] Basic (ANSI 16)  [•] Extended (256)  [ ] RGB",
            ColorPickerMode::RgbInput => "[ ] Basic (ANSI 16)  [ ] Extended (256)  [•] RGB",
        };

        f.render_widget(
            Paragraph::new(mode_text).block(Block::default().borders(Borders::ALL).title("Mode")),
            chunks[0],
        );

        // Color picker content
        match self.mode {
            ColorPickerMode::Basic16 => self.render_basic_colors(f, chunks[1]),
            ColorPickerMode::Extended256 => self.render_extended_colors(f, chunks[1]),
            ColorPickerMode::RgbInput => self.render_rgb_input(f, chunks[1]),
        }

        // Preview
        self.render_preview(f, chunks[2]);

        // Actions
        f.render_widget(
            Paragraph::new("[Enter] Select  [Esc] Cancel  [Tab] Cycle Mode  [R] RGB")
                .block(Block::default().borders(Borders::ALL)),
            chunks[3],
        );
    }

    fn render_basic_colors(&mut self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Basic Colors (ANSI 16)");

        let inner = block.inner(area);
        f.render_widget(block, area);

        // Create scrollbar-aware layout
        let content_layout = Layout::horizontal([
            Constraint::Min(10),   // Content area
            Constraint::Length(1), // Scrollbar area
        ])
        .split(inner);

        let content_area = content_layout[0];
        let scrollbar_area = content_layout[1];

        let available_height = content_area.height as usize;
        let available_width = content_area.width as usize;

        // Calculate how many colors can fit per row (each color takes ~6 chars: "[ ██ ]" or "  ██  ")
        let colors_per_row = (available_width / 6).max(1);
        let rows_needed = 16_usize.div_ceil(colors_per_row); // Each color needs 1 row

        // Cache columns for navigation consistency
        self.cached_basic_cols = colors_per_row;

        // Render colors in a grid with bracket selection indicator
        for color_index in 0..16 {
            let row = color_index / colors_per_row;
            let col = color_index % colors_per_row;

            // Add row spacing - skip every other row for spacing
            let display_row = row * 2;
            if display_row >= available_height {
                break;
            }

            let item_area = Rect {
                x: content_area.x + (col * 6) as u16,
                y: content_area.y + display_row as u16,
                width: 6, // "[ ██ ]"
                height: 1,
            };

            let is_selected = color_index == self.selected_basic;
            let color = ansi_to_ratatui_color(color_index as u8);

            // Create color block with bracket selection indicator
            let color_text = if is_selected {
                format!("[ {} ]", "██")
            } else {
                format!("  {}  ", "██")
            };

            let paragraph = Paragraph::new(color_text)
                .style(Style::default().fg(color))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(paragraph, item_area);
        }

        // Show color name info at the bottom if there's space (account for row spacing)
        let display_rows_needed = rows_needed * 2;
        if available_height > display_rows_needed && self.selected_basic < 16 {
            let info_area = Rect {
                x: content_area.x,
                y: content_area.y + display_rows_needed as u16,
                width: content_area.width,
                height: 1,
            };
            let info_text = format!(
                "Selected: {} ({})",
                self.selected_basic,
                get_color_name(self.selected_basic as u8)
            );
            let info_paragraph = Paragraph::new(info_text).style(Style::default().fg(Color::Gray));
            f.render_widget(info_paragraph, info_area);
        }

        // Render scrollbar (though not needed for 16 colors in grid)
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(Color::Gray));
        f.render_stateful_widget(scrollbar, scrollbar_area, &mut self.basic_scrollbar_state);
    }

    fn render_extended_colors(&mut self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Extended Colors (256)");

        let inner = block.inner(area);
        f.render_widget(block, area);

        // Create scrollbar-aware layout
        let content_layout = Layout::horizontal([
            Constraint::Min(10),   // Content area
            Constraint::Length(1), // Scrollbar area
        ])
        .split(inner);

        let content_area = content_layout[0];
        let scrollbar_area = content_layout[1];

        let available_height = content_area.height as usize;
        let available_width = content_area.width as usize;

        // Calculate how many colors can fit per row (each color takes ~7 chars: "[ ██ ]" or "  ██  ")
        let colors_per_row = (available_width / 7).max(1);
        // Account for row spacing - each logical row takes 2 display rows (color + spacing)
        let logical_rows_available = if available_height > 3 {
            (available_height - 2) / 2 // Reserve space for info and account for spacing
        } else {
            1
        };
        let colors_per_page = colors_per_row * logical_rows_available;

        // Cache columns for navigation consistency
        self.cached_extended_cols = colors_per_row;

        // Calculate start index based on selected color
        let page_index = self.selected_extended / colors_per_page;
        let start_index = page_index * colors_per_page;
        let end_index = (start_index + colors_per_page).min(256);

        // Render colors in a grid with bracket selection indicator
        for i in 0..(end_index - start_index) {
            let color_index = start_index + i;
            if color_index >= 256 {
                break;
            }

            let row = i / colors_per_row;
            let col = i % colors_per_row;

            // Add row spacing - skip every other row for spacing
            let display_row = row * 2;
            if display_row >= available_height.saturating_sub(2) {
                break;
            }

            let item_area = Rect {
                x: content_area.x + (col * 7) as u16,
                y: content_area.y + display_row as u16,
                width: 7, // "[ ██ ]" or "  ██  "
                height: 1,
            };

            let is_selected = color_index == self.selected_extended;
            let color = Color::Indexed(color_index as u8);

            // Create color block with bracket selection indicator
            let color_text = if is_selected {
                format!("[ {} ]", "██")
            } else {
                format!("  {}  ", "██")
            };

            let paragraph = Paragraph::new(color_text)
                .style(Style::default().fg(color))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(paragraph, item_area);
        }

        // Show navigation info at the bottom if there's space (account for row spacing)
        if available_height > 2 {
            let info_area = Rect {
                x: content_area.x,
                y: content_area.y + available_height.saturating_sub(1) as u16,
                width: content_area.width,
                height: 1,
            };
            let info_text = format!(
                "Selected: {} | Use ↑↓←→ to navigate",
                self.selected_extended
            );
            let info_paragraph = Paragraph::new(info_text).style(Style::default().fg(Color::Gray));
            f.render_widget(info_paragraph, info_area);
        }

        // Render a simple scrollbar indicator
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(Color::Gray));

        // Create a dummy scrollbar state for extended colors
        let mut scrollbar_state = ScrollbarState::new(256).position(self.selected_extended);
        f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    fn render_rgb_input(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // RGB input
                Constraint::Length(3), // Hex input
            ])
            .split(area);

        // RGB input fields
        let rgb_text = format!(
            "R[{}] G[{}] B[{}]",
            if self.rgb_input.editing_field == RgbField::Red {
                format!("> {} <", self.rgb_input.r)
            } else {
                self.rgb_input.r.clone()
            },
            if self.rgb_input.editing_field == RgbField::Green {
                format!("> {} <", self.rgb_input.g)
            } else {
                self.rgb_input.g.clone()
            },
            if self.rgb_input.editing_field == RgbField::Blue {
                format!("> {} <", self.rgb_input.b)
            } else {
                self.rgb_input.b.clone()
            },
        );

        f.render_widget(
            Paragraph::new(rgb_text)
                .block(Block::default().borders(Borders::ALL).title("RGB (0-255)")),
            chunks[0],
        );

        // Hex input
        let hex_text = format!(
            "#{}",
            if self.rgb_input.editing_field == RgbField::Hex {
                format!("> {} <", self.rgb_input.hex)
            } else {
                self.rgb_input.hex.clone()
            },
        );

        f.render_widget(
            Paragraph::new(hex_text).block(Block::default().borders(Borders::ALL).title("Hex")),
            chunks[1],
        );
    }

    fn render_preview(&self, f: &mut Frame, area: Rect) {
        let preview_text = if let Some(color) = &self.current_color {
            match color {
                AnsiColor::Color16 { c16 } => {
                    format!("████ Color 16: {} ({})", c16, get_color_name(*c16))
                }
                AnsiColor::Color256 { c256 } => format!("████ Color 256: {}", c256),
                AnsiColor::Rgb { r, g, b } => format!("████ RGB: ({}, {}, {})", r, g, b),
            }
        } else {
            "████ No color selected".to_string()
        };

        let color = self
            .current_color
            .as_ref()
            .map(|c| match c {
                AnsiColor::Color16 { c16 } => ansi_to_ratatui_color(*c16),
                AnsiColor::Color256 { c256 } => Color::Indexed(*c256),
                AnsiColor::Rgb { r, g, b } => Color::Rgb(*r, *g, *b),
            })
            .unwrap_or(Color::White);

        f.render_widget(
            Paragraph::new(preview_text)
                .style(Style::default().fg(color))
                .block(Block::default().borders(Borders::ALL).title("Preview")),
            area,
        );
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn ansi_to_ratatui_color(ansi: u8) -> Color {
    match ansi {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::White,
        8 => Color::DarkGray,
        9 => Color::LightRed,
        10 => Color::LightGreen,
        11 => Color::LightYellow,
        12 => Color::LightBlue,
        13 => Color::LightMagenta,
        14 => Color::LightCyan,
        15 => Color::Gray,
        _ => Color::White,
    }
}

fn get_color_name(ansi: u8) -> &'static str {
    match ansi {
        0 => "Black",
        1 => "Red",
        2 => "Green",
        3 => "Yellow",
        4 => "Blue",
        5 => "Magenta",
        6 => "Cyan",
        7 => "White",
        8 => "DarkGray",
        9 => "LightRed",
        10 => "LightGreen",
        11 => "LightYellow",
        12 => "LightBlue",
        13 => "LightMagenta",
        14 => "LightCyan",
        15 => "Gray",
        _ => "Unknown",
    }
}
