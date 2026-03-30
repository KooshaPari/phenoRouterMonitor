//! Configuration sources

use crate::error::{ConfigError, Result};
use serde::de::DeserializeOwned;
use std::path::Path;
use std::sync::Arc;

/// Configuration source trait
pub trait ConfigSource: Send + Sync {
    /// Load configuration from this source
    fn load<T: DeserializeOwned + Send + Sync>(&self) -> Result<T>;

    /// Source name for error messages
    fn name(&self) -> &str;
}

/// File configuration source
pub struct FileSource {
    path: Arc<Path>,
    format: ConfigFormat,
}

impl FileSource {
    /// Create a new file source
    pub fn new(path: impl AsRef<Path>, format: ConfigFormat) -> Self {
        Self {
            path: Arc::new(path.as_ref().to_path_buf()),
            format,
        }
    }
}

impl ConfigSource for FileSource {
    fn load<T: DeserializeOwned + Send + Sync>(&self) -> Result<T> {
        if !self.path.exists() {
            return Err(ConfigError::FileNotFound(self.path.display().to_string()));
        }
        let content = std::fs::read_to_string(&*self.path)?;
        self.format.parse(&content)
    }

    fn name(&self) -> &str {
        self.path.display().to_string().as_str()
    }
}

/// String configuration source
pub struct StringSource {
    content: String,
    format: ConfigFormat,
    name: String,
}

impl StringSource {
    /// Create from TOML string
    pub fn toml(content: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            format: ConfigFormat::Toml,
            name: name.into(),
        }
    }

    /// Create from JSON string
    pub fn json(content: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            format: ConfigFormat::Json,
            name: name.into(),
        }
    }
}

impl ConfigSource for StringSource {
    fn load<T: DeserializeOwned + Send + Sync>(&self) -> Result<T> {
        self.format.parse(&self.content)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Environment variable configuration source
#[derive(Debug, Clone, Default)]
pub struct EnvSource {
    prefix: Option<String>,
}

impl EnvSource {
    /// Create with a prefix (e.g., "APP_" for APP_DEBUG, APP_PORT)
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            prefix: Some(prefix.into()),
        }
    }

    /// Create without a prefix
    pub fn new() -> Self {
        Self::default()
    }
}

impl ConfigSource for EnvSource {
    fn load<T: DeserializeOwned + Send + Sync>(&self) -> Result<T> {
        // For env vars, we create a flat map of KEY=VALUE
        let map: std::collections::HashMap<String, String> = std::env::vars()
            .filter(|(key, _)| {
                if let Some(ref prefix) = self.prefix {
                    key.starts_with(prefix)
                } else {
                    true
                }
            })
            .map(|(key, value)| {
                // Strip prefix if present
                let key = if let Some(ref prefix) = self.prefix {
                    key.strip_prefix(prefix).unwrap_or(&key).to_string()
                } else {
                    key
                };
                // Convert to SCREAMING_SNAKE_CASE to camelCase
                let key = key.to_lowercase();
                (key, value)
            })
            .collect();

        serde_json::from_value(serde_json::to_value(map).map_err(|e| ConfigError::Json(e))?)
            .map_err(|e| ConfigError::Parse(e.to_string()))
    }

    fn name(&self) -> &str {
        "environment"
    }
}
