use super::{Segment, SegmentData};
use crate::config::{AnsiColor, InputData, ModelConfig, SegmentId};
use crate::core::effort::resolve_effort_label;
use std::collections::HashMap;

#[derive(Default)]
pub struct ModelSegment;

impl ModelSegment {
    pub fn new() -> Self {
        Self
    }
}

impl Segment for ModelSegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        let mut metadata = HashMap::new();
        metadata.insert("model_id".to_string(), input.model.id.clone());
        metadata.insert("display_name".to_string(), input.model.display_name.clone());

        let effort_level = resolve_effort_label(input).unwrap_or_default();
        let secondary_color = (!effort_level.is_empty()).then_some(AnsiColor::Rgb {
            r: 180,
            g: 92,
            b: 255,
        });

        Some(SegmentData {
            primary: self.format_model_name(&input.model.id, &input.model.display_name),
            secondary: effort_level,
            secondary_color,
            metadata,
        })
    }

    fn id(&self) -> SegmentId {
        SegmentId::Model
    }
}

impl ModelSegment {
    fn format_model_name(&self, id: &str, display_name: &str) -> String {
        let model_config = ModelConfig::load();

        if let Some(config_name) = model_config.get_display_name(id) {
            // Model recognized by config, display_name already includes modifier suffix
            config_name
        } else {
            // Fallback: prefer upstream display_name, fall back to model_id if empty
            let base = if display_name.is_empty() {
                id.to_string()
            } else {
                display_name.to_string()
            };
            // Still apply context modifier suffix (e.g., " 1M") if present
            match model_config.get_display_suffix(id) {
                Some(suffix) => format!("{}{}", base, suffix),
                None => base,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn input(model: serde_json::Value, effort: Option<serde_json::Value>) -> InputData {
        let mut value = json!({
            "model": model,
            "workspace": { "current_dir": "/tmp/project" },
            "transcript_path": "/tmp/missing.jsonl",
            "version": "2.1.216"
        });
        if let Some(effort) = effort {
            value["effort"] = effort;
        }

        serde_json::from_value(value).expect("statusline input should deserialize")
    }

    #[test]
    fn appends_supported_effort_levels_to_the_model_segment() {
        let segment = ModelSegment::new();

        for level in ["low", "medium", "high", "xhigh", "max", "ultracode"] {
            let data = segment
                .collect(&input(
                    json!({ "id": "kimi-k2.7-code", "display_name": "Kimi K2.7" }),
                    Some(json!({ "level": level })),
                ))
                .unwrap();

            assert_eq!(data.primary, "Kimi K2.7");
            assert_eq!(data.secondary, level);
            assert_eq!(
                data.secondary_color,
                Some(AnsiColor::Rgb {
                    r: 180,
                    g: 92,
                    b: 255
                })
            );
        }
    }

    #[test]
    fn omits_missing_null_or_empty_effort_levels() {
        let segment = ModelSegment::new();
        let cases = [
            None,
            Some(serde_json::Value::Null),
            Some(json!({})),
            Some(json!({ "level": "" })),
            Some(json!({ "level": "   " })),
        ];

        for effort in cases {
            let data = segment
                .collect(&input(json!("future-model"), effort))
                .unwrap();
            assert!(data.secondary.is_empty());
            assert_eq!(data.secondary_color, None);
        }
    }

    #[test]
    fn trims_and_preserves_future_effort_levels() {
        let data = ModelSegment::new()
            .collect(&input(
                json!({ "id": "future-model", "display_name": "Future Model" }),
                Some(json!({ "level": "  future-level  " })),
            ))
            .unwrap();

        assert_eq!(data.secondary, "future-level");
    }

    #[test]
    fn supports_string_and_object_models_without_rendering_the_cli_version() {
        let segment = ModelSegment::new();
        let string_model = segment
            .collect(&input(json!("future-string-model"), None))
            .unwrap();
        let object_model = segment
            .collect(&input(
                json!({ "id": "kimi-k2.7-code", "display_name": "Kimi K2.7" }),
                Some(json!({ "level": "xhigh" })),
            ))
            .unwrap();

        assert_eq!(string_model.primary, "future-string-model");
        assert_eq!(object_model.primary, "Kimi K2.7");
        assert_eq!(object_model.secondary, "xhigh");
        assert!(
            !format!("{} {}", object_model.primary, object_model.secondary).contains("2.1.216")
        );
    }
}
