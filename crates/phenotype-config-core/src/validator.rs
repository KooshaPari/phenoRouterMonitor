//! Configuration validation

use crate::error::{ConfigError, Result};
use serde_json::Value;
use std::collections::HashSet;

/// Configuration validator for ensuring required keys and types
pub struct ConfigValidator {
    required_keys: HashSet<String>,
    type_checks: Vec<(String, ValueType)>,
}

/// Supported value types for validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    String,
    Number,
    Boolean,
    Object,
    Array,
}

impl ConfigValidator {
    /// Create a new validator
    pub fn new() -> Self {
        Self {
            required_keys: HashSet::new(),
            type_checks: Vec::new(),
        }
    }

    /// Add a required key
    pub fn require_key(mut self, key: impl Into<String>) -> Self {
        self.required_keys.insert(key.into());
        self
    }

    /// Add multiple required keys
    pub fn require_keys(mut self, keys: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for key in keys {
            self.required_keys.insert(key.into());
        }
        self
    }

    /// Add a type check for a key
    pub fn require_type(mut self, key: impl Into<String>, ty: ValueType) -> Self {
        self.type_checks.push((key.into(), ty));
        self
    }

    /// Validate configuration
    pub fn validate(&self, config: &Value) -> Result<()> {
        let obj = config.as_object().ok_or_else(|| {
            ConfigError::Validation("configuration must be an object".to_string())
        })?;

        // Check required keys
        for key in &self.required_keys {
            if !obj.contains_key(key) {
                return Err(ConfigError::MissingKey(key.clone()));
            }
        }

        // Check types
        for (key, expected_type) in &self.type_checks {
            if let Some(value) = obj.get(key) {
                let actual_type = get_value_type(value);
                if actual_type != *expected_type {
                    return Err(ConfigError::InvalidType {
                        key: key.clone(),
                        expected: format!("{:?}", expected_type),
                        actual: format!("{:?}", actual_type),
                    });
                }
            }
        }

        Ok(())
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to determine the type of a JSON value
fn get_value_type(value: &Value) -> ValueType {
    match value {
        Value::String(_) => ValueType::String,
        Value::Number(_) => ValueType::Number,
        Value::Bool(_) => ValueType::Boolean,
        Value::Object(_) => ValueType::Object,
        Value::Array(_) => ValueType::Array,
        Value::Null => ValueType::Object, // Treat null as object for leniency
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_required_keys() {
        let validator = ConfigValidator::new().require_key("port");

        let valid = serde_json::json!({"port": 3000});
        assert!(validator.validate(&valid).is_ok());

        let invalid = serde_json::json!({"host": "localhost"});
        assert!(validator.validate(&invalid).is_err());
    }

    #[test]
    fn test_validator_type_checks() {
        let validator = ConfigValidator::new()
            .require_type("port", ValueType::Number)
            .require_type("debug", ValueType::Boolean);

        let valid = serde_json::json!({"port": 3000, "debug": true});
        assert!(validator.validate(&valid).is_ok());

        let invalid_type = serde_json::json!({"port": "3000", "debug": true});
        assert!(validator.validate(&invalid_type).is_err());
    }
}
