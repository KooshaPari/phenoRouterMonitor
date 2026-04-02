//! Configuration loader implementation

use crate::error::{ConfigError, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// Configuration loader for JSON, TOML, YAML formats
#[derive(Debug, Clone)]
pub struct ConfigLoader {
    data: Value,
}

impl ConfigLoader {
    /// Create a new loader from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        let data: Value = serde_json::from_str(json)?;
        Ok(Self { data })
    }

    /// Create a new loader from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let ext = path.as_ref().extension().and_then(|e| e.to_str()).unwrap_or("");

        let data = match ext {
            "json" => serde_json::from_str(&content)?,
            "toml" => {
                let v: toml::Value = content.parse()?;
                serde_json::to_value(v)?
            }
            "yaml" | "yml" => serde_yaml::from_str(&content)?,
            _ => return Err(ConfigError::Invalid(format!("unsupported format: {}", ext)),
        };

        Ok(Self { data })
    }

    /// Get a value by key path (e.g., "database.host")
    pub fn get(&self, key: &str) -> Result<Value> {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = &self.data;

        for part in parts {
            current = match current {
                Value::Object(map) => map.get(part).unwrap_or(&Value::Null),
                _ => return Err(ConfigError::KeyNotFound(key.to_string())),
            };
        }

        if current.is_null() {
            Err(ConfigError::KeyNotFound(key.to_string()))
        } else {
            Ok(current.clone())
        }
    }

    /// Get a typed value by key path
    pub fn get_typed<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        let value = self.get(key)?;
        serde_json::from_value(value).map_err(|e| ConfigError::Invalid(e.to_string()))
    }

    /// Check if a key exists
    pub fn contains(&self, key: &str) -> bool {
        self.get(key).is_ok()
    }

    /// Get all keys at the root level
    pub fn keys(&self) -> Vec<String> {
        match &self.data {
            Value::Object(map) => map.keys().cloned().collect(),
            _ => vec![],
        }
    }

    /// Merge another configuration
    pub fn merge(&mut self, other: ConfigLoader) {
        self.data = Self::merge_values(self.data.clone(), other.data);
    }

    fn merge_values(base: Value, override_: Value) -> Value {
        match (base, override_) {
            (Value::Object(mut base_map), Value::Object(override_map)) => {
                for (key, value) in override_map {
                    let merged = if let Some(base_value) = base_map.remove(&key) {
                        Self::merge_values(base_value, value)
                    } else {
                        value
                    };
                    base_map.insert(key, merged);
                }
                Value::Object(base_map)
            }
            (_, override_) => override_,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-CFG-001
    #[test]
    fn test_load_json() {
        let loader = ConfigLoader::from_json(r#"{"key": "value"}"#).unwrap();
        assert_eq!(loader.get("key").unwrap(), "value");
    }

    // Traces to: FR-CFG-002
    #[test]
    fn test_nested_keys() {
        let loader = ConfigLoader::from_json(r#"{"a": {"b": {"c": 1}}"#).unwrap();
        assert_eq!(loader.get("a.b.c").unwrap(), 1);
    }
}
