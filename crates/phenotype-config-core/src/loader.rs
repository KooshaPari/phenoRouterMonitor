//! Core configuration loading and management.

use crate::error::{ConfigError, Result};
use crate::format::ConfigFormat;
use crate::dirs_helper::ConfigDirs;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::{Path, PathBuf};

/// Main configuration loader for unified config file handling.
///
/// Provides a consistent interface for loading configurations across all formats
/// with support for multiple file types and directory locations.
pub struct ConfigLoader {
    // Currently stateless, but can be extended for caching or other features
}

impl ConfigLoader {
    /// Create a new configuration loader.
    pub fn new() -> Self {
        ConfigLoader {}
    }

    /// Load configuration from a specific file path.
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Type Parameters
    /// * `T` - The struct type to deserialize into (must implement Deserialize)
    ///
    /// # Returns
    /// The deserialized configuration object, or an error if loading/parsing fails.
    ///
    /// # Examples
    /// ```ignore
    /// use phenotype_config_core::ConfigLoader;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Deserialize, Serialize)]
    /// struct MyConfig {
    ///     app_name: String,
    /// }
    ///
    /// let loader = ConfigLoader::new();
    /// let config: MyConfig = loader.load_from_file("config.toml")?;
    /// ```
    pub fn load_from_file<P: AsRef<Path>, T: DeserializeOwned>(
        &self,
        path: P,
    ) -> Result<T> {
        let path = path.as_ref();

        // Read file contents
        let content = fs::read_to_string(path).map_err(|e| {
            ConfigError::file_read(
                path,
                format!("{}({})", e, if e.kind() == std::io::ErrorKind::NotFound {
                    "file not found"
                } else {
                    "I/O error"
                }),
            )
        })?;

        // Detect format
        let format = ConfigFormat::from_path(path)?;

        // Deserialize
        format.deserialize(&content)
    }

    /// Load configuration from a file as a JSON value (format-agnostic).
    ///
    /// This is useful when you don't know the exact structure ahead of time,
    /// or need to work with the configuration as a dynamic structure.
    pub fn load_from_file_as_json<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<serde_json::Value> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::file_read(path, e.to_string()))?;

        let format = ConfigFormat::from_path(path)?;
        format.parse_to_json(&content)
    }

    /// Load configuration from a string with explicit format.
    ///
    /// # Arguments
    /// * `content` - Configuration content as a string
    /// * `format` - The format of the content
    pub fn load_from_string<T: DeserializeOwned>(
        &self,
        content: &str,
        format: ConfigFormat,
    ) -> Result<T> {
        format.deserialize(content)
    }

    /// Load configuration from a string as JSON (format-agnostic).
    pub fn load_from_string_as_json(
        &self,
        content: &str,
        format: ConfigFormat,
    ) -> Result<serde_json::Value> {
        format.parse_to_json(content)
    }

    /// Search for a configuration file in standard locations.
    ///
    /// Searches in standard platform directories:
    /// - User config: ~/.config/app_name/
    /// - System config: /etc/app_name/ (Unix) or %ProgramData%/app_name/ (Windows)
    ///
    /// # Arguments
    /// * `app_name` - Name of the application (e.g., "myapp")
    /// * `filename` - Configuration filename (e.g., "config.toml")
    ///
    /// # Returns
    /// The deserialized configuration if found, or None if not found anywhere.
    pub fn search_default_locations<T: DeserializeOwned>(
        &self,
        app_name: &str,
        filename: &str,
    ) -> Result<Option<T>> {
        let search_paths = ConfigDirs::search_paths(app_name, filename)?;

        for path in search_paths {
            if path.exists() {
                // Try to load from this path
                match self.load_from_file::<_, T>(&path) {
                    Ok(config) => {
                        tracing::info!("Loaded config from {}", path.display());
                        return Ok(Some(config));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load config from {}: {}", path.display(), e);
                        // Continue to next location
                    }
                }
            }
        }

        Ok(None)
    }

    /// Get the list of directories where configuration would be searched.
    ///
    /// Useful for error messages or configuration documentation.
    pub fn search_paths(app_name: &str, filename: &str) -> Result<Vec<PathBuf>> {
        ConfigDirs::search_paths(app_name, filename)
    }

    /// Load configuration from a file with validation callback.
    ///
    /// # Arguments
    /// * `path` - Path to configuration file
    /// * `validator` - Function to validate the loaded configuration
    ///
    /// Returns the validated configuration or a validation error.
    pub fn load_from_file_with_validation<P: AsRef<Path>, T: DeserializeOwned, F>(
        &self,
        path: P,
        validator: F,
    ) -> Result<T>
    where
        F: Fn(&T) -> Result<()>,
    {
        let config = self.load_from_file::<_, T>(path)?;
        validator(&config)?;
        Ok(config)
    }

    /// Merge multiple configuration sources with precedence.
    ///
    /// Later configurations in the list take precedence over earlier ones.
    /// Useful for merging system, user, and local configurations.
    ///
    /// # Arguments
    /// * `sources` - List of file paths in order of precedence
    ///
    /// # Returns
    /// Merged configuration as a JSON value
    pub fn merge_sources<P: AsRef<Path>>(&self, sources: &[P]) -> Result<serde_json::Value> {
        let mut merged = serde_json::json!({});

        for source in sources {
            let json = self.load_from_file_as_json(source)?;
            merge_json_values(&mut merged, &json);
        }

        Ok(merged)
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Recursively merge a source JSON value into a target.
fn merge_json_values(target: &mut serde_json::Value, source: &serde_json::Value) {
    match (target, source) {
        (serde_json::Value::Object(target_map), serde_json::Value::Object(source_map)) => {
            for (key, source_val) in source_map {
                let target_val = target_map
                    .entry(key.clone())
                    .or_insert_with(|| serde_json::json!({}));
                merge_json_values(target_val, source_val);
            }
        }
        (target, source) => {
            *target = source.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        value: i32,
    }

    #[test]
    fn test_load_from_json_file() {
        let mut file = NamedTempFile::with_suffix(".json").unwrap();
        file.write_all(br#"{"name":"test","value":42}"#).unwrap();
        file.flush().unwrap();

        let loader = ConfigLoader::new();
        let config: TestConfig = loader.load_from_file(file.path()).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.value, 42);
    }

    #[test]
    fn test_load_from_toml_file() {
        let mut file = NamedTempFile::with_suffix(".toml").unwrap();
        file.write_all(b"name = \"test\"\nvalue = 42\n").unwrap();
        file.flush().unwrap();

        let loader = ConfigLoader::new();
        let config: TestConfig = loader.load_from_file(file.path()).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.value, 42);
    }

    #[test]
    fn test_load_from_yaml_file() {
        let mut file = NamedTempFile::with_suffix(".yaml").unwrap();
        file.write_all(b"name: test\nvalue: 42\n").unwrap();
        file.flush().unwrap();

        let loader = ConfigLoader::new();
        let config: TestConfig = loader.load_from_file(file.path()).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.value, 42);
    }

    #[test]
    fn test_load_from_string_json() {
        let loader = ConfigLoader::new();
        let config: TestConfig = loader
            .load_from_string(r#"{"name":"test","value":99}"#, ConfigFormat::Json)
            .unwrap();
        assert_eq!(config.value, 99);
    }

    #[test]
    fn test_load_from_string_toml() {
        let loader = ConfigLoader::new();
        let config: TestConfig =
            loader
                .load_from_string("name = \"toml\"\nvalue = 55\n", ConfigFormat::Toml)
                .unwrap();
        assert_eq!(config.name, "toml");
    }

    #[test]
    fn test_load_nonexistent_file() {
        let loader = ConfigLoader::new();
        let result: Result<TestConfig> = loader.load_from_file("/nonexistent/path/config.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_json_values() {
        let mut target = serde_json::json!({ "a": 1, "b": { "c": 2 } });
        let source = serde_json::json!({ "b": { "d": 3 }, "e": 4 });
        merge_json_values(&mut target, &source);

        assert_eq!(target["a"], 1);
        assert_eq!(target["b"]["c"], 2);
        assert_eq!(target["b"]["d"], 3);
        assert_eq!(target["e"], 4);
    }

    #[test]
    fn test_load_as_json() {
        let mut file = NamedTempFile::with_suffix(".toml").unwrap();
        file.write_all(b"name = \"test\"\nvalue = 42\n").unwrap();
        file.flush().unwrap();

        let loader = ConfigLoader::new();
        let json = loader.load_from_file_as_json(file.path()).unwrap();
        assert_eq!(json["name"].as_str(), Some("test"));
        assert_eq!(json["value"].as_i64(), Some(42));
    }

    #[test]
    fn test_load_with_validation() {
        let mut file = NamedTempFile::with_suffix(".json").unwrap();
        file.write_all(br#"{"name":"test","value":42}"#).unwrap();
        file.flush().unwrap();

        let loader = ConfigLoader::new();
        let config: TestConfig = loader
            .load_from_file_with_validation(file.path(), |cfg: &TestConfig| {
                if cfg.value > 0 {
                    Ok(())
                } else {
                    Err(ConfigError::validation("value must be positive"))
                }
            })
            .unwrap();
        assert_eq!(config.value, 42);
    }

    #[test]
    fn test_validation_error() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(br#"{"name":"test","value":-5}"#).unwrap();
        file.flush().unwrap();

        let loader = ConfigLoader::new();
        let result: Result<TestConfig> = loader.load_from_file_with_validation(
            file.path(),
            |cfg: &TestConfig| {
                if cfg.value > 0 {
                    Ok(())
                } else {
                    Err(ConfigError::validation("value must be positive"))
                }
            },
        );
        assert!(result.is_err());
    }
}
