//! Configuration format detection and serialization.

use crate::{ConfigError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Supported configuration formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigFormat {
    /// JSON format.
    Json,
    /// TOML format.
    Toml,
    /// YAML format.
    Yaml,
    /// Format to be auto-detected.
    Auto,
}

impl Default for ConfigFormat {
    fn default() -> Self {
        Self::Auto
    }
}

/// Format detection strategy.
#[derive(Debug, Clone, Copy)]
pub enum FormatDetect {
    /// Auto-detect based on content.
    Auto,
    /// Force a specific format.
    Forced(ConfigFormat),
}

impl ConfigFormat {
    /// Detect format from file extension.
    pub fn from_path(path: &str) -> Self {
        let path = path.to_lowercase();
        if path.ends_with(".json") {
            Self::Json
        } else if path.ends_with(".toml") {
            Self::Toml
        } else if path.ends_with(".yaml") || path.ends_with(".yml") {
            Self::Yaml
        } else {
            Self::Auto
        }
    }

    /// Detect format from content by examining the first character.
    pub fn from_content(content: &str) -> Self {
        let trimmed = content.trim();
        // Check for TOML section headers first (more specific)
        if trimmed.starts_with('[') && !trimmed.starts_with("[]") {
            // TOML sections like [section] or [section.subsection]
            Self::Toml
        } else if trimmed.starts_with('{') || trimmed.starts_with('[') {
            // JSON objects { } or arrays [ ]
            Self::Json
        } else if trimmed.starts_with("---")
            || trimmed.starts_with('-')
            || (trimmed.contains(':') && !trimmed.starts_with('{'))
        {
            // YAML: starts with ---, list items -, or key: value format
            Self::Yaml
        } else {
            Self::Toml // Default to TOML
        }
    }

    /// Parse content into a JSON Value.
    pub fn parse_to_json(self, content: &str) -> Result<JsonValue> {
        match self {
            Self::Json => serde_json::from_str(content).map_err(|e| ConfigError::json_parse(e.to_string())),
            #[cfg(feature = "toml")]
            Self::Toml => {
                let value: toml::Value = toml::from_str(content)
                    .map_err(|e| ConfigError::toml_parse(e.to_string()))?;
                serde_json::to_value(value).map_err(|e| ConfigError::custom("toml", e.to_string()))
            }
            #[cfg(not(feature = "toml"))]
            Self::Toml => Err(ConfigError::custom("toml", "TOML feature not enabled")),
            #[cfg(feature = "yaml")]
            Self::Yaml => {
                let value: serde_yaml::Value = serde_yaml::from_str(content)
                    .map_err(|e| ConfigError::yaml_parse(e.to_string()))?;
                serde_json::to_value(value).map_err(|e| ConfigError::custom("yaml", e.to_string()))
            }
            #[cfg(not(feature = "yaml"))]
            Self::Yaml => Err(ConfigError::custom("yaml", "YAML feature not enabled")),
            Self::Auto => {
                serde_json::to_string_pretty(&serde_json::json!({}))
                    .map_err(|_| ConfigError::custom("auto", "could not parse"))?;
                Self::from_content(content).parse_to_json(content)
            }
        }
    }

    /// Serialize a value to string in this format.
    pub fn serialize(self, value: &JsonValue) -> Result<String> {
        match self {
            Self::Json => serde_json::to_string_pretty(value).map_err(|e| ConfigError::json_parse(e.to_string())),
            #[cfg(feature = "toml")]
            Self::Toml => {
                let value: toml::Value = serde_json::from_value(value.clone())
                    .map_err(|e| ConfigError::custom("toml", e.to_string()))?;
                toml::to_string_pretty(&value).map_err(|e| ConfigError::toml_parse(e.to_string()))
            }
            #[cfg(not(feature = "toml"))]
            Self::Toml => Err(ConfigError::custom("toml", "TOML feature not enabled")),
            #[cfg(feature = "yaml")]
            Self::Yaml => serde_yaml::to_string(value).map_err(|e| ConfigError::yaml_parse(e.to_string())),
            #[cfg(not(feature = "yaml"))]
            Self::Yaml => Err(ConfigError::custom("yaml", "YAML feature not enabled")),
            Self::Auto => {
                serde_json::to_string_pretty(value).map_err(|e| ConfigError::json_parse(e.to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_path() {
        assert_eq!(ConfigFormat::from_path("config.json"), ConfigFormat::Json);
        assert_eq!(ConfigFormat::from_path("config.toml"), ConfigFormat::Toml);
        assert_eq!(ConfigFormat::from_path("config.yaml"), ConfigFormat::Yaml);
        assert_eq!(ConfigFormat::from_path("config"), ConfigFormat::Auto);
    }

    #[test]
    fn test_from_content() {
        assert_eq!(ConfigFormat::from_content(r#"{"key": "value"}"#), ConfigFormat::Json);
        assert_eq!(ConfigFormat::from_content(r#"[section]"#), ConfigFormat::Toml);
        assert_eq!(ConfigFormat::from_content("key: value"), ConfigFormat::Yaml);
    }

    #[test]
    fn test_toml_parse() {
        let content = r#"
            [database]
            host = "localhost"
            port = 5432
        "#;
        let format = ConfigFormat::from_content(content);
        let json = format.parse_to_json(content).unwrap();
        assert_eq!(json["database"]["host"], "localhost");
    }

    #[test]
    fn test_json_roundtrip() {
        let content = r#"{"name": "test", "count": 42}"#;
        let format = ConfigFormat::Json;
        let parsed = format.parse_to_json(content).unwrap();
        let serialized = format.serialize(&parsed).unwrap();
        assert!(serialized.contains("test"));
        assert!(serialized.contains("42"));
    }
}
