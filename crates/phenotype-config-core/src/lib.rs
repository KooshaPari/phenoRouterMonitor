//! # Phenotype Config Core
//!
//! Configuration management for Phenotype ecosystem.
//!
//! ## Features
//!
//! - **Environment-based**: Load config from environment variables
//! - **File-based**: Load from TOML, JSON, YAML files
//! - **Layered**: Merge multiple config sources with priority
//! - **Validation**: Validate config values at load time
//!
//! ## Usage
//!
//! ```rust
//! use phenotype_config_core::{Config, ConfigSource};
//!
//! let config = Config::builder()
//!     .with_source(ConfigSource::File("config.toml"))
//!     .with_source(ConfigSource::Env("APP_"))
//!     .build()?;
//! ```

pub mod builder;
pub mod error;
pub mod source;

pub use builder::ConfigBuilder;
pub use error::{ConfigError, Result};
pub use source::ConfigSource;

// Re-export commonly used types
pub use crate::error::ConfigErrorKind;

/// Configuration value container
#[derive(Debug, Clone)]
pub struct Config {
    /// Internal config storage
    data: serde_json::Map<String, serde_json::Value>,
}

impl Config {
    /// Create a new empty config
    pub fn new() -> Self {
        Self {
            data: serde_json::Map::new(),
        }
    }

    /// Get a string value
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.data.get(key)?.as_str().map(String::from)
    }

    /// Get an integer value
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.data.get(key)?.as_i64()
    }

    /// Get a boolean value
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key)?.as_bool()
    }

    /// Get a nested config section
    pub fn get_section(&self, key: &str) -> Option<Config> {
        self.data.get(key)?.as_object().map(|obj| Config {
            data: obj.clone(),
        })
    }

    /// Check if a key exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Get the number of keys
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get all keys
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.data.keys().map(String::as_str)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
