//! Unified configuration loading and management for the Phenotype ecosystem.
//!
//! This library provides a consistent, reusable interface for loading configuration
//! from multiple formats (TOML, YAML, JSON) with standard error handling and validation.
//!
//! ## Features
//!
//! - **Multi-format support**: TOML, YAML, and JSON configuration files
//! - **Standard configuration locations**: XDG directories, system paths
//! - **Type-safe deserialization**: via serde
//! - **Comprehensive error handling**: detailed, actionable error messages
//! - **Directory detection**: automatic platform-aware config directory resolution
//!
//! ## Examples
//!
//! ```no_run
//! use phenotype_config_core::{ConfigLoader, ConfigFormat};
//! use serde::{Deserialize, Serialize};
//! use std::path::Path;
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct AppConfig {
//!     app_name: String,
//!     version: String,
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let loader = ConfigLoader::new();
//!
//!     // Load from specific file
//!     let config: AppConfig = loader.load_from_file("config.toml")?;
//!
//!     // Or search standard locations
//!     let config: AppConfig = loader
//!         .search_default_locations("myapp", "config.toml")?
//!         .ok_or("Config not found")?;
//!
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod loader;
pub mod format;
pub mod dirs_helper;

pub use error::{ConfigError, Result};
pub use loader::ConfigLoader;
pub use format::ConfigFormat;
pub use dirs_helper::ConfigDirs;

/// Re-export commonly used types from serde_json and serde_yaml
pub use serde_json::json;
