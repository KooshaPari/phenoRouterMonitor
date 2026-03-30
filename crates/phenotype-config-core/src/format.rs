//! Configuration file format detection and handling.

use crate::error::{ConfigError, Result};
use std::path::Path;

/// Supported configuration file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// TOML format (.toml)
    Toml,
    /// YAML format (.yaml, .yml)
    Yaml,
    /// JSON format (.json)
    Json,
}

impl ConfigFormat {
    /// Detect format from file extension.
    ///
    /// # Arguments
    /// * `path` - File path to detect format from
    ///
    /// # Returns
    /// The detected format, or `UnsupportedFormat` error if unrecognized.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let format = match extension.as_str() {
            "toml" => Some(ConfigFormat::Toml),
            "yaml" | "yml" => Some(ConfigFormat::Yaml),
            "json" => Some(ConfigFormat::Json),
            _ => None,
        };

        if let Some(fmt) = format {
            return Ok(fmt);
        }

        // Attempt content-based detection for files without extension.
        if path.exists() && path.is_file() {
            if let Ok(content) = std::fs::read_to_string(path) {
                return ConfigFormat::from_content(&content);
            }
        }

        Err(ConfigError::unsupported_format(extension))
    }

    /// Detect format from the content of a configuration file.
    pub fn from_content(content: &str) -> Result<Self> {
        let content = content.trim_start();

        if content.is_empty() {
            return Err(ConfigError::unsupported_format("empty content".to_string()));
        }

        if content.starts_with('{') || content.starts_with('[') {
            return Ok(ConfigFormat::Json);
        }

        if content.starts_with("---") || content.contains(":") {
            return Ok(ConfigFormat::Yaml);
        }

        if content.contains('=') {
            // TOML typically uses key = value syntax.
            return Ok(ConfigFormat::Toml);
        }

        Err(ConfigError::unsupported_format("unknown format".to_string()))
    }

    /// Get the typical file extension for this format.
    pub fn extension(self) -> &'static str {
        match self {
            ConfigFormat::Toml => "toml",
            ConfigFormat::Yaml => "yaml",
            ConfigFormat::Json => "json",
        }
    }

    /// Parse a configuration string in this format to a JSON value.
    ///
    /// This normalizes all formats to JSON for uniform handling.
    pub fn parse_to_json(self, content: &str) -> Result<serde_json::Value> {
        match self {
            ConfigFormat::Json => {
                serde_json::from_str(content).map_err(|e| ConfigError::json_parse(e.to_string()))
            }
            ConfigFormat::Toml => {
                let toml_value: toml::Value =
                    toml::from_str(content).map_err(|e| ConfigError::toml_parse(e.to_string()))?;
                // Convert TOML to JSON for uniform handling
                serde_json::to_value(toml_value)
                    .map_err(|e| ConfigError::deserialize(e.to_string()))
            }
            ConfigFormat::Yaml => {
                let yaml_value: serde_yaml::Value =
                    serde_yaml::from_str(content).map_err(|e| ConfigError::yaml_parse(e.to_string()))?;
                // Convert YAML to JSON for uniform handling
                serde_json::to_value(yaml_value)
                    .map_err(|e| ConfigError::deserialize(e.to_string()))
            }
        }
    }

    /// Deserialize a configuration string to a strongly-typed struct.
    pub fn deserialize<T: serde::de::DeserializeOwned>(self, content: &str) -> Result<T> {
        match self {
            ConfigFormat::Json => {
                serde_json::from_str(content).map_err(|e| ConfigError::json_parse(e.to_string()))
            }
            ConfigFormat::Toml => {
                toml::from_str(content).map_err(|e| ConfigError::toml_parse(e.to_string()))
            }
            ConfigFormat::Yaml => {
                serde_yaml::from_str(content).map_err(|e| ConfigError::yaml_parse(e.to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection_toml() {
        assert_eq!(
            ConfigFormat::from_path("config.toml").unwrap(),
            ConfigFormat::Toml
        );
    }

    #[test]
    fn test_format_detection_yaml() {
        assert_eq!(
            ConfigFormat::from_path("config.yaml").unwrap(),
            ConfigFormat::Yaml
        );
        assert_eq!(
            ConfigFormat::from_path("config.yml").unwrap(),
            ConfigFormat::Yaml
        );
    }

    #[test]
    fn test_format_detection_json() {
        assert_eq!(
            ConfigFormat::from_path("config.json").unwrap(),
            ConfigFormat::Json
        );
    }

    #[test]
    fn test_format_detection_unsupported() {
        assert!(ConfigFormat::from_path("config.xml").is_err());
        assert!(ConfigFormat::from_path("config.ini").is_err());
    }

    #[test]
    fn test_format_extension() {
        assert_eq!(ConfigFormat::Toml.extension(), "toml");
        assert_eq!(ConfigFormat::Yaml.extension(), "yaml");
        assert_eq!(ConfigFormat::Json.extension(), "json");
    }

    #[test]
    fn test_from_content_json() {
        assert_eq!(ConfigFormat::from_content(r#"{"key":"value"}"#).unwrap(), ConfigFormat::Json);
    }

    #[test]
    fn test_from_content_toml() {
        assert_eq!(ConfigFormat::from_content("key = \"value\"\n").unwrap(), ConfigFormat::Toml);
    }

    #[test]
    fn test_from_content_yaml() {
        assert_eq!(ConfigFormat::from_content("---\nkey: value\n").unwrap(), ConfigFormat::Yaml);
    }

    #[test]
    fn test_from_content_unknown() {
        assert!(ConfigFormat::from_content("***").is_err());
    }

    #[test]
    fn test_json_parse() {
        let json = r#"{"key": "value", "number": 42}"#;
        let result = ConfigFormat::Json.parse_to_json(json);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["key"].as_str(), Some("value"));
        assert_eq!(value["number"].as_i64(), Some(42));
    }

    #[test]
    fn test_toml_parse() {
        let toml = r#"
key = "value"
number = 42
"#;
        let result = ConfigFormat::Toml.parse_to_json(toml);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["key"].as_str(), Some("value"));
        assert_eq!(value["number"].as_i64(), Some(42));
    }

    #[test]
    fn test_yaml_parse() {
        let yaml = r#"
key: value
number: 42
"#;
        let result = ConfigFormat::Yaml.parse_to_json(yaml);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["key"].as_str(), Some("value"));
        assert_eq!(value["number"].as_i64(), Some(42));
    }
}
