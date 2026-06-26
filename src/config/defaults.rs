// Legacy defaults - now using ui/themes/presets.rs for configuration
// This file kept for backward compatibility

use super::types::Config;

impl Default for Config {
    fn default() -> Self {
        // Use the theme presets as the source of truth
        crate::ui::themes::ThemePresets::get_default()
    }
}
