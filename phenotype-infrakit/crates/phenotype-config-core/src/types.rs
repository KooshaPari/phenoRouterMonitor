//! Configuration types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Array value
    Array(Vec<ConfigValue>),
    /// Object value
    Object(HashMap<String, ConfigValue>),
    /// Null value
    Null,
}

impl ConfigValue {
    /// Get a string value
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get an integer value
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ConfigValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Get a boolean value
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

/// Configuration source
#[derive(Debug, Clone)]
pub enum ConfigSource {
    /// Environment variable
    Env(String),
    /// File path
    File(String),
    /// Command line argument
    Arg(String),
    /// Default value
    Default,
    /// Programmatically set
    Programmatic,
}
