//! Configuration source types and utilities.
//!
//! This module provides the foundational types for tracking configuration sources
//! and their priority-based merging.

use serde::{Deserialize, Serialize};

/// Priority ordering for config sources (higher = higher priority).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigSource {
    System = 1,
    User = 2,
    Project = 3,
    Env = 4,
    Inline = 5,
}

impl std::fmt::Display for ConfigSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigSource::System => write!(f, "system"),
            ConfigSource::User => write!(f, "user"),
            ConfigSource::Project => write!(f, "project"),
            ConfigSource::Env => write!(f, "env"),
            ConfigSource::Inline => write!(f, "inline"),
        }
    }
}

/// A value with its source tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValue {
    pub value: serde_json::Value,
    pub source: ConfigSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl ConfigValue {
    pub fn new(value: impl Into<serde_json::Value>, source: ConfigSource) -> Self {
        Self { value: value.into(), source, path: None }
    }

    pub fn from_object(map: serde_json::Map<String, serde_json::Value>, source: ConfigSource) -> Self {
        Self::new(serde_json::Value::Object(map), source)
    }

    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}

/// A collection of config values from multiple sources.
#[derive(Debug, Clone, Default)]
pub struct ConfigSet {
    values: Vec<ConfigValue>,
}

impl ConfigSet {
    pub fn new() -> Self { Self::default() }

    pub fn add(&mut self, value: ConfigValue) { self.values.push(value); }

    pub fn add_from(&mut self, value: impl Into<serde_json::Value>, source: ConfigSource) {
        self.values.push(ConfigValue::new(value, source));
    }

    pub fn merge(&self) -> serde_json::Value {
        let mut sorted: Vec<_> = self.values.clone();
        sorted.sort_by_key(|v| v.source);
        let mut result = serde_json::Value::Object(serde_json::Map::new());
        for config_value in sorted {
            Self::deep_merge(&mut result, config_value.value);
        }
        result
    }

    fn deep_merge(target: &mut serde_json::Value, source: serde_json::Value) {
        match (target, source) {
            (target @ serde_json::Value::Object(_), serde_json::Value::Object(source_obj)) => {
                let target_obj = target.as_object_mut().unwrap();
                for (key, value) in source_obj {
                    if target_obj.contains_key(key.as_str()) {
                        Self::deep_merge(target_obj.get_mut(key.as_str()).unwrap(), value.clone());
                    } else {
                        target_obj.insert(key.clone(), value);
                    }
                }
            }
            (target, source) => *target = source,
        }
    }

    pub fn highest_source(&self) -> Option<ConfigSource> {
        self.values.iter().map(|v| v.source).max()
    }

    pub fn from_source(&self, source: ConfigSource) -> Vec<&ConfigValue> {
        self.values.iter().filter(|v| v.source == source).collect()
    }
}

impl From<Vec<ConfigValue>> for ConfigSet {
    fn from(values: Vec<ConfigValue>) -> Self {
        let mut set = Self::new();
        for value in values { set.add(value); }
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_priority() {
        assert!(ConfigSource::Inline > ConfigSource::Env);
        assert!(ConfigSource::Env > ConfigSource::Project);
        assert!(ConfigSource::Project > ConfigSource::User);
        assert!(ConfigSource::User > ConfigSource::System);
    }

    #[test]
    fn test_config_value() {
        let value = ConfigValue::new(serde_json::json!({"key": "value"}), ConfigSource::User);
        assert_eq!(value.source, ConfigSource::User);
    }

    #[test]
    fn test_merge_override() {
        let mut set = ConfigSet::new();
        set.add(ConfigValue::new(serde_json::json!({"key": "low"}), ConfigSource::System));
        set.add(ConfigValue::new(serde_json::json!({"key": "high"}), ConfigSource::User));
        let merged = set.merge();
        assert_eq!(merged["key"], "high");
    }

    #[test]
    fn test_deep_merge() {
        let mut set = ConfigSet::new();
        set.add(ConfigValue::new(serde_json::json!({"db": {"host": "localhost", "port": 5432}}), ConfigSource::System));
        set.add(ConfigValue::new(serde_json::json!({"db": {"host": "prod.db.local"}}), ConfigSource::User));
        let merged = set.merge();
        let db = merged["db"].as_object().unwrap();
        assert_eq!(db["host"], "prod.db.local");
        assert_eq!(db["port"], 5432);
    }
}
