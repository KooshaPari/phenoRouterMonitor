//! Configuration source types and priorities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Priority levels for configuration sources.
/// Higher priority values override lower ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum SourcePriority {
    /// Default values built into the application.
    Default = 0,
    /// Environment variables.
    Env = 10,
    /// Local configuration files (e.g., `.config.toml`).
    Local = 20,
    /// User-specific configuration.
    User = 30,
    /// System-wide configuration.
    System = 40,
}

/// Source of configuration values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigSource {
    /// Configuration from the environment.
    Env,
    /// Configuration from a file.
    File,
    /// Configuration from default values.
    Default,
    /// Configuration from arguments.
    Args,
}

impl ConfigSource {
    /// Get the default priority for this source type.
    pub fn default_priority(self) -> SourcePriority {
        match self {
            Self::Default => SourcePriority::Default,
            Self::Env => SourcePriority::Env,
            Self::File => SourcePriority::Local,
            Self::Args => SourcePriority::User,
        }
    }
}

/// A configuration value with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValue {
    /// The source of this configuration value.
    pub source: ConfigSource,
    /// Priority of this value (higher overrides lower).
    pub priority: SourcePriority,
    /// The actual value.
    #[serde(flatten)]
    pub value: serde_json::Value,
}

impl ConfigValue {
    /// Create a new ConfigValue.
    pub fn new(source: ConfigSource, value: serde_json::Value) -> Self {
        Self {
            source,
            priority: source.default_priority(),
            value,
        }
    }

    /// Create with custom priority.
    pub fn with_priority(source: ConfigSource, priority: SourcePriority, value: serde_json::Value) -> Self {
        Self {
            source,
            priority,
            value,
        }
    }
}

/// Merges multiple configuration values according to priority.
pub fn merge_configs(configs: Vec<ConfigValue>) -> HashMap<String, serde_json::Value> {
    let mut result: HashMap<String, serde_json::Value> = HashMap::new();

    // Sort by priority (low to high)
    let mut sorted = configs;
    sorted.sort_by_key(|c| c.priority);

    // Higher priority values override lower ones
    for config in sorted {
        if let serde_json::Value::Object(map) = config.value {
            for (key, value) in map {
                result.insert(key, value);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_priority() {
        assert!(SourcePriority::System > SourcePriority::User);
        assert!(SourcePriority::User > SourcePriority::Local);
        assert!(SourcePriority::Local > SourcePriority::Env);
        assert!(SourcePriority::Env > SourcePriority::Default);
    }

    #[test]
    fn test_config_value() {
        let value = ConfigValue::new(ConfigSource::File, serde_json::json!({"key": "value"}));
        assert_eq!(value.source, ConfigSource::File);
        assert_eq!(value.priority, SourcePriority::Local);
    }

    #[test]
    fn test_merge_override() {
        let configs = vec![
            ConfigValue::with_priority(
                ConfigSource::Default,
                SourcePriority::Default,
                serde_json::json!({"host": "localhost", "port": 8080}),
            ),
            ConfigValue::with_priority(
                ConfigSource::File,
                SourcePriority::Local,
                serde_json::json!({"port": 9000, "debug": true}),
            ),
        ];

        let merged = merge_configs(configs);
        assert_eq!(merged.get("host"), Some(&serde_json::json!("localhost")));
        assert_eq!(merged.get("port"), Some(&serde_json::json!(9000)));
        assert_eq!(merged.get("debug"), Some(&serde_json::json!(true)));
    }

    #[test]
    fn test_deep_merge() {
        // For simple values, higher priority wins
        let configs = vec![
            ConfigValue::with_priority(
                ConfigSource::Default,
                SourcePriority::Default,
                serde_json::json!({"key": "old"}),
            ),
            ConfigValue::with_priority(
                ConfigSource::Env,
                SourcePriority::Env,
                serde_json::json!({"key": "new"}),
            ),
        ];

        let merged = merge_configs(configs);
        assert_eq!(merged.get("key"), Some(&serde_json::json!("new")));
    }
}
