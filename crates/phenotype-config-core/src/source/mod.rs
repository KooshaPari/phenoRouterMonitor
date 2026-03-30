//! Config source module

use serde_json::Value;

/// Configuration source trait
pub trait ConfigSource {
    fn get(&self, key: &str) -> Option<Value>;
}
