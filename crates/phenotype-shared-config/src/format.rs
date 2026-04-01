//! Configuration format detection and parsing.
//!
//! Supports TOML, JSON, and YAML formats with automatic detection.

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::{ConfigError, Result};

/// Supported configuration file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// TOML format (`.toml` files)
    Toml,
    /// JSON format (`.json` files)
    Json,
    /// YAML format (`.yaml`, `.yml` files)
    Yaml,
    /// Auto-detected format
    Auto,
}

impl ConfigFormat {
    /// Detect format from file extension.
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Self {
        let path = path.as_ref();
        match path.extension().and_then(|e| e.to_str()) {
            Some("toml") => Self::Toml,
            Some("json") => Self::Json,
            Some("yaml") | Some("yml") => Self::Yaml,
            _ => Self::Auto,
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
    pub fn parse_to_json(self, content: &str) -> Result<serde_json::Value> {
        match self {
            Self::Json => serde_json::from_str(content).map_err(|e| ConfigError::json_parse(e.to_string())),
            #[cfg(feature = "toml")]
            Self::Toml => {
                let toml_value: toml::Value = toml::from_str(content)?;
                Ok(toml_to_json(&toml_value))
            }
            #[cfg(not(feature = "toml"))]
            Self::Toml => Err(ConfigError::custom("toml", "TOML feature not enabled")),
            #[cfg(feature = "yaml")]
            Self::Yaml => {
                let yaml_value: serde_yaml::Value = serde_yaml::from_str(content)?;
                Ok(yaml_to_json(&yaml_value))
            }
            #[cfg(not(feature = "yaml"))]
            Self::Yaml => Err(ConfigError::custom("yaml", "YAML feature not enabled")),
            Self::Auto => {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(content) {
                    return Ok(v);
                }
                #[cfg(feature = "toml")]
                if let Ok(v) = toml::from_str::<toml::Value>(content) {
                    return Ok(toml_to_json(&v));
                }
                #[cfg(feature = "yaml")]
                if let Ok(v) = serde_yaml::from_str::<serde_yaml::Value>(content) {
                    return Ok(yaml_to_json(&v));
                }
                Err(ConfigError::invalid_format("JSON, TOML, or YAML"))
            }
        }
    }

    /// Deserialize content into a typed struct.
    pub fn deserialize<T: DeserializeOwned>(self, content: &str) -> Result<T> {
        match self {
            Self::Json => serde_json::from_str(content).map_err(|e| ConfigError::json_parse(e.to_string())),
            #[cfg(feature = "toml")]
            Self::Toml => toml::from_str(content).map_err(|e| ConfigError::TomlParse {
                path: None,
                reason: e.to_string(),
            }),
            #[cfg(not(feature = "toml"))]
            Self::Toml => Err(ConfigError::custom("toml", "TOML feature not enabled")),
            #[cfg(feature = "yaml")]
            Self::Yaml => serde_yaml::from_str(content).map_err(|e| ConfigError::YamlParse {
                path: None,
                reason: e.to_string(),
            }),
            #[cfg(not(feature = "yaml"))]
            Self::Yaml => Err(ConfigError::custom("yaml", "YAML feature not enabled")),
            Self::Auto => {
                if let Ok(v) = serde_json::from_str(content) {
                    return Ok(v);
                }
                #[cfg(feature = "toml")]
                if let Ok(v) = toml::from_str(content) {
                    return Ok(v);
                }
                #[cfg(feature = "yaml")]
                if let Ok(v) = serde_yaml::from_str(content) {
                    return Ok(v);
                }
                Err(ConfigError::invalid_format("JSON, TOML, or YAML"))
            }
        }
    }

    /// Serialize a value to the format's string representation.
    pub fn serialize<T: Serialize>(self, value: &T) -> Result<String> {
        match self {
            Self::Json => serde_json::to_string_pretty(value).map_err(|e| ConfigError::json_parse(e.to_string())),
            #[cfg(feature = "toml")]
            Self::Toml => {
                let toml_value = json_to_toml(value)?;
                toml::to_string_pretty(&toml_value)
                    .map_err(|e| ConfigError::TomlParse { path: None, reason: e.to_string() })
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

    /// Get the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Toml => "toml",
            Self::Yaml => "yaml",
            Self::Auto => "json",
        }
    }
}

impl Default for ConfigFormat {
    fn default() -> Self {
        Self::Auto
    }
}

impl std::fmt::Display for ConfigFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "JSON"),
            Self::Toml => write!(f, "TOML"),
            Self::Yaml => write!(f, "YAML"),
            Self::Auto => write!(f, "auto"),
        }
    }
}

// ============================================================================
// Conversion utilities
// ============================================================================

/// Convert a TOML value to a JSON Value.
fn toml_to_json(value: &toml::Value) -> serde_json::Value {
    match value {
        toml::Value::String(s) => serde_json::Value::String(s.clone()),
        toml::Value::Integer(i) => serde_json::Value::Number((*i).into()),
        toml::Value::Float(f) => {
            serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        toml::Value::Boolean(b) => serde_json::Value::Bool(*b),
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
        toml::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(toml_to_json).collect())
        }
        toml::Value::Table(table) => {
            serde_json::Value::Object(
                table
                    .iter()
                    .map(|(k, v)| (k.clone(), toml_to_json(v)))
                    .collect(),
            )
        }
    }
}

/// Convert a JSON value to a TOML Value.
#[cfg(feature = "toml")]
fn json_to_toml<T: Serialize>(value: &T) -> Result<toml::Value> {
    let json = serde_json::to_value(value).map_err(|e| ConfigError::json_parse(e.to_string()))?;
    json_to_toml_value(&json)
}

#[cfg(feature = "toml")]
fn json_to_toml_value(value: &serde_json::Value) -> Result<toml::Value> {
    match value {
        serde_json::Value::Null => Ok(toml::Value::String(String::new())),
        serde_json::Value::Bool(b) => Ok(toml::Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(toml::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(toml::Value::Float(f))
            } else {
                Ok(toml::Value::Integer(0))
            }
        }
        serde_json::Value::String(s) => Ok(toml::Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            Ok(toml::Value::Array(arr.iter().map(json_to_toml_value).collect::<Result<_>>()?))
        }
        serde_json::Value::Object(obj) => {
            let table: toml::map::Map<String, toml::Value> = obj
                .iter()
                .map(|(k, v)| Ok((k.clone(), json_to_toml_value(v)?)))
                .collect::<Result<_>>()?;
            Ok(toml::Value::Table(table))
        }
    }
}

#[cfg(not(feature = "toml"))]
fn json_to_toml<T: Serialize>(_value: &T) -> Result<toml::Value> {
    Err(ConfigError::custom("toml feature", "TOML feature not enabled"))
}

/// Convert YAML value to JSON Value.
#[cfg(feature = "yaml")]
fn yaml_to_json(value: &serde_yaml::Value) -> serde_json::Value {
    match value {
        serde_yaml::Value::Null => serde_json::Value::Null,
        serde_yaml::Value::Bool(b) => serde_json::Value::Bool(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::Value::Number(i.into())
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::Value::Null
            }
        }
        serde_yaml::Value::String(s) => serde_json::Value::String(s.clone()),
        serde_yaml::Value::Sequence(arr) => {
            serde_json::Value::Array(arr.iter().map(yaml_to_json).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            serde_json::Value::Object(
                map.iter()
                    .filter_map(|(k, v)| {
                        let key = match k {
                            serde_yaml::Value::String(s) => s.clone(),
                            serde_yaml::Value::Number(n) => n.to_string(),
                            _ => return None,
                        };
                        Some((key, yaml_to_json(v)))
                    })
                    .collect(),
            )
        }
        serde_yaml::Value::Tagged(tagged) => yaml_to_json(&tagged.value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_path() {
        assert_eq!(ConfigFormat::from_path("config.toml"), ConfigFormat::Toml);
        assert_eq!(ConfigFormat::from_path("config.json"), ConfigFormat::Json);
        assert_eq!(ConfigFormat::from_path("config.yaml"), ConfigFormat::Yaml);
        assert_eq!(ConfigFormat::from_path("config.yml"), ConfigFormat::Yaml);
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
        let json = format.parse_to_json(content).unwrap();
        let serialized = format.serialize(&json).unwrap();
        assert!(serialized.contains("test"));
    }
}
