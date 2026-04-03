//! # Phenotype Shared Config
//!
//! Shared configuration types and utilities for Phenotype crates.
//! Provides a unified approach to configuration loading from multiple sources.

mod dirs;
mod error;
mod format;
mod source;

pub use dirs::{search_config_dirs, AppDirs, ConfigDir};
pub use error::{ConfigError, Result};
pub use format::{ConfigFormat, FormatDetect};
pub use source::{ConfigSource, ConfigValue, SourcePriority};

/// Configuration metadata.
#[derive(Debug, Clone)]
pub struct ConfigMeta {
    pub name: String,
    pub format: ConfigFormat,
    pub source: ConfigSource,
}

impl ConfigMeta {
    /// Create a new ConfigMeta.
    pub fn new(name: impl Into<String>, format: ConfigFormat, source: ConfigSource) -> Self {
        Self {
            name: name.into(),
            format,
            source,
        }
    }
}
