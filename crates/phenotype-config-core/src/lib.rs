//! Configuration management for Phenotype ecosystem.

pub mod builder;
pub mod error;
pub mod source;

pub use error::ConfigError;
pub use source::ConfigSource;

/// Configuration value container
#[derive(Debug, Clone, Default)]
pub struct Config {
    data: std::collections::HashMap<String, serde_json::Value>,
}

impl Config {
    /// Create a new empty config
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    /// Insert a value
    pub fn insert(&mut self, key: String, value: serde_json::Value) {
        self.data.insert(key, value);
    }

    /// Check if key exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Number of keys
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_config() {
        let config = Config::new();
        assert!(config.is_empty());
    }
}
