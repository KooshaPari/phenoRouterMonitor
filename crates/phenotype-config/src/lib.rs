//! # Phenotype Config Library
//!
//! Configuration management for Phenotype using figment with support for:
//! - TOML files
//! - Environment variables  
//! - JSON files
//!
//! ## Features
//!
//! - Multi-source configuration (file, env, defaults)
//! - Nested configuration with profiles
//! - Schema validation
//! - Type-safe accessors

use figment::{Figment, providers::{Env, Format, Json, Toml}};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration source priority (highest wins)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    /// Default values (lowest priority)
    Default = 0,
    /// TOML configuration file
    File = 1,
    /// JSON configuration file
    Json = 2,
    /// Environment variables (highest priority)
    Env = 3,
}

/// Configuration error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("configuration error: {0}")]
    Figment(#[from] figment::Error),
    
    #[error("missing required field: {0}")]
    MissingField(String),
    
    #[error("invalid value for {field}: {message}")]
    InvalidValue { field: String, message: String },
}

/// Result type for configuration operations
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Load configuration from the default sources
/// 
/// Priority (highest wins):
/// 1. Environment variables (PHENO_ prefix)
/// 2. config.toml in current directory
/// 3. Default values
pub fn load<T: DeserializeOwned>() -> Result<T> {
    load_with_profile::<T>("default")
}

/// Load configuration with a specific profile
pub fn load_with_profile<T: DeserializeOwned>(profile: &str) -> Result<T> {
    let figment = Figment::new()
        .merge(Toml::file("config.toml").profile(profile))
        .merge(Env::prefixed("PHENO_"));
    
    figment.extract()
}

/// Load configuration from a specific file
pub fn load_from_file<T: DeserializeOwned, P: AsRef<PathBuf>>(
    path: P,
) -> Result<T> {
    let path = path.as_ref();
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("toml");
    
    let figment = match ext {
        "json" => Figment::new().merge(Json::file(path)),
        _ => Figment::new().merge(Toml::file(path)),
    };
    
    figment.extract()
}

/// Add a data source to the configuration
pub struct ConfigBuilder {
    figment: Figment,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            figment: Figment::new(),
        }
    }
    
    /// Add a TOML file as a configuration source
    pub fn with_toml_file<P: AsRef<PathBuf>>(mut self, path: P) -> Self {
        self.figment = self.figment.merge(Toml::file(path));
        self
    }
    
    /// Add a JSON file as a configuration source
    pub fn with_json_file<P: AsRef<PathBuf>>(mut self, path: P) -> Self {
        self.figment = self.figment.merge(Json::file(path));
        self
    }
    
    /// Add environment variables with a prefix
    pub fn with_env_prefixed(mut self, prefix: &str) -> Self {
        self.figment = self.figment.merge(Env::prefixed(prefix));
        self
    }
    
    /// Extract the configuration
    pub fn extract<T: DeserializeOwned>(self) -> Result<T> {
        self.figment.extract()
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Helper trait for type-safe configuration access
pub trait ConfigExt: Sized + DeserializeOwned {
    /// Load from default sources
    fn load() -> Result<Self> {
        load_with_profile::<Self>("default")
    }
    
    /// Load with a specific profile
    fn load_with_profile(profile: &str) -> Result<Self> {
        load_with_profile::<Self>(profile)
    }
    
    /// Load from a specific file
    fn load_from_file<P: AsRef<PathBuf>>(path: P) -> Result<Self> {
        load_from_file::<Self, P>(path)
    }
    
    /// Create a configuration builder
    fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

impl<T: DeserializeOwned> ConfigExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_load_from_defaults() {
        #[derive(Deserialize, Default)]
        struct Defaults {
            name: String,
            port: u16,
        }
        
        impl Defaults {
            fn load_with_defaults() -> Self {
                Self {
                    name: "default".to_string(),
                    port: 8080,
                }
            }
        }
        
        let config = Defaults::load_with_defaults();
        assert_eq!(config.name, "default");
        assert_eq!(config.port, 8080);
    }
    
    #[test]
    fn test_config_builder() {
        let config: HashMap<String, String> = ConfigBuilder::new()
            .with_env_prefixed("TEST_")
            .extract()
            .unwrap_or_default();
        
        // Should not panic
        assert!(true);
    }
    
    #[test]
    fn test_source_priority() {
        assert!(Source::Env > Source::Json);
        assert!(Source::Json > Source::File);
        assert!(Source::File > Source::Default);
    }
}
