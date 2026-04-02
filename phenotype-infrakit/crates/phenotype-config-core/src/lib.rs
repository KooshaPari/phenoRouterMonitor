//! Phenotype configuration core module
//!
//! Provides foundational types for configuration management.

pub mod error;
pub mod types;

pub use error::{ConfigError, Result};
pub use types::{Config, ConfigSource, ConfigValue};
