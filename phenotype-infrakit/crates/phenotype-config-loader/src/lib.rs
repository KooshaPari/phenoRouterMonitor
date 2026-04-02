//! Configuration loading for Phenotype
//!
//! Provides loaders for JSON and TOML configuration files.

mod error;
mod loader;

pub use error::LoaderError;
pub use loader::{FileLoader, JsonLoader, TomlLoader};

/// Trait for configuration file loaders
pub trait ConfigLoader: Send + Sync {
    /// Load configuration from a file path
    fn load_path(&self, path: &str) -> Result<serde_json::Value, LoaderError>;

    /// Load configuration from a string
    fn load_str(&self, content: &str) -> Result<serde_json::Value, LoaderError>;
}
