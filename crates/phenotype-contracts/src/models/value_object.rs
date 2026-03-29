//! # Value Object Module
//!
//! Immutable value types with value-based equality.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A base value object with value-based equality.
///
/// Value objects are immutable domain objects that are equal
/// if all their attributes are equal (not by identity).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueObject {
    /// Serialized attributes (flexible key-value).
    pub attributes: serde_json::Value,
}

impl ValueObject {
    /// Create a new value object from JSON attributes.
    pub fn new(attributes: serde_json::Value) -> Self {
        Self { attributes }
    }

    /// Create from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        Ok(Self::new(serde_json::from_str(json)?))
    }

    /// Get an attribute by key.
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.attributes.get(key)
    }

    /// Serialize to a JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.attributes).unwrap_or_default()
    }

    /// Get a reference to the underlying attributes.
    pub fn attributes(&self) -> &serde_json::Value {
        &self.attributes
    }
}

impl PartialEq for ValueObject {
    fn eq(&self, other: &Self) -> bool {
        self.attributes == other.attributes
    }
}

impl Eq for ValueObject {}

impl fmt::Display for ValueObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ValueObject({})", self.attributes)
    }
}

impl Default for ValueObject {
    fn default() -> Self {
        Self::new(serde_json::json!({}))
    }
}
