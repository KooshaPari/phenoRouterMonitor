//! # Phenotype Config Core
//!
//! Configuration management for Phenotype ecosystem with support for multiple sources,
//! formats, and validation.
//!
//! ## Features
//!
//! - **Multi-format**: Load from JSON, TOML, YAML files
//! - **Environment variables**: Load and override with env vars (with optional prefix)
//! - **Inline config**: Add default values programmatically
//! - **Override chain**: Environment > File > Defaults (later sources override earlier)
//! - **Type-safe access**: Get values with automatic type conversion
//! - **Validation**: Validate config structure at load time
//!
//! ## Usage
//!
//! ```rust,ignore
//! use phenotype_config_core::ConfigBuilder;
//!
//! // Build config with override chain
//! let config = ConfigBuilder::new()
//!     .with_inline_value("port", serde_json::json!(3000))
//!     .with_file("config.toml")
//!     .with_env_prefix("APP_")
//!     .build()?;
//!
//! // Access values
//! let port = config.get_i64("port").unwrap_or(3000);
//! let debug = config.get_bool("debug").unwrap_or(false);
//! ```

pub mod builder;
pub mod error;
pub mod source;
pub mod validator;

pub use builder::ConfigBuilder;
pub use error::{ConfigError, ConfigErrorKind, Result};
pub use source::{ConfigFormat, EnvLoader, FileLoader, InlineLoader};

/// Configuration value container with type-safe access
#[derive(Debug, Clone)]
pub struct Config {
    /// Internal config storage using JSON Value as universal format
    pub(crate) data: serde_json::Map<String, serde_json::Value>,
}

impl Config {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self {
            data: serde_json::Map::new(),
        }
    }

    /// Create a builder for fluent config construction
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    /// Get a string value, returning None if key doesn't exist or value is not a string
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.data.get(key)?.as_str().map(String::from)
    }

    /// Get a required string value, returning an error if missing or wrong type
    pub fn get_string_required(&self, key: &str) -> Result<String> {
        self.get_string(key).ok_or_else(|| {
            ConfigError::MissingKey(format!("string key '{}' not found", key))
        })
    }

    /// Get an integer value
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.data.get(key)?.as_i64()
    }

    /// Get a required integer value
    pub fn get_i64_required(&self, key: &str) -> Result<i64> {
        self.get_i64(key).ok_or_else(|| {
            ConfigError::MissingKey(format!("integer key '{}' not found", key))
        })
    }

    /// Get a boolean value
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key)?.as_bool()
    }

    /// Get a required boolean value
    pub fn get_bool_required(&self, key: &str) -> Result<bool> {
        self.get_bool(key).ok_or_else(|| {
            ConfigError::MissingKey(format!("boolean key '{}' not found", key))
        })
    }

    /// Get a nested configuration section
    pub fn get_section(&self, key: &str) -> Option<Config> {
        self.data.get(key)?.as_object().map(|obj| Config {
            data: obj.clone(),
        })
    }

    /// Get a generic JSON value
    pub fn get_value(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    /// Get a mutable reference to a generic JSON value
    pub fn get_value_mut(&mut self, key: &str) -> Option<&mut serde_json::Value> {
        self.data.get_mut(key)
    }

    /// Set a value directly
    pub fn set_value(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.data.insert(key.into(), value);
    }

    /// Check if a key exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Get the number of keys
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if configuration is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get all keys as an iterator
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.data.keys().map(String::as_str)
    }

    /// Get underlying data as JSON object
    pub fn as_json_object(&self) -> &serde_json::Map<String, serde_json::Value> {
        &self.data
    }

    /// Get mutable reference to underlying data
    pub fn as_json_object_mut(&mut self) -> &mut serde_json::Map<String, serde_json::Value> {
        &mut self.data
    }

    /// Convert to JSON Value
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::Value::Object(self.data.clone())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl AsRef<serde_json::Map<String, serde_json::Value>> for Config {
    fn as_ref(&self) -> &serde_json::Map<String, serde_json::Value> {
        &self.data
    }
}
