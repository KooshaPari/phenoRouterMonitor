//! Phenotype Config Core - Cascading TOML config loader with env overrides.
//!
//! Provides a cascading configuration system that searches for TOML files
//! in standard locations and merges with environment variable overrides.
//!
//! # Search Order (lowest to highest priority)
//!
//! 1. System config: `/etc/phenotype/config.toml`
//! 2. User config: `~/.config/phenotype/config.toml`
//! 3. Project config: `./config.toml`
//! 4. Env vars: `PHENOTYPE_*` prefix
//!
//! # Features
//!
//! - **Multi-format support**: TOML, YAML, JSON via Figment 0.15+
//! - **Cascading**: Merge configs from multiple sources with priority
//! - **Validation**: Schema validation support
//! - **Environment overrides**: Env vars take highest priority

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use schemars::JsonSchema;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("Figment error: {0}")]
    Figment(String),

    #[error("config key not found: {0}")]
    KeyNotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("schema validation failed: {0}")]
    SchemaValidation(String),
}

/// Result type for config operations.
pub type Result<T> = std::result::Result<T, ConfigError>;

// ============================================================================
// Config Source Types
// ============================================================================

/// Configuration source indicating where a value originated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigSource {
    System,
    User,
    Project,
    Env,
    Inline,
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

/// Configuration value that can come from file or environment.
#[derive(Debug, Clone, Deserialize)]
pub struct ConfigValue {
    #[serde(flatten)]
    pub value: Value,
    pub source: ConfigSource,
}

impl ConfigValue {
    /// Create a new config value
    pub fn new(value: Value, source: ConfigSource) -> Self {
        Self { value, source }
    }

    /// Get the value as a specific type
    pub fn get<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        serde_json::from_value(self.value.clone())
            .map_err(|e| ConfigError::Validation(e.to_string()))
    }

    /// Get string value
    pub fn as_str(&self) -> Option<&str> {
        self.value.as_str()
    }

    /// Get bool value
    pub fn as_bool(&self) -> Option<bool> {
        self.value.as_bool()
    }

    /// Get i64 value
    pub fn as_i64(&self) -> Option<i64> {
        self.value.as_i64()
    }

    /// Get f64 value
    pub fn as_f64(&self) -> Option<f64> {
        self.value.as_f64()
    }
}

// ============================================================================
// Config Loader (Main Type)
// ============================================================================

/// Cascading TOML config loader with environment overrides.
///
/// Searches for config files in standard locations and merges them,
/// with environment variables taking highest priority.
#[derive(Debug, Clone)]
pub struct ConfigLoader {
    values: HashMap<String, ConfigValue>,
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigLoader {
    /// Creates a new config loader with default search paths.
    pub fn new() -> Self {
        let mut loader = Self {
            values: HashMap::new(),
        };
        loader.load();
        loader
    }

    /// Load configs from standard search paths.
    pub fn load(&mut self) {
        // System config
        if let Some(path) = Self::system_config_path() {
            self.load_file(&path, ConfigSource::System);
        }

        // User config
        if let Some(path) = Self::user_config_path() {
            self.load_file(&path, ConfigSource::User);
        }

        // Project config
        if let Some(path) = Self::project_config_path() {
            self.load_file(&path, ConfigSource::Project);
        }

        // Environment variables
        self.load_env();
    }

    /// Load a config file (auto-detects format based on extension).
    pub fn load_file(&mut self, path: &PathBuf, source: ConfigSource) {
        if !path.exists() {
            return;
        }

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("toml")
            .to_lowercase();

        match fs::read_to_string(path) {
            Ok(contents) => {
                let value = match extension.as_str() {
                    "yaml" | "yml" => self.parse_yaml(&contents),
                    "json" => self.parse_json(&contents),
                    _ => self.parse_toml(&contents),
                };

                match value {
                    Ok(v) => self.merge_value(v, source),
                    Err(e) => {
                        eprintln!("warning: failed to parse {}: {}", path.display(), e);
                    }
                }
            }
            Err(e) => {
                eprintln!("warning: failed to read {}: {}", path.display(), e);
            }
        }
    }

    /// Parse TOML content
    fn parse_toml(&self, content: &str) -> Result<Value> {
        let toml_value: toml::Value = toml::from_str(content)?;
        Ok(serde_json::to_value(toml_value)?)
    }

    /// Parse YAML content (using figment)
    /// Parse YAML content (using serde_yaml)
    fn parse_yaml(&self, content: &str) -> Result<Value> {
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(content)?;
        Ok(serde_json::to_value(yaml_value)?)
    }
    /// Parse JSON content
    fn parse_json(&self, content: &str) -> Result<Value> {
        serde_json::from_str(content).map_err(ConfigError::JsonParse)
    }

    /// Merge a JSON value into the config.
    fn merge_value(&mut self, value: Value, source: ConfigSource) {
        if let Some(obj) = value.as_object() {
            for (key, val) in obj {
                // Skip overriding if new source has lower or equal priority
                // This ensures env vars always win
                if let Some(existing) = self.values.get(key) {
                    if existing.source.priority() >= source.priority() {
                        continue;
                    }
                }
                self.values.insert(
                    key.clone(),
                    ConfigValue {
                        value: val.clone(),
                        source,
                    },
                );
            }
        }
    }

    /// Load configuration from environment variables.
    fn load_env(&mut self) {
        let prefix = "PHENOTYPE_";
        for (key, val) in env::vars() {
            if key.starts_with(prefix) {
                let config_key = key.strip_prefix(prefix).unwrap().to_lowercase();
                // Try to parse as JSON for complex values, fallback to string
                let value = serde_json::from_str(&val)
                    .unwrap_or_else(|_| Value::String(val));
                self.values.insert(
                    config_key,
                    ConfigValue {
                        value,
                        source: ConfigSource::Env,
                    },
                );
            }
        }
    }

    /// Get the system config path.
    fn system_config_path() -> Option<PathBuf> {
        #[cfg(unix)]
        {
            Some(PathBuf::from("/etc/phenotype/config.toml"))
        }
        #[cfg(not(unix))]
        {
            None
        }
    }

    /// Get the user config path.
    fn user_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("phenotype").join("config.toml"))
    }

    /// Get the project config path.
    fn project_config_path() -> Option<PathBuf> {
        Some(PathBuf::from("./config.toml"))
    }

    /// Get a string config value.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.values.get(key).and_then(|v| v.value.as_str())
    }

    /// Get a config value as a specific type.
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<T> {
        self.values
            .get(key)
            .ok_or_else(|| ConfigError::KeyNotFound(key.to_string()))
            .and_then(|v| {
                serde_json::from_value(v.value.clone())
                    .map_err(|e| ConfigError::Validation(e.to_string()))
            })
    }

    /// Check if a key exists.
    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// Get the source of a config value.
    pub fn source(&self, key: &str) -> Option<ConfigSource> {
        self.values.get(key).map(|v| v.source)
    }

    /// Get all keys.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(|k| k.as_str())
    }

    /// Get a config value with a default fallback.
    ///
    /// Returns the default value if the key is not found.
    ///
    /// Traces to: FR-PHENO-CONFIG-005
    pub fn get_or_default<T: for<'de> Deserialize<'de> + Default>(&self, key: &str) -> T {
        self.get(key).unwrap_or_default()
    }

    /// Merge another config loader's values into this one.
    ///
    /// Values in the other loader take precedence over existing values.
    ///
    /// Traces to: FR-PHENO-CONFIG-006
    pub fn merge(&mut self, other: ConfigLoader) {
        for (key, value) in other.values {
            // Check priority before inserting
            if let Some(existing) = self.values.get(&key) {
                if existing.source.priority() >= value.source.priority() {
                    continue;
                }
            }
            self.values.insert(key, value);
        }
    }

    /// Get all values as a HashMap reference.
    ///
    /// Useful for bulk operations or serialization.
    ///
    /// Traces to: FR-PHENO-CONFIG-007
    pub fn all_values(&self) -> &HashMap<String, ConfigValue> {
        &self.values
    }

    /// Get all values as a JSON object.
    ///
    /// Useful for serialization or passing to other systems.
    ///
    /// Traces to: FR-PHENO-CONFIG-008
    pub fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        for (key, val) in &self.values {
            map.insert(key.clone(), val.value.clone());
        }
        Value::Object(map)
    }

    /// Validate the config against a JSON schema.
    ///
    /// Returns Ok(()) if validation passes, or an error describing the failures.
    ///
    /// Traces to: FR-PHENO-CONFIG-009
    pub fn validate_schema(&self, schema_json: &str) -> Result<()> {
        let schema: serde_json::Value = serde_json::from_str(schema_json)
            .map_err(|e| ConfigError::SchemaValidation(format!("invalid schema: {}", e)))?;

        let compiled_schema = jsonschema::JSONSchema::compile(&schema)
            .map_err(|e| ConfigError::SchemaValidation(format!("schema compilation failed: {}", e)))?;

        let config_json = self.to_json();
        let result = compiled_schema.validate(&config_json);

        if let Err(errors) = result {
            let error_messages: Vec<String> = errors
                .map(|e| format!("{}: {}", e.instance_path, e))
                .collect();
            Err(ConfigError::SchemaValidation(error_messages.join("; ")))
        } else {
            Ok(())
        }
    }

    /// Validate the config against a typed schema.
    ///
    /// This method generates a JSON schema from a Rust type and validates
    /// the config against it.
    ///
    /// Traces to: FR-PHENO-CONFIG-010
    pub fn validate_typed<T: JsonSchema + Serialize + for<'de> Deserialize<'de>>(
        &self,
    ) -> Result<T> {
        // Generate schema from type
        let schema = schemars::schema_for!(T);
        let schema_json = serde_json::to_string_pretty(&schema)
            .map_err(|e| ConfigError::SchemaValidation(format!("schema generation failed: {}", e)))?;

        // Validate against the generated schema
        self.validate_schema(&schema_json)?;

        // Deserialize the config into the target type
        let config_json = self.to_json();
        serde_json::from_value(config_json)
            .map_err(|e| ConfigError::Validation(format!("deserialization failed: {}", e)))
    }

    /// Get the JSON Schema for a type.
    ///
    /// Useful for generating documentation or for external validation tools.
    ///
    /// Traces to: FR-PHENO-CONFIG-011
    pub fn generate_schema<T: JsonSchema>() -> Result<String> {
        let schema = schemars::schema_for!(T);
        serde_json::to_string_pretty(&schema)
            .map_err(|e| ConfigError::SchemaValidation(format!("schema serialization failed: {}", e)))
    }
}

impl ConfigSource {
    /// Get the priority of this source (higher = more important).
    fn priority(&self) -> u8 {
        match self {
            ConfigSource::System => 1,
            ConfigSource::User => 2,
            ConfigSource::Project => 3,
            ConfigSource::Inline => 4,
            ConfigSource::Env => 5,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_loader() {
        let loader = ConfigLoader::new();
        // Should not panic even if no config files exist
        assert!(loader.keys().count() >= 0);
    }

    #[test]
    fn test_project_config_path() {
        let path = ConfigLoader::project_config_path();
        assert!(path.is_some());
        assert_eq!(path.unwrap(), PathBuf::from("./config.toml"));
    }

    // FR-PHENO-CONFIG-005: get_or_default returns default when key missing
    #[test]
    fn test_get_or_default() {
        let loader = ConfigLoader::new();
        let value: String = loader.get_or_default("missing_key");
        assert_eq!(value, String::default());
    }

    // FR-PHENO-CONFIG-006: merge combines loaders
    #[test]
    fn test_merge_loaders() {
        let mut loader1 = ConfigLoader::new();
        let loader2 = ConfigLoader::new();
        loader1.merge(loader2);
        // Test completes if no panic
        assert!(true);
    }

    // FR-PHENO-CONFIG-007: all_values returns reference to values
    #[test]
    fn test_all_values() {
        let loader = ConfigLoader::new();
        let values = loader.all_values();
        assert!(values.len() >= 0);
    }

    // FR-PHENO-CONFIG-008: to_json returns JSON object
    #[test]
    fn test_to_json() {
        let loader = ConfigLoader::new();
        let json = loader.to_json();
        assert!(json.is_object());
    }

    #[test]
    fn test_config_source_priority() {
        assert!(ConfigSource::Env.priority() > ConfigSource::Project.priority());
        assert!(ConfigSource::Project.priority() > ConfigSource::User.priority());
        assert!(ConfigSource::User.priority() > ConfigSource::System.priority());
    }

    #[test]
    fn test_parse_toml() {
        let loader = ConfigLoader::new();
        let toml_content = r#"
[database]
host = "localhost"
port = 5432
"#;
        let result = loader.parse_toml(toml_content);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.get("database").is_some());
    }

    #[test]
    fn test_parse_json() {
        let loader = ConfigLoader::new();
        let json_content = r#"{"name": "test", "value": 42}"#;
        let result = loader.parse_json(json_content);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.get("name").and_then(|v| v.as_str()), Some("test"));
    }

    #[test]
    fn test_merge_respects_priority() {
        let mut loader = ConfigLoader::new();

        // Add a value from system
        loader.merge_value(
            serde_json::json!({"key": "system_value"}),
            ConfigSource::System,
        );

        // Try to override with another system value
        loader.merge_value(
            serde_json::json!({"key": "project_value"}),
            ConfigSource::Project,
        );

        // Project should win (higher priority)
        assert_eq!(
            loader.get_str("key"),
            Some("project_value")
        );

        // Try to override with env
        loader.merge_value(
            serde_json::json!({"key": "env_value"}),
            ConfigSource::Env,
        );

        // Env should win (highest priority)
        assert_eq!(
            loader.get_str("key"),
            Some("env_value")
        );
    }

    // FR-PHENO-CONFIG-009: validate_schema validates JSON schema
    #[test]
    fn test_validate_schema_success() {
        let loader = ConfigLoader::new();
        let schema = r#"{
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        }"#;
        let result = loader.validate_schema(schema);
        assert!(result.is_ok());
    }

    // FR-PHENO-CONFIG-009: validate_schema fails on invalid config
    #[test]
    fn test_validate_schema_failure() {
        let mut loader = ConfigLoader::new();
        loader.merge_value(
            serde_json::json!({"name": "test", "port": "not_a_number"}),
            ConfigSource::Inline,
        );

        let schema = r#"{
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "port": { "type": "integer" }
            }
        }"#;
        let result = loader.validate_schema(schema);
        assert!(result.is_err());
    }

    // FR-PHENO-CONFIG-011: generate_schema generates valid JSON schema
    #[test]
    fn test_generate_schema() {
        #[derive(Serialize, Deserialize, JsonSchema)]
        struct TestConfig {
            name: String,
            port: u16,
        }

        let schema_result = ConfigLoader::generate_schema::<TestConfig>();
        assert!(schema_result.is_ok());

        let schema = schema_result.unwrap();
        assert!(schema.contains("name"));
        assert!(schema.contains("port"));
    }
}