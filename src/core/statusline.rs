use crate::config::{AnsiColor, Config, SegmentConfig, SegmentId, StyleMode};
use crate::core::segments::SegmentData;
use unicode_width::UnicodeWidthStr;

const EFFORT_ICON_PLAIN: &str = "🧠";
const EFFORT_ICON_NERD_FONT: &str = "\u{f09d1}";

/// Strip ANSI escape sequences and return visible text length
fn visible_width(text: &str) -> usize {
    let mut visible = String::new();
    let mut in_escape = false;
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Start of ANSI escape sequence
            in_escape = true;
            // Skip the [ character
            if chars.peek() == Some(&'[') {
                chars.next();
            }
        } else if in_escape {
            // Skip until we find the end of the escape sequence (letter)
            if ch.is_alphabetic() {
                in_escape = false;
            }
        } else {
            // Regular character
            visible.push(ch);
        }
    }

    visible.width()
}

pub struct StatusLineGenerator {
    config: Config,
}

impl StatusLineGenerator {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn generate(&self, segments: Vec<(SegmentConfig, SegmentData)>) -> String {
        let mut output = Vec::new();
        let enabled_segments: Vec<_> = segments
            .into_iter()
            .filter(|(config, _)| config.enabled)
            .collect();

        for (config, data) in enabled_segments.iter() {
            let rendered = self.render_segment(config, data);
            if !rendered.is_empty() {
                output.push(rendered);
            }
        }

        if output.is_empty() {
            return String::new();
        }

        // Handle Powerline arrow separators with color transition
        if self.config.style.separator == "\u{e0b0}" {
            self.join_with_powerline_arrows(&output, &enabled_segments)
        } else {
            // For all other separators, use white color and simple join
            self.join_with_white_separators(&output)
        }
    }

    /// Generate statusline for TUI preview with proper width calculation
    /// This method handles ANSI escape sequences properly for ratatui rendering
    pub fn generate_for_tui(
        &self,
        segments: Vec<(SegmentConfig, SegmentData)>,
    ) -> ratatui::text::Line<'static> {
        use ansi_to_tui::IntoText;
        use ratatui::text::{Line, Span};

        // Use the same generate method and convert to TUI
        let full_output = self.generate(segments);

        if let Ok(text) = full_output.into_text() {
            if let Some(line) = text.lines.into_iter().next() {
                return line;
            }
        }

        // Fallback to raw text
        Line::from(vec![Span::raw(full_output)])
    }

    /// Generate TUI-optimized text with intelligent wrapping by segment for preview
    pub fn generate_for_tui_preview(
        &self,
        segments: Vec<(SegmentConfig, SegmentData)>,
        max_width: u16,
    ) -> ratatui::text::Text<'_> {
        use ansi_to_tui::IntoText;
        use ratatui::text::{Line, Span, Text};

        let enabled_segments: Vec<_> = segments
            .into_iter()
            .filter(|(config, _)| config.enabled)
            .collect();

        if enabled_segments.is_empty() {
            return Text::from(vec![Line::default()]);
        }

        // Render each segment individually
        let mut rendered_segments = Vec::new();
        let mut segment_configs = Vec::new();

        for (config, data) in &enabled_segments {
            let rendered = self.render_segment(config, data);
            if !rendered.is_empty() {
                rendered_segments.push(rendered);
                segment_configs.push(config.clone());
            }
        }

        if rendered_segments.is_empty() {
            return Text::from(vec![Line::default()]);
        }

        // Pre-calculate separators between segments
        let mut separators = Vec::new();
        for i in 0..rendered_segments.len().saturating_sub(1) {
            let separator = if self.config.style.separator == "\u{e0b0}" {
                // Powerline arrows with color transition
                let prev_bg = segment_configs
                    .get(i)
                    .and_then(|config| config.colors.background.as_ref());
                let curr_bg = segment_configs
                    .get(i + 1)
                    .and_then(|config| config.colors.background.as_ref());
                self.create_powerline_arrow(prev_bg, curr_bg)
            } else {
                // Regular separators with white color
                format!("\x1b[37m{}\x1b[0m", self.config.style.separator)
            };
            separators.push(separator);
        }

        // Intelligent line wrapping by segment
        let mut lines: Vec<String> = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0usize;
        let max_w = max_width as usize;

        for i in 0..rendered_segments.len() {
            let segment = &rendered_segments[i];
            let segment_width = visible_width(segment);

            // Check if adding this segment would exceed max_width
            if current_width > 0 && current_width + segment_width > max_w {
                // Current line would overflow, start a new line
                lines.push(current_line.clone());
                current_line.clear();
                current_width = 0;
            }

            // Add the segment to current line
            current_line.push_str(segment);
            current_width += segment_width;

            // Handle separator if not the last segment
            if i < separators.len() {
                let separator = &separators[i];
                let separator_width = visible_width(separator);

                // Check if next segment exists
                if i + 1 < rendered_segments.len() {
                    let next_segment = &rendered_segments[i + 1];
                    let next_width = visible_width(next_segment);

                    // Check if separator AND next segment both fit
                    if current_width + separator_width + next_width <= max_w {
                        // Both fit, add separator and continue on same line
                        current_line.push_str(separator);
                        current_width += separator_width;
                    } else {
                        // Separator and/or next segment don't fit
                        // Don't add separator, just break line
                        lines.push(current_line.clone());
                        current_line.clear();
                        current_width = 0;
                    }
                }
            }
        }

        // Add the last line if it's not empty
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        // Convert string lines to ratatui Text
        let mut tui_lines = Vec::new();
        for line in lines {
            if let Ok(text) = line.into_text() {
                for tui_line in text.lines {
                    tui_lines.push(tui_line);
                }
            } else {
                tui_lines.push(Line::from(vec![Span::raw(line)]));
            }
        }

        // Ensure we have at least one line
        if tui_lines.is_empty() {
            tui_lines.push(Line::default());
        }

        Text::from(tui_lines)
    }

    fn render_segment(&self, config: &SegmentConfig, data: &SegmentData) -> String {
        let icon = if let Some(dynamic_icon) = data.metadata.get("dynamic_icon") {
            dynamic_icon.clone()
        } else {
            self.get_icon(config)
        };

        // Apply background color to the entire segment if set
        if let Some(bg_color) = &config.colors.background {
            let bg_code = self.apply_background_color(bg_color);

            // Build the entire segment content first
            let icon_colored = if let Some(icon_color) = &config.colors.icon {
                self.apply_color(&icon, Some(icon_color))
                    .replace("\x1b[0m", "")
            } else {
                icon.clone()
            };

            let text = self
                .render_primary_and_secondary(config, data)
                .replace("\x1b[0m", "");
            let segment_content = format!(" {} {} ", icon_colored, text);

            // Apply background to the entire content and reset at the end
            format!("{}{}\x1b[0m", bg_code, segment_content)
        } else {
            // No background color, use original logic
            let icon_colored = self.apply_color(&icon, config.colors.icon.as_ref());
            let text = self.render_primary_and_secondary(config, data);

            format!("{} {}", icon_colored, text)
        }
    }

    fn render_primary_and_secondary(&self, config: &SegmentConfig, data: &SegmentData) -> String {
        let primary = self.apply_style(
            &data.primary,
            config.colors.text.as_ref(),
            config.styles.text_bold,
        );
        if data.secondary.is_empty() {
            return primary;
        }

        let secondary_color = data
            .secondary_color
            .as_ref()
            .or(config.colors.text.as_ref());
        let secondary = self.apply_style(&data.secondary, secondary_color, config.styles.text_bold);

        if config.id == SegmentId::Model {
            let separator = self.apply_color("|", Some(&AnsiColor::Color16 { c16: 7 }));
            let effort_icon = match self.config.style.mode {
                StyleMode::Plain => EFFORT_ICON_PLAIN,
                StyleMode::NerdFont | StyleMode::Powerline => EFFORT_ICON_NERD_FONT,
            };
            let effort_icon = self.apply_style(effort_icon, secondary_color, false);

            return format!("{} {} {} {}", primary, separator, effort_icon, secondary);
        }

        format!("{} {}", primary, secondary)
    }

    fn get_icon(&self, config: &SegmentConfig) -> String {
        match self.config.style.mode {
            StyleMode::Plain => config.icon.plain.clone(),
            StyleMode::NerdFont => config.icon.nerd_font.clone(),
            StyleMode::Powerline => config.icon.nerd_font.clone(), // Future: use Powerline icons
        }
    }

    fn apply_color(&self, text: &str, color: Option<&AnsiColor>) -> String {
        match color {
            Some(AnsiColor::Color16 { c16 }) => {
                let code = if *c16 < 8 { 30 + c16 } else { 90 + (c16 - 8) };
                format!("\x1b[{}m{}\x1b[0m", code, text)
            }
            Some(AnsiColor::Color256 { c256 }) => {
                format!("\x1b[38;5;{}m{}\x1b[0m", c256, text)
            }
            Some(AnsiColor::Rgb { r, g, b }) => {
                format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
            }
            None => text.to_string(),
        }
    }

    fn apply_style(&self, text: &str, color: Option<&AnsiColor>, bold: bool) -> String {
        let mut codes = Vec::new();

        // Add style codes
        if bold {
            codes.push("1".to_string()); // Bold: \x1b[1m
        }

        // Add color codes
        match color {
            Some(AnsiColor::Color16 { c16 }) => {
                let color_code = if *c16 < 8 { 30 + c16 } else { 90 + (c16 - 8) };
                codes.push(color_code.to_string());
            }
            Some(AnsiColor::Color256 { c256 }) => {
                codes.push("38".to_string());
                codes.push("5".to_string());
                codes.push(c256.to_string());
            }
            Some(AnsiColor::Rgb { r, g, b }) => {
                codes.push("38".to_string());
                codes.push("2".to_string());
                codes.push(r.to_string());
                codes.push(g.to_string());
                codes.push(b.to_string());
            }
            None => {}
        }

        if codes.is_empty() {
            text.to_string()
        } else {
            format!("\x1b[{}m{}\x1b[0m", codes.join(";"), text)
        }
    }

    fn apply_background_color(&self, color: &AnsiColor) -> String {
        match color {
            AnsiColor::Color16 { c16 } => {
                let code = if *c16 < 8 { 40 + c16 } else { 100 + (c16 - 8) };
                format!("\x1b[{}m", code)
            }
            AnsiColor::Color256 { c256 } => {
                format!("\x1b[48;5;{}m", c256)
            }
            AnsiColor::Rgb { r, g, b } => {
                format!("\x1b[48;2;{};{};{}m", r, g, b)
            }
        }
    }

    /// Join segments with white separators (non-Powerline)
    fn join_with_white_separators(&self, rendered_segments: &[String]) -> String {
        if rendered_segments.is_empty() {
            return String::new();
        }

        // Use white color for separator
        let white_separator = format!("\x1b[37m{}\x1b[0m", self.config.style.separator);
        rendered_segments.join(&white_separator)
    }

    /// Join segments with Powerline arrow separators with proper color transitions
    fn join_with_powerline_arrows(
        &self,
        rendered_segments: &[String],
        segment_configs: &[(SegmentConfig, SegmentData)],
    ) -> String {
        if rendered_segments.is_empty() {
            return String::new();
        }

        if rendered_segments.len() == 1 {
            return rendered_segments[0].clone();
        }

        let mut result = rendered_segments[0].clone();

        for (i, _) in rendered_segments.iter().enumerate().skip(1) {
            let prev_bg = segment_configs
                .get(i - 1)
                .and_then(|(config, _)| config.colors.background.as_ref());
            let curr_bg = segment_configs
                .get(i)
                .and_then(|(config, _)| config.colors.background.as_ref());

            // Create Powerline arrow with color transition
            let arrow = self.create_powerline_arrow(prev_bg, curr_bg);

            result.push_str(&arrow);
            result.push_str(&rendered_segments[i]);
        }

        // Reset colors at the end
        result.push_str("\x1b[0m");
        result
    }

    /// Create a Powerline arrow with proper color transition
    fn create_powerline_arrow(
        &self,
        prev_bg: Option<&AnsiColor>,
        curr_bg: Option<&AnsiColor>,
    ) -> String {
        let arrow_char = "\u{e0b0}";

        match (prev_bg, curr_bg) {
            (Some(prev), Some(curr)) => {
                // Arrow foreground = previous segment's background
                // Arrow background = current segment's background
                let fg_code = self.color_to_foreground_code(prev);
                let bg_code = self.apply_background_color(curr);
                format!("{}{}{}\x1b[0m", bg_code, fg_code, arrow_char)
            }
            (Some(prev), None) => {
                // Previous segment has background, current doesn't
                let fg_code = self.color_to_foreground_code(prev);
                format!("{}{}\x1b[0m", fg_code, arrow_char)
            }
            (None, Some(curr)) => {
                // Current segment has background, previous doesn't
                let bg_code = self.apply_background_color(curr);
                format!("{}{}\x1b[0m", bg_code, arrow_char)
            }
            (None, None) => {
                // Neither segment has background color
                arrow_char.to_string()
            }
        }
    }

    /// Convert AnsiColor to foreground color code
    fn color_to_foreground_code(&self, color: &AnsiColor) -> String {
        match color {
            AnsiColor::Color16 { c16 } => {
                let code = if *c16 < 8 { 30 + c16 } else { 90 + (c16 - 8) };
                format!("\x1b[{}m", code)
            }
            AnsiColor::Color256 { c256 } => {
                format!("\x1b[38;5;{}m", c256)
            }
            AnsiColor::Rgb { r, g, b } => {
                format!("\x1b[38;2;{};{};{}m", r, g, b)
            }
        }
    }
}

pub fn collect_all_segments(
    config: &Config,
    input: &crate::config::InputData,
) -> Vec<(SegmentConfig, SegmentData)> {
    use crate::core::segments::*;

    let mut results = Vec::new();

    for segment_config in &config.segments {
        // Skip disabled segments to avoid unnecessary API requests
        if !segment_config.enabled {
            continue;
        }

        let segment_data = match segment_config.id {
            crate::config::SegmentId::Model => {
                let segment = ModelSegment::new();
                segment.collect(input)
            }
            crate::config::SegmentId::Directory => {
                let segment = DirectorySegment::new();
                segment.collect(input)
            }
            crate::config::SegmentId::Git => {
                let show_sha = segment_config
                    .options
                    .get("show_sha")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let segment = GitSegment::new().with_sha(show_sha);
                segment.collect(input)
            }
            crate::config::SegmentId::ContextWindow => {
                let segment = ContextWindowSegment::new();
                segment.collect(input)
            }
            crate::config::SegmentId::Usage => {
                let segment = UsageSegment::new();
                segment.collect(input)
            }
            crate::config::SegmentId::Cost => {
                let segment = CostSegment::new();
                segment.collect(input)
            }
            crate::config::SegmentId::Session => {
                let segment = SessionSegment::new();
                segment.collect(input)
            }
            crate::config::SegmentId::OutputStyle => {
                let segment = OutputStyleSegment::new();
                segment.collect(input)
            }
            crate::config::SegmentId::Update => {
                let segment = UpdateSegment::new();
                segment.collect(input)
            }
        };

        if let Some(data) = segment_data {
            results.push((segment_config.clone(), data));
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        AnsiColor, ColorConfig, IconConfig, SegmentId, StyleConfig, TextStyleConfig,
    };
    use std::collections::HashMap;

    fn strip_ansi(text: &str) -> String {
        let mut visible = String::new();
        let mut in_escape = false;

        for character in text.chars() {
            if character == '\x1b' {
                in_escape = true;
            } else if in_escape {
                if character.is_alphabetic() {
                    in_escape = false;
                }
            } else {
                visible.push(character);
            }
        }

        visible
    }

    fn model_test_config(
        mode: StyleMode,
        background: Option<AnsiColor>,
    ) -> (Config, SegmentConfig) {
        let config = Config {
            style: StyleConfig {
                mode,
                separator: if mode == StyleMode::Powerline {
                    "\u{e0b0}".to_string()
                } else {
                    " | ".to_string()
                },
            },
            segments: Vec::new(),
            theme: "test".to_string(),
        };
        let segment_config = SegmentConfig {
            id: SegmentId::Model,
            enabled: true,
            icon: IconConfig {
                plain: "🤖".to_string(),
                nerd_font: "M".to_string(),
            },
            colors: ColorConfig {
                icon: Some(AnsiColor::Color16 { c16: 14 }),
                text: Some(AnsiColor::Color16 { c16: 14 }),
                background,
            },
            styles: TextStyleConfig::default(),
            options: HashMap::new(),
        };

        (config, segment_config)
    }

    fn model_data(primary: &str, secondary: &str) -> SegmentData {
        SegmentData {
            primary: primary.to_string(),
            secondary: secondary.to_string(),
            secondary_color: Some(AnsiColor::Rgb {
                r: 180,
                g: 92,
                b: 255,
            }),
            metadata: HashMap::new(),
        }
    }

    fn directory_segment() -> (SegmentConfig, SegmentData) {
        let config = SegmentConfig {
            id: SegmentId::Directory,
            enabled: true,
            icon: IconConfig {
                plain: "📁".to_string(),
                nerd_font: "D".to_string(),
            },
            colors: ColorConfig {
                icon: None,
                text: None,
                background: None,
            },
            styles: TextStyleConfig::default(),
            options: HashMap::new(),
        };
        let data = SegmentData {
            primary: "repo".to_string(),
            secondary: String::new(),
            secondary_color: None,
            metadata: HashMap::new(),
        };

        (config, data)
    }

    #[test]
    fn renders_model_effort_with_its_independent_bright_purple_color() {
        let (config, segment_config) = model_test_config(StyleMode::Plain, None);
        let data = model_data("k3[1m] 1M", "ultracode");

        let output = StatusLineGenerator::new(config).generate(vec![(segment_config, data)]);

        assert!(output.contains("\x1b[96mk3[1m] 1M\x1b[0m"));
        assert!(output.contains("\x1b[37m|\x1b[0m"));
        assert!(output.contains("\x1b[38;2;180;92;255m🧠\x1b[0m"));
        assert!(output.contains("\x1b[38;2;180;92;255multracode\x1b[0m"));
        assert_eq!(strip_ansi(&output), "🤖 k3[1m] 1M | 🧠 ultracode");
    }

    #[test]
    fn renders_model_effort_item_with_a_background() {
        let (config, segment_config) =
            model_test_config(StyleMode::Plain, Some(AnsiColor::Color256 { c256: 24 }));
        let data = model_data("Kimi K2.7", "max");

        let output = StatusLineGenerator::new(config).generate(vec![(segment_config, data)]);

        assert_eq!(strip_ansi(&output), " 🤖 Kimi K2.7 | 🧠 max ");
        assert!(output.ends_with("\x1b[0m"));
    }

    #[test]
    fn renders_nerd_font_brain_for_model_effort() {
        let (config, segment_config) = model_test_config(StyleMode::NerdFont, None);
        let data = model_data("Kimi K2.7", "xhigh");

        let output = StatusLineGenerator::new(config).generate(vec![(segment_config, data)]);

        assert_eq!(strip_ansi(&output), "M Kimi K2.7 | 󰧑 xhigh");
    }

    #[test]
    fn renders_nerd_font_brain_for_powerline_model_effort() {
        let (config, segment_config) =
            model_test_config(StyleMode::Powerline, Some(AnsiColor::Color256 { c256: 24 }));
        let data = model_data("Kimi K2.7", "max");
        let (directory_config, directory_data) = directory_segment();

        let output = StatusLineGenerator::new(config).generate(vec![
            (segment_config, data),
            (directory_config, directory_data),
        ]);

        assert!(output.contains("\u{e0b0}"));
        assert_eq!(strip_ansi(&output), " M Kimi K2.7 | 󰧑 max \u{e0b0}D repo");
    }

    #[test]
    fn omits_model_effort_separator_and_icon_without_effort() {
        let (config, segment_config) = model_test_config(StyleMode::Plain, None);
        let data = model_data("Kimi K2.7", "");

        let output = StatusLineGenerator::new(config).generate(vec![(segment_config, data)]);

        assert_eq!(strip_ansi(&output), "🤖 Kimi K2.7");
    }

    #[test]
    fn counts_model_and_effort_emoji_by_terminal_column_width() {
        assert_eq!(visible_width("🤖 Kimi K2.7 | 🧠 max"), 21);
    }

    #[test]
    fn wraps_before_next_segment_at_the_true_emoji_width_boundary() {
        let (config, model_config) = model_test_config(StyleMode::Plain, None);
        let model_data = model_data("Kimi K2.7", "max");
        let (directory_config, directory_data) = directory_segment();

        let generator = StatusLineGenerator::new(config);
        let output = generator.generate_for_tui_preview(
            vec![
                (model_config, model_data),
                (directory_config, directory_data),
            ],
            28,
        );

        assert_eq!(output.lines.len(), 2);
    }
}
