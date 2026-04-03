//! Core configuration types

use thiserror::Error;

/// Result type alias using ConfigError.
pub type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("config not found: {0}")]
    NotFound(String),
    #[error("{0}")]
    Other(String),
}

/// Loads configuration from this source as JSON value.
pub trait ConfigLoader {
    type Error: std::fmt::Debug + std::error::Error + From<std::io::Error> + From<serde_json::Error>;
    fn load<T: serde::de::DeserializeOwned>(&self) -> Result<T, Self::Error>;
}

#[derive(Debug, Clone)]
pub enum ConfigValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Object(std::collections::HashMap<String, ConfigValue>),
    Null,
}

impl ConfigValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_config_value_as_str() {
        let val = ConfigValue::String("test".to_string());
        assert_eq!(val.as_str(), Some("test"));

        let val = ConfigValue::Number(42.0);
        assert_eq!(val.as_str(), None);
    }

    #[test]
    fn test_config_value_nested() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), ConfigValue::String("value".to_string()));
        let val = ConfigValue::Object(map);

        match val {
            ConfigValue::Object(m) => {
                assert_eq!(m.get("key").unwrap().as_str(), Some("value"));
            }
            _ => panic!("Expected Object"),
        }
    }
}
