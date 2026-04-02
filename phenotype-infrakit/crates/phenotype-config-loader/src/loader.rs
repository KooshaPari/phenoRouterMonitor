//! Configuration loaders

use crate::error::LoaderError;
use crate::ConfigLoader;
use std::path::Path;

/// JSON configuration loader
#[derive(Debug, Clone, Default)]
pub struct JsonLoader;

impl JsonLoader {
    /// Create a new JsonLoader
    pub fn new() -> Self {
        Self
    }
}

impl ConfigLoader for JsonLoader {
    fn load_path(&self, path: &str) -> Result<serde_json::Value, LoaderError> {
        let path = Path::new(path);
        if !path.exists() {
            return Err(LoaderError::file_not_found(path.display().to_string()));
        }
        let content = std::fs::read_to_string(path)
            .map_err(|e| LoaderError::read_error(path.display().to_string(), e))?;
        self.load_str(&content)
    }

    fn load_str(&self, content: &str) -> Result<serde_json::Value, LoaderError> {
        Ok(serde_json::from_str(content)?)
    }
}

/// TOML configuration loader
#[derive(Debug, Clone, Default)]
pub struct TomlLoader;

impl TomlLoader {
    /// Create a new TomlLoader
    pub fn new() -> Self {
        Self
    }

    /// Load TOML file and convert to JSON Value
    pub fn load_as_json(&self, path: &str) -> Result<serde_json::Value, LoaderError> {
        let path = Path::new(path);
        if !path.exists() {
            return Err(LoaderError::file_not_found(path.display().to_string()));
        }
        let content = std::fs::read_to_string(path)
            .map_err(|e| LoaderError::read_error(path.display().to_string(), e))?;
        let v: toml::Value = toml::from_str(&content)?;
        Ok(toml_to_json(v))
    }
}

impl ConfigLoader for TomlLoader {
    fn load_path(&self, path: &str) -> Result<serde_json::Value, LoaderError> {
        self.load_as_json(path)
    }

    fn load_str(&self, content: &str) -> Result<serde_json::Value, LoaderError> {
        let v: toml::Value = toml::from_str(content)?;
        Ok(toml_to_json(v))
    }
}

/// File loader that auto-detects format based on extension
#[derive(Debug, Clone, Default)]
pub struct FileLoader {
    json: JsonLoader,
    toml: TomlLoader,
}

impl FileLoader {
    /// Create a new FileLoader
    pub fn new() -> Self {
        Self {
            json: JsonLoader::new(),
            toml: TomlLoader::new(),
        }
    }
}

impl ConfigLoader for FileLoader {
    fn load_path(&self, path: &str) -> Result<serde_json::Value, LoaderError> {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "json" => self.json.load_path(path),
            "toml" => self.toml.load_path(path),
            _ => Err(LoaderError::UnsupportedFormat { format: ext }),
        }
    }

    fn load_str(&self, content: &str) -> Result<serde_json::Value, LoaderError> {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(content) {
            return Ok(v);
        }
        if let Ok(v) = content.parse::<toml::Value>() {
            return Ok(toml_to_json(v));
        }
        Err(LoaderError::UnsupportedFormat {
            format: "unknown".to_string(),
        })
    }
}

/// Convert TOML Value to JSON Value
fn toml_to_json(v: toml::Value) -> serde_json::Value {
    match v {
        toml::Value::String(s) => serde_json::Value::String(s),
        toml::Value::Integer(i) => serde_json::Value::Number(i.into()),
        toml::Value::Float(f) => serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        toml::Value::Boolean(b) => serde_json::Value::Bool(b),
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
        toml::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(toml_to_json).collect())
        }
        toml::Value::Table(t) => {
            serde_json::Value::Object(t.into_iter().map(|(k, v)| (k, toml_to_json(v))).collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_loader_load_str() {
        let loader = JsonLoader::new();
        let result = loader.load_str(r#"{"key": "value"}"#);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["key"], "value");
    }

    #[test]
    fn test_json_loader_invalid_json() {
        let loader = JsonLoader::new();
        let result = loader.load_str("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_json_loader_complex() {
        let loader = JsonLoader::new();
        let json = r#"{"name": "test", "count": 42, "active": true, "items": [1, 2, 3]}"#;
        let result = loader.load_str(json);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["name"], "test");
        assert_eq!(value["count"], 42);
        assert_eq!(value["active"], true);
    }

    #[test]
    fn test_toml_loader_load_str() {
        let loader = TomlLoader::new();
        let toml_str = r#"key = "value""#;
        let result = loader.load_str(toml_str);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["key"], "value");
    }

    #[test]
    fn test_toml_loader_invalid_toml() {
        let loader = TomlLoader::new();
        let result = loader.load_str("not = toml [invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_toml_to_json_conversion() {
        let toml_str = r#"
name = "test"
count = 42
active = true
"#;
        let loader = TomlLoader::new();
        let result = loader.load_str(toml_str);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["name"], "test");
        assert_eq!(value["count"], 42);
        assert_eq!(value["active"], true);
    }

    #[test]
    fn test_file_loader_json() {
        let loader = FileLoader::new();
        let result = loader.load_str(r#"{"type": "json"}"#);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["type"], "json");
    }

    #[test]
    fn test_file_loader_toml() {
        let loader = FileLoader::new();
        let result = loader.load_str(r#"type = "toml""#);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["type"], "toml");
    }

    #[test]
    fn test_file_loader_unsupported() {
        let loader = FileLoader::new();
        let result = loader.load_str("not json or toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_toml_nested_structure() {
        let toml_str = r#"
[database]
host = "localhost"
port = 5432

[database.credentials]
user = "admin"
"#;
        let loader = TomlLoader::new();
        let result = loader.load_str(toml_str);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["database"]["host"], "localhost");
        assert_eq!(value["database"]["port"], 5432);
        assert_eq!(value["database"]["credentials"]["user"], "admin");
    }
}
