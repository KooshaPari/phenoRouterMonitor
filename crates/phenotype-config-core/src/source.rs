//! Configuration sources

use crate::error::{ConfigError, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Supported configuration file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// JSON format
    Json,
    /// TOML format
    Toml,
    /// YAML format
    Yaml,
}

impl ConfigFormat {
    /// Auto-detect format from file extension
    pub fn from_extension(path: &Path) -> Result<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| ConfigError::Parse("No file extension provided".to_string()))
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "json" => Ok(ConfigFormat::Json),
                "toml" => Ok(ConfigFormat::Toml),
                "yaml" | "yml" => Ok(ConfigFormat::Yaml),
                _ => Err(ConfigError::Parse(format!("Unsupported file format: {}", ext))),
            })
    }

    /// Parse configuration string in this format
    pub fn parse_str(&self, content: &str) -> Result<Value> {
        match self {
            ConfigFormat::Json => serde_json::from_str(content)
                .map_err(|e| ConfigError::Json(e)),
            ConfigFormat::Toml => toml::from_str(content)
                .map_err(|e| ConfigError::Toml(e))
                .and_then(|v: toml::Value| {
                    serde_json::to_value(v)
                        .map_err(|e| ConfigError::Json(e))
                }),
            ConfigFormat::Yaml => serde_yaml::from_str(content)
                .map_err(|e| ConfigError::Parse(format!("YAML parse error: {}", e)))
                .and_then(|v: serde_yaml::Value| {
                    serde_json::to_value(v)
                        .map_err(|e| ConfigError::Json(e))
                }),
        }
    }
}

/// Environment variable loader with prefix support
pub struct EnvLoader {
    prefix: Option<String>,
}

impl EnvLoader {
    /// Create a new environment loader
    pub fn new() -> Self {
        Self { prefix: None }
    }

    /// Set prefix for environment variables
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            prefix: Some(prefix.into()),
        }
    }

    /// Load configuration from environment variables
    pub fn load(&self) -> Result<Value> {
        let mut map = serde_json::Map::new();

        for (key, value) in std::env::vars() {
            // Check prefix if set
            if let Some(ref prefix) = self.prefix {
                if !key.starts_with(prefix) {
                    continue;
                }
            }

            // Strip prefix from key
            let key = if let Some(ref prefix) = self.prefix {
                key.strip_prefix(prefix)
                    .unwrap_or(&key)
                    .to_string()
            } else {
                key
            };

            // Convert to lowercase for consistency
            let key = key.to_lowercase();
            map.insert(key, Value::String(value));
        }

        Ok(Value::Object(map))
    }
}

impl Default for EnvLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// File-based configuration loader
pub struct FileLoader {
    path: PathBuf,
    format: Option<ConfigFormat>,
}

impl FileLoader {
    /// Create a new file loader
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            format: None,
        }
    }

    /// Set explicit format (auto-detect if not set)
    pub fn with_format(mut self, format: ConfigFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Load configuration from file
    pub fn load(&self) -> Result<Value> {
        if !self.path.exists() {
            return Err(ConfigError::FileNotFound(self.path.display().to_string()));
        }

        let content = fs::read_to_string(&self.path)?;
        let format = self.format
            .ok_or_else(|| ConfigFormat::from_extension(&self.path))?;

        format.parse_str(&content)
    }
}

/// In-memory configuration source
pub struct InlineLoader {
    data: HashMap<String, Value>,
}

impl InlineLoader {
    /// Create a new inline loader
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Insert a key-value pair
    pub fn insert(mut self, key: impl Into<String>, value: Value) -> Self {
        self.data.insert(key.into(), value);
        self
    }

    /// Load configuration
    pub fn load(&self) -> Result<Value> {
        let mut map = serde_json::Map::new();
        for (key, value) in &self.data {
            map.insert(key.clone(), value.clone());
        }
        Ok(Value::Object(map))
    }
}

impl Default for InlineLoader {
    fn default() -> Self {
        Self::new()
    }
}
