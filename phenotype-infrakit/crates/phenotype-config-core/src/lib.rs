//! Core configuration types and traits for Phenotype
//!
//! This crate provides foundational configuration abstractions for the Phenotype ecosystem.
//!
//! # Example
//!
//! ```
//! use phenotype_config_core::{ConfigValue, ConfigSource};
//!
//! // Create a configuration value from environment
//! let value = ConfigValue::new("localhost:5432", ConfigSource::Environment);
//! assert_eq!(*value.inner(), "localhost:5432");
//! assert_eq!(value.source(), ConfigSource::Environment);
//! ```

use serde::{Deserialize, Serialize};

pub mod error;
pub mod traits;

pub use error::ConfigError;
pub use traits::{ConfigSource, Provider};

/// Configuration value with metadata
///
/// Stores both the value and its source (where the configuration came from).
///
/// # Example
///
/// ```
/// use phenotype_config_core::{ConfigValue, ConfigSource};
///
/// let value = ConfigValue::new(42, ConfigSource::File);
/// assert_eq!(*value.inner(), 42);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValue<T> {
    pub value: T,
    pub source: ConfigSource,
}

impl<T> ConfigValue<T> {
    /// Create a new ConfigValue with the given value and source
    pub fn new(value: T, source: ConfigSource) -> Self {
        Self { value, source }
    }

    /// Consume self and return the inner value
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Borrow the inner value
    pub fn inner(&self) -> &T {
        &self.value
    }

    /// Get the source of this configuration
    pub fn source(&self) -> ConfigSource {
        self.source.clone()
    }
}

impl<T: Default> Default for ConfigValue<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
            source: ConfigSource::Default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_value_new() {
        let value = ConfigValue::new(42, ConfigSource::Environment);
        assert_eq!(*value.inner(), 42);
        assert_eq!(value.source(), ConfigSource::Environment);
    }

    #[test]
    fn test_config_value_into_inner() {
        let value = ConfigValue::new(42, ConfigSource::Default);
        let inner = value.clone().into_inner();
        assert_eq!(inner, 42);
        assert_eq!(value.source(), ConfigSource::Default);
    }

    #[test]
    fn test_config_value_clone() {
        let original = ConfigValue::new("test".to_string(), ConfigSource::File);
        let cloned = original.clone();
        assert_eq!(original.inner(), cloned.inner());
        assert_eq!(original.source(), cloned.source());
    }

    #[test]
    fn test_config_value_default() {
        let value: ConfigValue<i32> = ConfigValue::default();
        assert_eq!(*value.inner(), 0);
        assert_eq!(value.source(), ConfigSource::Default);
    }

    #[test]
    fn test_config_value_string() {
        let value = ConfigValue::new("hello".to_string(), ConfigSource::CommandLine);
        assert_eq!(value.inner(), "hello");
        assert_eq!(value.source(), ConfigSource::CommandLine);
    }

    #[test]
    fn test_config_value_bool() {
        let value = ConfigValue::new(true, ConfigSource::Default);
        assert!(*value.inner());
        assert_eq!(value.source(), ConfigSource::Default);
    }

    #[test]
    fn test_config_value_serialize() {
        let value = ConfigValue::new(42, ConfigSource::File);
        let json = serde_json::to_string(&value).unwrap();
        assert!(json.contains("42"));
        assert!(json.contains("file"));
    }

    #[test]
    fn test_config_value_deserialize() {
        let json = r#"{"value":100,"source":"environment"}"#;
        let value: ConfigValue<i32> = serde_json::from_str(json).unwrap();
        assert_eq!(*value.inner(), 100);
        assert_eq!(value.source(), ConfigSource::Environment);
    }
}
