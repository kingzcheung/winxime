use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SmartSuggestionConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_suggestion_count")]
    pub suggestion_count: i32,
    #[serde(default)]
    pub record_user_frequency: bool,
    #[serde(default)]
    pub auto_adjust_frequency: bool,
    #[serde(default = "default_learning_threshold")]
    pub learning_threshold: i32,
    #[serde(default)]
    pub model: SmartSuggestionModelConfig,
}

impl Default for SmartSuggestionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            suggestion_count: default_suggestion_count(),
            record_user_frequency: false,
            auto_adjust_frequency: false,
            learning_threshold: default_learning_threshold(),
            model: SmartSuggestionModelConfig::default(),
        }
    }
}

fn default_suggestion_count() -> i32 {
    5
}
fn default_learning_threshold() -> i32 {
    3
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SmartSuggestionModelConfig {
    #[serde(default = "default_model_provider")]
    pub provider: String,
    #[serde(default = "default_model_name")]
    pub name: String,
    #[serde(default)]
    pub auto_download: bool,
    #[serde(default)]
    pub files: Vec<SmartSuggestionModelFile>,
}

impl Default for SmartSuggestionModelConfig {
    fn default() -> Self {
        Self {
            provider: default_model_provider(),
            name: default_model_name(),
            auto_download: false,
            files: Vec::new(),
        }
    }
}

fn default_model_provider() -> String {
    "modelscope".to_string()
}
fn default_model_name() -> String {
    "predictive-text-small".to_string()
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct SmartSuggestionModelFile {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub filename: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ColorScheme, HotkeyConfig, StyleConfig};

    #[test]
    fn test_smart_suggestion_defaults() {
        let config = SmartSuggestionConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.suggestion_count, 5);
        assert_eq!(config.learning_threshold, 3);
        assert_eq!(config.model.provider, "modelscope");
        assert_eq!(config.model.name, "predictive-text-small");
    }

    #[test]
    fn test_smart_suggestion_deserialize() {
        let yaml = "
enabled: true
suggestion_count: 10
model:
  provider: custom
  name: my-model
  files:
    - url: https://example.com/file1.bin
      filename: file1.bin
    - url: https://example.com/file2.bin
      filename: file2.bin
";
        let config: SmartSuggestionConfig = serde_saphyr::from_str(yaml).ok().unwrap();
        assert!(config.enabled);
        assert_eq!(config.suggestion_count, 10);
        assert_eq!(config.model.provider, "custom");
        assert_eq!(config.model.name, "my-model");
        assert_eq!(config.model.files.len(), 2);
        assert_eq!(config.model.files[0].filename, "file1.bin");
        assert_eq!(config.model.files[1].filename, "file2.bin");
    }

    #[test]
    fn test_builtin_config() {
        const DEFAULT_CONFIG: &[u8] = include_bytes!("../resources/xime.yaml");
        let config: crate::XimeConfig = serde_saphyr::from_slice(DEFAULT_CONFIG).ok().unwrap();
        assert!(config.smart_suggestion.enabled);
        assert_eq!(config.smart_suggestion.suggestion_count, 5);
        assert_eq!(config.smart_suggestion.learning_threshold, 3);
        assert_eq!(config.smart_suggestion.model.name, "predictive-text-small");
        assert_eq!(config.smart_suggestion.model.files.len(), 3);
    }

    #[test]
    fn test_style_config_defaults() {
        let config = StyleConfig::default();
        assert_eq!(config.font_size, 14.0);
        assert_eq!(config.candidate_count, 5);
        assert!(config.horizontal);
        assert_eq!(config.corner_radius, 8.0);
        assert_eq!(config.color_scheme, "lavender_purple");
    }

    #[test]
    fn test_color_scheme_defaults() {
        let scheme = ColorScheme::default();
        assert_eq!(scheme.primary_color, 0x8F73E2);
    }

    #[test]
    fn test_hotkey_config_defaults() {
        let config = HotkeyConfig::default();
        assert_eq!(config.show_key, "Ctrl");
    }
}
