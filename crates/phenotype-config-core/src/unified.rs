//! Figment-based unified configuration loader.
//!
//! Replaces scattered config loading implementations across the Phenotype ecosystem
//! with a single, composable loader that supports TOML/YAML/JSON files, environment
//! variable overrides, defaults, and runtime overrides.

use crate::dirs_helper::ConfigDirs;
use crate::error::{ConfigError, Result};
use figment::providers::{Env, Format, Json, Serialized, Toml, Yaml};
use figment::Figment;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Unified configuration loader built on figment.
///
/// Composes configuration from multiple sources with well-defined precedence:
/// 1. Compiled defaults (lowest)
/// 2. System config files (`/etc/<app>/config.*`)
/// 3. User config files (`~/.config/<app>/config.*`)
/// 4. Environment-specific files (`config.<env>.toml`)
/// 5. Project-local files (`./config.*`)
/// 6. Environment variables (`<PREFIX>_*`)
/// 7. Runtime overrides (highest)
pub struct UnifiedConfigLoader {
    figment: Figment,
    config_dir: Option<PathBuf>,
    env_prefix: String,
    app_name: String,
    environment: Option<String>,
    overrides: HashMap<String, String>,
}

impl UnifiedConfigLoader {
    /// Create a new loader for the given application.
    ///
    /// # Arguments
    /// * `app_name` - Application name, used for directory discovery and env prefix
    pub fn new(app_name: impl Into<String>) -> Self {
        let app_name = app_name.into();
        let env_prefix = app_name.to_uppercase().replace('-', "_");
        Self {
            figment: Figment::new(),
            config_dir: None,
            env_prefix,
            app_name,
            environment: None,
            overrides: HashMap::new(),
        }
    }

    /// Set an explicit config directory instead of using auto-discovery.
    pub fn with_config_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.config_dir = Some(dir.into());
        self
    }

    /// Set the environment prefix for env var overrides (default: APP_NAME uppercased).
    pub fn with_env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = prefix.into();
        self
    }

    /// Set the deployment environment (e.g. "dev", "staging", "prod").
    ///
    /// When set, the loader also merges `config.<env>.toml` (or yaml/json).
    pub fn with_environment(mut self, env: impl Into<String>) -> Self {
        self.environment = Some(env.into());
        self
    }

    /// Merge compiled defaults into the config stack (lowest precedence).
    pub fn with_defaults<T: Serialize>(mut self, defaults: T) -> Self {
        self.figment = self.figment.merge(Serialized::defaults(defaults));
        self
    }

    /// Add runtime key-value overrides (highest precedence).
    pub fn with_override(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.overrides.insert(key.into(), value.into());
        self
    }

    /// Add multiple runtime overrides.
    pub fn with_overrides(mut self, overrides: HashMap<String, String>) -> Self {
        self.overrides.extend(overrides);
        self
    }

    /// Load configuration, merging all sources according to precedence.
    pub fn load<T: DeserializeOwned>(self) -> Result<T> {
        let figment = self.build_figment()?;
        figment.extract().map_err(|e| ConfigError::deserialize(e.to_string()))
    }

    /// Load configuration as a `serde_json::Value` for dynamic inspection.
    pub fn load_value(self) -> Result<serde_json::Value> {
        self.load()
    }

    /// Build the composed figment without extracting, for inspection or further composition.
    pub fn build_figment(self) -> Result<Figment> {
        let mut figment = self.figment;

        // Discover config directory
        let config_dir = match self.config_dir {
            Some(dir) => Some(dir),
            None => ConfigDirs::config_home()
                .ok()
                .map(|home| home.join(&self.app_name)),
        };

        // Merge system config files
        if let Ok(sys_dir) = ConfigDirs::config_system() {
            let sys_app_dir = sys_dir.join(&self.app_name);
            figment = merge_directory_configs(figment, &sys_app_dir);
        }

        // Merge user config files
        if let Some(ref dir) = config_dir {
            figment = merge_directory_configs(figment, dir);
        }

        // Merge environment-specific files
        if let Some(ref env_name) = self.environment {
            if let Some(ref dir) = config_dir {
                figment = merge_env_configs(figment, dir, env_name);
            }
        }

        // Merge project-local config files (cwd)
        if let Ok(cwd) = std::env::current_dir() {
            figment = merge_directory_configs(figment, &cwd);
        }

        // Merge environment variables
        figment = figment.merge(
            Env::prefixed(&format!("{}_", self.env_prefix))
                .split("__"),
        );

        // Merge runtime overrides
        if !self.overrides.is_empty() {
            figment = figment.merge(Serialized::defaults(self.overrides));
        }

        Ok(figment)
    }
}

/// Discover config files in a directory matching standard names.
pub fn discover_config_files(dir: &Path) -> Vec<PathBuf> {
    let candidates = ["config.toml", "config.yaml", "config.yml", "config.json"];
    candidates
        .iter()
        .map(|name| dir.join(name))
        .filter(|p| p.is_file())
        .collect()
}

/// Discover environment-specific config files.
pub fn discover_env_config_files(dir: &Path, env: &str) -> Vec<PathBuf> {
    let candidates = [
        format!("config.{env}.toml"),
        format!("config.{env}.yaml"),
        format!("config.{env}.yml"),
        format!("config.{env}.json"),
    ];
    candidates
        .iter()
        .map(|name| dir.join(name))
        .filter(|p| p.is_file())
        .collect()
}

/// Merge all standard config files from a directory into a figment.
fn merge_directory_configs(mut figment: Figment, dir: &Path) -> Figment {
    let toml_path = dir.join("config.toml");
    if toml_path.is_file() {
        figment = figment.merge(Toml::file(&toml_path));
    }
    let yaml_path = dir.join("config.yaml");
    if yaml_path.is_file() {
        figment = figment.merge(Yaml::file(&yaml_path));
    }
    let yml_path = dir.join("config.yml");
    if yml_path.is_file() {
        figment = figment.merge(Yaml::file(&yml_path));
    }
    let json_path = dir.join("config.json");
    if json_path.is_file() {
        figment = figment.merge(Json::file(&json_path));
    }
    figment
}

/// Merge environment-specific config files into a figment.
fn merge_env_configs(mut figment: Figment, dir: &Path, env: &str) -> Figment {
    let toml_path = dir.join(format!("config.{env}.toml"));
    if toml_path.is_file() {
        figment = figment.merge(Toml::file(&toml_path));
    }
    let yaml_path = dir.join(format!("config.{env}.yaml"));
    if yaml_path.is_file() {
        figment = figment.merge(Yaml::file(&yaml_path));
    }
    let yml_path = dir.join(format!("config.{env}.yml"));
    if yml_path.is_file() {
        figment = figment.merge(Yaml::file(&yml_path));
    }
    let json_path = dir.join(format!("config.{env}.json"));
    if json_path.is_file() {
        figment = figment.merge(Json::file(&json_path));
    }
    figment
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::fs;
    use tempfile::TempDir;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct AppConfig {
        app_name: String,
        #[serde(default)]
        port: u16,
        #[serde(default)]
        debug: bool,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
    struct Defaults {
        app_name: String,
        port: u16,
        debug: bool,
    }

    #[test]
    fn test_load_from_toml_directory() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("config.toml"),
            "app_name = \"test\"\nport = 8080\ndebug = true\n",
        )
        .unwrap();

        let config: AppConfig = UnifiedConfigLoader::new("test")
            .with_config_dir(dir.path())
            .load()
            .unwrap();

        assert_eq!(config.app_name, "test");
        assert_eq!(config.port, 8080);
        assert!(config.debug);
    }

    #[test]
    fn test_load_from_yaml_directory() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("config.yaml"),
            "app_name: yaml-app\nport: 3000\ndebug: false\n",
        )
        .unwrap();

        let config: AppConfig = UnifiedConfigLoader::new("test")
            .with_config_dir(dir.path())
            .load()
            .unwrap();

        assert_eq!(config.app_name, "yaml-app");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_load_from_json_directory() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("config.json"),
            r#"{"app_name":"json-app","port":5000,"debug":true}"#,
        )
        .unwrap();

        let config: AppConfig = UnifiedConfigLoader::new("test")
            .with_config_dir(dir.path())
            .load()
            .unwrap();

        assert_eq!(config.app_name, "json-app");
        assert_eq!(config.port, 5000);
    }

    #[test]
    fn test_defaults_merge() {
        let dir = TempDir::new().unwrap();
        // Only override app_name in file, port/debug come from defaults
        fs::write(dir.path().join("config.toml"), "app_name = \"overridden\"\n").unwrap();

        let defaults = Defaults {
            app_name: "default".into(),
            port: 9090,
            debug: false,
        };

        let config: AppConfig = UnifiedConfigLoader::new("test")
            .with_defaults(defaults)
            .with_config_dir(dir.path())
            .load()
            .unwrap();

        assert_eq!(config.app_name, "overridden");
        assert_eq!(config.port, 9090);
        assert!(!config.debug);
    }

    #[test]
    fn test_runtime_overrides() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("config.toml"),
            "app_name = \"file\"\nport = 1000\n",
        )
        .unwrap();

        let config: AppConfig = UnifiedConfigLoader::new("test")
            .with_config_dir(dir.path())
            .with_override("app_name", "override")
            .load()
            .unwrap();

        assert_eq!(config.app_name, "override");
    }

    #[test]
    fn test_env_specific_files() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("config.toml"),
            "app_name = \"base\"\nport = 80\ndebug = false\n",
        )
        .unwrap();
        fs::write(dir.path().join("config.dev.toml"), "debug = true\n").unwrap();

        let config: AppConfig = UnifiedConfigLoader::new("test")
            .with_config_dir(dir.path())
            .with_environment("dev")
            .load()
            .unwrap();

        assert_eq!(config.app_name, "base");
        assert!(config.debug);
    }

    #[test]
    fn test_env_var_override() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("config.toml"),
            "app_name = \"file\"\nport = 80\n",
        )
        .unwrap();

        // Set env var: MYAPP_PORT=9999
        std::env::set_var("MYAPP_PORT", "9999");

        let config: AppConfig = UnifiedConfigLoader::new("test")
            .with_config_dir(dir.path())
            .with_env_prefix("MYAPP")
            .load()
            .unwrap();

        assert_eq!(config.port, 9999);

        std::env::remove_var("MYAPP_PORT");
    }

    #[test]
    fn test_missing_config_with_defaults() {
        let dir = TempDir::new().unwrap();
        // No config files at all

        let defaults = Defaults {
            app_name: "fallback".into(),
            port: 3000,
            debug: false,
        };

        let config: AppConfig = UnifiedConfigLoader::new("test")
            .with_defaults(defaults)
            .with_config_dir(dir.path())
            .load()
            .unwrap();

        assert_eq!(config.app_name, "fallback");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_discover_config_files() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("config.toml"), "").unwrap();
        fs::write(dir.path().join("config.json"), "").unwrap();
        fs::write(dir.path().join("other.txt"), "").unwrap();

        let found = discover_config_files(dir.path());
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn test_discover_env_config_files() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("config.prod.toml"), "").unwrap();
        fs::write(dir.path().join("config.dev.toml"), "").unwrap();

        let found = discover_env_config_files(dir.path(), "prod");
        assert_eq!(found.len(), 1);
        assert!(found[0].to_string_lossy().contains("config.prod.toml"));
    }
}
