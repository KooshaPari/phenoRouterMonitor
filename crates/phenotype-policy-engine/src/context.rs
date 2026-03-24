//! Evaluation context for policy evaluation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A context containing key-value pairs (facts) used for policy evaluation.
///
/// The EvaluationContext is a flexible key-value map that holds domain-specific
/// facts about the entity being evaluated. Policies evaluate rules against these facts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluationContext {
    /// Key-value pairs representing facts about the entity.
    facts: HashMap<String, serde_json::Value>,
}

impl EvaluationContext {
    /// Creates a new empty evaluation context.
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
        }
    }

    /// Creates an evaluation context from a HashMap.
    pub fn from_map(facts: HashMap<String, serde_json::Value>) -> Self {
        Self { facts }
    }

    /// Creates an evaluation context from a JSON value.
    pub fn from_json(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Object(map) => {
                let facts = map.into_iter().collect();
                Self { facts }
            }
            _ => Self::new(),
        }
    }

    /// Sets a fact in the context.
    pub fn set(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.facts.insert(key.into(), value);
    }

    /// Sets a string fact.
    pub fn set_string(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.facts
            .insert(key.into(), serde_json::Value::String(value.into()));
    }

    /// Sets a numeric fact.
    pub fn set_number(&mut self, key: impl Into<String>, value: f64) {
        if let Some(n) = serde_json::Number::from_f64(value) {
            self.facts.insert(key.into(), serde_json::Value::Number(n));
        }
    }

    /// Sets a boolean fact.
    pub fn set_bool(&mut self, key: impl Into<String>, value: bool) {
        self.facts
            .insert(key.into(), serde_json::Value::Bool(value));
    }

    /// Gets a fact from the context.
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.facts.get(key)
    }

    /// Gets a fact as a string.
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.facts
            .get(key)
            .and_then(|v| v.as_str().map(String::from))
    }

    /// Gets a fact as a number.
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.facts.get(key).and_then(|v| v.as_f64())
    }

    /// Gets a fact as a boolean.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.facts.get(key).and_then(|v| v.as_bool())
    }

    /// Checks if a fact exists.
    pub fn contains(&self, key: &str) -> bool {
        self.facts.contains_key(key)
    }

    /// Gets all facts.
    pub fn facts(&self) -> &HashMap<String, serde_json::Value> {
        &self.facts
    }

    /// Gets a mutable reference to all facts.
    pub fn facts_mut(&mut self) -> &mut HashMap<String, serde_json::Value> {
        &mut self.facts
    }

    /// Merges another context into this one.
    pub fn merge(&mut self, other: EvaluationContext) {
        self.facts.extend(other.facts);
    }
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_context_empty() {
        let ctx = EvaluationContext::new();
        assert!(ctx.facts().is_empty());
    }

    #[test]
    fn test_set_and_get_string() {
        let mut ctx = EvaluationContext::new();
        ctx.set_string("name", "Alice");
        assert_eq!(ctx.get_string("name"), Some("Alice".to_string()));
    }

    #[test]
    fn test_set_and_get_number() {
        let mut ctx = EvaluationContext::new();
        ctx.set_number("age", 30.0);
        assert_eq!(ctx.get_number("age"), Some(30.0));
    }

    #[test]
    fn test_set_and_get_bool() {
        let mut ctx = EvaluationContext::new();
        ctx.set_bool("active", true);
        assert_eq!(ctx.get_bool("active"), Some(true));
    }

    #[test]
    fn test_contains() {
        let mut ctx = EvaluationContext::new();
        ctx.set_string("key", "value");
        assert!(ctx.contains("key"));
        assert!(!ctx.contains("missing"));
    }

    #[test]
    fn test_merge() {
        let mut ctx1 = EvaluationContext::new();
        ctx1.set_string("a", "1");

        let mut ctx2 = EvaluationContext::new();
        ctx2.set_string("b", "2");

        ctx1.merge(ctx2);
        assert_eq!(ctx1.get_string("a"), Some("1".to_string()));
        assert_eq!(ctx1.get_string("b"), Some("2".to_string()));
    }
}
