//! Config builder module

use crate::source::{ConfigFormat, EnvLoader, FileLoader, InlineLoader};
use crate::{Config, Result};
use serde_json::Value;
use std::path::Path;

/// Builder for constructing Config with multiple sources
/// Supports override chain: environment > file > defaults
#[derive(Debug)]
pub struct ConfigBuilder {
    loaders: Vec<Box<dyn ConfigLoaderFn>>,
}

/// Trait for config loader functions
trait ConfigLoaderFn: Send + Sync {
    fn load(&self) -> Result<Value>;
}

/// Wrapper for EnvLoader
struct EnvLoaderWrapper(EnvLoader);

impl ConfigLoaderFn for EnvLoaderWrapper {
    fn load(&self) -> Result<Value> {
        self.0.load()
    }
}

/// Wrapper for FileLoader
struct FileLoaderWrapper(FileLoader);

impl ConfigLoaderFn for FileLoaderWrapper {
    fn load(&self) -> Result<Value> {
        self.0.load()
    }
}

/// Wrapper for InlineLoader
struct InlineLoaderWrapper(InlineLoader);

impl ConfigLoaderFn for InlineLoaderWrapper {
    fn load(&self) -> Result<Value> {
        self.0.load()
    }
}

impl ConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            loaders: Vec::new(),
        }
    }

    /// Add a file configuration source (auto-detects format from extension)
    pub fn with_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        let loader = FileLoader::new(path);
        self.loaders.push(Box::new(FileLoaderWrapper(loader)));
        self
    }

    /// Add a file configuration source with explicit format
    pub fn with_file_format<P: AsRef<Path>>(mut self, path: P, format: ConfigFormat) -> Self {
        let loader = FileLoader::new(path).with_format(format);
        self.loaders.push(Box::new(FileLoaderWrapper(loader)));
        self
    }

    /// Add environment variables with optional prefix
    pub fn with_env(mut self) -> Self {
        let loader = EnvLoader::new();
        self.loaders.push(Box::new(EnvLoaderWrapper(loader)));
        self
    }

    /// Add environment variables with prefix (e.g., "APP_")
    pub fn with_env_prefix<P: Into<String>>(mut self, prefix: P) -> Self {
        let loader = EnvLoader::with_prefix(prefix);
        self.loaders.push(Box::new(EnvLoaderWrapper(loader)));
        self
    }

    /// Add inline configuration
    pub fn with_inline_value(mut self, key: impl Into<String>, value: Value) -> Self {
        let loader = InlineLoader::new().insert(key, value);
        self.loaders.push(Box::new(InlineLoaderWrapper(loader)));
        self
    }

    /// Build the final configuration with override chain
    /// Later loaders override earlier loaders
    pub fn build(self) -> Result<Config> {
        let mut config = Config::new();

        // Load and merge all sources (later sources override earlier)
        for loader in self.loaders {
            let value = loader.load()?;
            if let Value::Object(obj) = value {
                for (key, val) in obj {
                    config.data.insert(key, val);
                }
            }
        }

        Ok(config)
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
