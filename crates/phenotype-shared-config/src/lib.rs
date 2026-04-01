//! # phenotype-shared-config
//!
//! Shared configuration types and utilities for the Phenotype ecosystem.
//!
//! This crate provides foundational types for configuration loading, validation,
//! and source management across Phenotype projects.
//!
//! ## Features
//!
//! - `yaml` - Enable YAML format support
//! - `toml` - Enable TOML format support (default)
//!
//! ## Modules
//!
//! - [`error`] - Structured error types for configuration operations
//! - [`format`] - Format detection and parsing (TOML, JSON, YAML)
//! - [`dirs`] - XDG-compliant directory resolution
//! - [`source`] - Configuration source tracking and priority-based merging
//!
//! ## Example
//!
//! ```rust
//! use phenotype_shared_config::{ConfigDirs, ConfigFormat};
//!
//! // Find config file
//! let dirs = ConfigDirs::new("myapp");
//! if let Ok(Some(path)) = dirs.find_config_file("config.toml") {
//!     // Load and parse
//!     let format = ConfigFormat::from_path(&path);
//!     // ...
//! }
//! ```

// Re-export commonly used types
pub use crate::dirs::ConfigDirs;
pub use crate::error::{ConfigError, Result as ConfigResult};
pub use crate::format::ConfigFormat;
pub use crate::source::{ConfigSource, ConfigValue, ConfigSet};

// Module declarations
mod dirs;
mod error;
mod format;
mod source;
