//! Configuration traits

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

/// Source of configuration value
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigSource {
    /// Default value, no configuration provided
    #[default]
    Default,
    /// Value from environment variable
    Environment,
    /// Value from configuration file
    File,
    /// Value from remote service
    Remote,
    /// Value from command line argument
    CommandLine,
}

/// Provider trait for configuration sources
#[async_trait]
pub trait Provider: Send + Sync {
    /// Type of keys used by this provider
    type Key;
    /// Type of values returned by this provider
    type Value;

    /// Get a value by key
    async fn get(&self, key: &Self::Key) -> Result<Option<Self::Value>, ConfigError>;

    /// Check if a key exists
    async fn contains(&self, key: &Self::Key) -> Result<bool, ConfigError>;

    /// Get all keys
    async fn keys(&self) -> Result<Vec<Self::Key>, ConfigError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_source_default() {
        let source = ConfigSource::Default;
        assert_eq!(source, ConfigSource::default());
    }

    #[test]
    fn test_config_source_variants() {
        assert!(matches!(ConfigSource::Default, ConfigSource::Default));
        assert!(matches!(
            ConfigSource::Environment,
            ConfigSource::Environment
        ));
        assert!(matches!(ConfigSource::File, ConfigSource::File));
        assert!(matches!(ConfigSource::Remote, ConfigSource::Remote));
        assert!(matches!(
            ConfigSource::CommandLine,
            ConfigSource::CommandLine
        ));
    }

    #[test]
    fn test_config_source_debug() {
        let source = ConfigSource::File;
        let debug_str = format!("{:?}", source);
        assert!(debug_str.contains("File"));
    }

    #[test]
    fn test_config_source_clone() {
        let source = ConfigSource::Environment;
        let cloned = source.clone();
        assert_eq!(source, cloned);
    }

    #[test]
    fn test_config_source_eq() {
        assert_eq!(ConfigSource::Default, ConfigSource::Default);
        assert_ne!(ConfigSource::Default, ConfigSource::Environment);
    }

    #[test]
    fn test_config_source_serialize() {
        let source = ConfigSource::File;
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, "\"file\"");
    }

    #[test]
    fn test_config_source_deserialize() {
        let json = "\"environment\"";
        let source: ConfigSource = serde_json::from_str(json).unwrap();
        assert_eq!(source, ConfigSource::Environment);
    }
}
