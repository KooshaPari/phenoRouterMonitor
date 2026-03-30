//! # Phenotype Config Loader
//!
//! Unified configuration loader for Phenotype using figment.
//!
//! This crate provides a comprehensive configuration loading system that supports:
//! - Environment variables (with prefix)
//! - TOML configuration files (with cascading search paths)
//! - Default values via serde defaults
//! - Type-safe configuration structs
//!
//! ## Example
//!
//! ```no_run
//! use phenotype_config_loader::AppConfigLoader;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct AppConfig {
//!     database: DatabaseConfig,
//!     cache: CacheConfig,
//!     server: ServerConfig,
//! }
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct DatabaseConfig {
//!     url: String,
//!     pool_size: u32,
//! }
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct CacheConfig {
//!     enabled: bool,
//!     ttl_secs: u64,
//! }
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct ServerConfig {
//!     host: String,
//!     port: u16,
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config: AppConfig = AppConfigLoader::new()
//!     .with_env_prefix("APP")
//!     .load()?;
//! # Ok(())
//! # }
//! ```

use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigLoaderError {
    #[error("figment error: {0}")]
    Figment(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("config not found at any search path")]
    NotFound,

    #[error("invalid configuration: {0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, ConfigLoaderError>;

/// Application configuration builder using figment
pub struct AppConfigLoader {
    env_prefix: Option<String>,
    search_paths: Vec<PathBuf>,
    config_name: String,
}

impl AppConfigLoader {
    /// Create a new config loader with default configuration name
    pub fn new() -> Self {
        Self {
            env_prefix: None,
            search_paths: Vec::new(),
            config_name: "config".to_string(),
        }
    }

    /// Set the configuration name (used for TOML file lookup)
    pub fn with_config_name(mut self, name: impl Into<String>) -> Self {
        self.config_name = name.into();
        self
    }

    /// Set environment variable prefix (e.g., "APP" for APP_DATABASE_URL)
    pub fn with_env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = Some(prefix.into());
        self
    }

    /// Add a custom search path for TOML files
    pub fn with_search_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.search_paths.push(path.into());
        self
    }

    /// Add multiple search paths
    pub fn with_search_paths(mut self, paths: impl IntoIterator<Item = PathBuf>) -> Self {
        self.search_paths.extend(paths);
        self
    }

    /// Load configuration from cascading sources (env vars > TOML files > defaults)
    pub fn load<T: DeserializeOwned>(self) -> Result<T> {
        let mut builder = figment::Figment::new();

        // Add search paths in reverse order (last one has highest priority)
        for path in &self.search_paths {
            if path.exists() {
                let content = std::fs::read_to_string(path)?;
                let parsed: serde_json::Value = toml::from_str(&content)?;
                builder = builder.merge(figment::providers::Serialized::defaults(parsed));
            }
        }

        // Add cascading default search paths
        let default_paths = self.default_search_paths();
        for path in default_paths {
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                let parsed: serde_json::Value = toml::from_str(&content)?;
                builder = builder.merge(figment::providers::Serialized::defaults(parsed));
            }
        }

        // Environment variables have highest priority
        if let Some(prefix) = self.env_prefix {
            builder = builder.merge(figment::providers::Env::prefixed(&prefix));
        } else {
            // Use environment provider without prefix
            builder = builder.merge(figment::providers::Env::raw());
        }

        // Extract the configuration
        builder
            .extract::<T>()
            .map_err(|e| ConfigLoaderError::Figment(e.to_string()))
    }

    /// Load configuration from a specific file path
    pub fn load_from_file<T: DeserializeOwned>(path: &Path) -> Result<T> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(ConfigLoaderError::Toml)
    }

    /// Get the default search paths in order of precedence (ascending)
    fn default_search_paths(&self) -> Vec<PathBuf> {
        let mut paths = vec![
            PathBuf::from(format!("{}.toml", self.config_name)),
        ];

        if let Some(config_dir) = dirs::config_dir() {
            paths.push(
                config_dir
                    .join("phenotype")
                    .join(format!("{}.toml", self.config_name)),
            );
        }

        paths.push(PathBuf::from(format!("/etc/phenotype/{}.toml", self.config_name)));

        paths
    }

    /// Get the list of paths that will be searched
    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }

    /// Get the environment prefix if set
    pub fn env_prefix(&self) -> Option<&str> {
        self.env_prefix.as_deref()
    }

    /// Get the configuration name
    pub fn config_name(&self) -> &str {
        &self.config_name
    }
}

impl Default for AppConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper struct for database configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_pool_size() -> u32 {
    10
}

fn default_timeout() -> u64 {
    30
}

/// Helper struct for cache configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_ttl")]
    pub ttl_secs: u64,
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
}

fn default_true() -> bool {
    true
}

fn default_ttl() -> u64 {
    3600
}

fn default_max_entries() -> usize {
    10000
}

/// Helper struct for server configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_workers")]
    pub worker_threads: usize,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_workers() -> usize {
    num_cpus::get()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::io::Write;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        port: u16,
        #[serde(default = "default_workers")]
        workers: usize,
    }

    #[test]
    fn test_load_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(
            f,
            "name = \"test-app\"\nport = 8080\nworkers = 4"
        )
        .unwrap();

        let config: TestConfig = AppConfigLoader::load_from_file(&path).unwrap();
        assert_eq!(config.name, "test-app");
        assert_eq!(config.port, 8080);
        assert_eq!(config.workers, 4);
    }

    #[test]
    fn test_default_values() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "name = \"test-app\"\nport = 3000").unwrap();

        let config: TestConfig = AppConfigLoader::load_from_file(&path).unwrap();
        assert_eq!(config.name, "test-app");
        assert_eq!(config.port, 3000);
        assert_eq!(config.workers, num_cpus::get()); // default value
    }

    #[test]
    fn test_database_config_defaults() {
        let _db = DatabaseConfig {
            url: "postgresql://localhost/db".to_string(),
            pool_size: 0,
            timeout_secs: 0,
        };
        assert_eq!(default_pool_size(), 10);
        assert_eq!(default_timeout(), 30);
    }

    #[test]
    fn test_cache_config_defaults() {
        let cache = CacheConfig {
            enabled: true,
            ttl_secs: default_ttl(),
            max_entries: default_max_entries(),
        };
        assert_eq!(cache.ttl_secs, 3600);
        assert_eq!(cache.max_entries, 10000);
    }

    #[test]
    fn test_server_config_defaults() {
        let server = ServerConfig {
            host: default_host(),
            port: default_port(),
            worker_threads: default_workers(),
        };
        assert_eq!(server.host, "0.0.0.0");
        assert_eq!(server.port, 8080);
        assert_eq!(server.worker_threads, num_cpus::get());
    }

    #[test]
    fn test_loader_with_config_name() {
        let loader = AppConfigLoader::new().with_config_name("myapp");
        assert_eq!(loader.config_name(), "myapp");
    }

    #[test]
    fn test_loader_with_env_prefix() {
        let loader = AppConfigLoader::new().with_env_prefix("MY_APP");
        assert_eq!(loader.env_prefix(), Some("MY_APP"));
    }

    #[test]
    fn test_loader_with_search_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let loader = AppConfigLoader::new().with_search_path(path.clone());
        assert_eq!(loader.search_paths(), &[path]);
    }

    #[test]
    fn test_load_with_custom_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("custom.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "name = \"custom-app\"\nport = 9000").unwrap();

        let loader = AppConfigLoader::new()
            .with_config_name("custom")
            .with_search_path(dir.path().join("custom.toml"));

        let config: TestConfig = loader.load().unwrap();
        assert_eq!(config.name, "custom-app");
        assert_eq!(config.port, 9000);
    }
}
