// Evaluation context for policy evaluation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluationContext {
    facts: HashMap<String, serde_json::Value>,
}

impl EvaluationContext {
    pub fn new() -> Self { Self { facts: HashMap::new() } }
    pub fn from_map(facts: HashMap<String, serde_json::Value>) -> Self { Self { facts } }
    pub fn from_json(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Object(map) => Self { facts: map.into_iter().collect() },
            _ => Self::new(),
        }
    }
    pub fn set(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.facts.insert(key.into(), value);
    }
    pub fn set_string(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.facts.insert(key.into(), serde_json::Value::String(value.into()));
    }
    pub fn set_number(&mut self, key: impl Into<String>, value: f64) {
        if let Some(n) = serde_json::Number::from_f64(value) {
            self.facts.insert(key.into(), serde_json::Value::Number(n));
        }
    }
    pub fn set_bool(&mut self, key: impl Into<String>, value: bool) {
        self.facts.insert(key.into(), serde_json::Value::Bool(value));
    }
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> { self.facts.get(key) }
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.facts.get(key).and_then(|v| v.as_str().map(String::from))
    }
    pub fn get_number(&self, key: &str) -> Option<f64> { self.facts.get(key).and_then(|v| v.as_f64()) }
    pub fn get_bool(&self, key: &str) -> Option<bool> { self.facts.get(key).and_then(|v| v.as_bool()) }
    pub fn get_nested(&self, path: &str) -> Option<&serde_json::Value> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() { return None; }
        let mut current = self.facts.get(parts[0])?;
        for part in parts.iter().skip(1) {
            match current {
                serde_json::Value::Object(map) => { current = map.get(*part)?; }
                _ => return None,
            }
        }
        Some(current)
    }
    pub fn set_nested(&mut self, path: &str, value: serde_json::Value) {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() { return; }
        let root_key = parts[0].to_string();
        if !self.facts.contains_key(&root_key) {
            self.facts.insert(root_key.clone(), serde_json::Value::Object(Default::default()));
        }
        let mut current = self.facts.get_mut(&root_key).unwrap();
        for part in parts.iter().skip(1).take(parts.len().saturating_sub(2)) {
            if let serde_json::Value::Object(map) = current {
                if !map.contains_key(*part) {
                    map.insert(part.to_string(), serde_json::Value::Object(Default::default()));
                }
                current = map.get_mut(*part).unwrap();
            } else { return; }
        }
        if parts.len() > 1 {
            let final_key = parts[parts.len() - 1];
            if let serde_json::Value::Object(map) = current {
                map.insert(final_key.to_string(), value);
            }
        }
    }
    pub fn contains(&self, key: &str) -> bool { self.facts.contains_key(key) }
    pub fn facts(&self) -> &HashMap<String, serde_json::Value> { &self.facts }
    pub fn facts_mut(&mut self) -> &mut HashMap<String, serde_json::Value> { &mut self.facts }
    pub fn merge(&mut self, other: EvaluationContext) { self.facts.extend(other.facts); }
}

impl Default for EvaluationContext { fn default() -> Self { Self::new() } }

#[cfg(test)] mod tests {
    use super::*;

    #[test] fn test_from_json() {
        let json = serde_json::json!({ "name": "test", "value": 42 });
        let ctx = EvaluationContext::from_json(json);
        assert!(ctx.contains("name"));
        assert_eq!(ctx.get_number("value"), Some(42.0));
    }
    #[test] fn test_from_map() {
        let mut facts = HashMap::new();
        facts.insert("key".to_string(), serde_json::json!("val"));
        let ctx = EvaluationContext::from_map(facts);
        assert_eq!(ctx.get_string("key"), Some("val".to_string()));
    }
    #[test] fn test_from_map_empty() { assert!(EvaluationContext::from_map(HashMap::new()).facts().is_empty()); }
    #[test] fn test_set_and_get_string() {
        let mut ctx = EvaluationContext::new();
        ctx.set_string("name", "Alice");
        assert_eq!(ctx.get_string("name"), Some("Alice".to_string()));
    }
    #[test] fn test_set_and_get_number() {
        let mut ctx = EvaluationContext::new();
        ctx.set_number("age", 30.0);
        assert_eq!(ctx.get_number("age"), Some(30.0));
    }
    #[test] fn test_set_and_get_bool() {
        let mut ctx = EvaluationContext::new();
        ctx.set_bool("active", true);
        assert_eq!(ctx.get_bool("active"), Some(true));
    }
    #[test] fn test_contains() {
        let mut ctx = EvaluationContext::new();
        ctx.set_string("key", "value");
        assert!(ctx.contains("key"));
        assert!(!ctx.contains("missing"));
    }
    #[test] fn test_merge() {
        let mut ctx1 = EvaluationContext::new();
        ctx1.set_string("a", "1");
        let mut ctx2 = EvaluationContext::new();
        ctx2.set_string("b", "2");
        ctx1.merge(ctx2);
        assert_eq!(ctx1.get_string("a"), Some("1".to_string()));
        assert_eq!(ctx1.get_string("b"), Some("2".to_string()));
    }
    #[test] fn test_get_nested_simple() {
        let mut ctx = EvaluationContext::new();
        ctx.set("user", serde_json::json!({ "name": "Alice" }));
        assert_eq!(ctx.get_nested("user.name"), Some(&serde_json::json!("Alice")));
    }
    #[test] fn test_get_nested_deep() {
        let mut ctx = EvaluationContext::new();
        ctx.set("config", serde_json::json!({ "db": { "host": "localhost" } }));
        assert_eq!(ctx.get_nested("config.db.host"), Some(&serde_json::json!("localhost")));
    }
    #[test] fn test_get_nested_missing() { assert_eq!(EvaluationContext::new().get_nested("missing"), None); }
    #[test] fn test_get_nested_non_object() {
        let mut ctx = EvaluationContext::new();
        ctx.set_string("name", "Alice");
        assert_eq!(ctx.get_nested("name.invalid"), None);
    }
    #[test] fn test_get_nested_empty_path() { assert_eq!(EvaluationContext::new().get_nested(""), None); }
    #[test] fn test_set_nested_simple() {
        let mut ctx = EvaluationContext::new();
        ctx.set_nested("user.name", serde_json::json!("Alice"));
        assert_eq!(ctx.get_nested("user.name"), Some(&serde_json::json!("Alice")));
    }
    #[test] fn test_set_nested_creates_intermediate() {
        let mut ctx = EvaluationContext::new();
        ctx.set_nested("a.b.c", serde_json::json!(42));
        assert_eq!(ctx.get_nested("a.b.c"), Some(&serde_json::json!(42)));
    }
    #[test] fn test_set_nested_overwrites() {
        let mut ctx = EvaluationContext::new();
        ctx.set_nested("user.name", serde_json::json!("Alice"));
        ctx.set_nested("user.name", serde_json::json!("Bob"));
        assert_eq!(ctx.get_nested("user.name"), Some(&serde_json::json!("Bob")));
    }
}
