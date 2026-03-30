//! Minimal, composable config loading for Phenotype crates.
//!
//! Provides [`ConfigLoader`] trait, free functions for TOML loading with
//! optional environment-variable overlay, and config-file discovery across
//! standard paths (cwd, `~/.config/`, `/etc/`).

use std::path::{Path, PathBuf};

pub use serde::{de::DeserializeOwned, Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors produced during config loading.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Config file not found at the expected path.
    #[error("config file not found: {0}")]
    NotFound(PathBuf),

    /// Failed to parse the config file contents.
    #[error("parse error: {0}")]
    ParseError(String),

    /// A value in the config failed domain validation.
    #[error("validation error: {0}")]
    ValidationError(String),

    /// Underlying I/O error.
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Trait for types that know how to load themselves from config.
pub trait ConfigLoader: Sized + DeserializeOwned {
    /// Load from the default discovery path.
    ///
    /// # Errors
    /// Returns [`ConfigError`] if the file is missing, unparseable, or invalid.
    fn load() -> Result<Self, ConfigError>;

    /// Load from an explicit path.
    ///
    /// # Errors
    /// Returns [`ConfigError`] if the file is missing, unparseable, or invalid.
    fn load_from_path(path: &Path) -> Result<Self, ConfigError> {
        load_config(path)
    }
}

// ---------------------------------------------------------------------------
// Free functions
// ---------------------------------------------------------------------------

/// Load a TOML config file at `path` and deserialize into `T`.
///
/// # Errors
/// Returns [`ConfigError::NotFound`] if the path does not exist,
/// [`ConfigError::IoError`] on read failure, or [`ConfigError::ParseError`]
/// if deserialization fails.
pub fn load_config<T: DeserializeOwned>(path: &Path) -> Result<T, ConfigError> {
    if !path.exists() {
        return Err(ConfigError::NotFound(path.to_path_buf()));
    }
    let text = std::fs::read_to_string(path)?;
    toml::from_str::<T>(&text).map_err(|e| ConfigError::ParseError(e.to_string()))
}

/// Load a TOML config file then overlay any environment variables whose names
/// start with `{prefix}_`.
///
/// Environment variable mapping: a field named `foo_bar` matches
/// `{PREFIX}_FOO_BAR`. Nested tables are not walked -- only top-level
/// string fields are overlaid.
///
/// Internally this deserializes to a [`toml::Table`], patches matching keys,
/// then re-serializes and deserializes into `T`.
///
/// # Errors
/// Same as [`load_config`], plus [`ConfigError::ParseError`] if the patched
/// table cannot round-trip into `T`.
pub fn load_config_with_env<T: DeserializeOwned>(
    path: &Path,
    prefix: &str,
) -> Result<T, ConfigError> {
    if !path.exists() {
        return Err(ConfigError::NotFound(path.to_path_buf()));
    }
    let text = std::fs::read_to_string(path)?;
    let mut table: toml::Table =
        toml::from_str(&text).map_err(|e| ConfigError::ParseError(e.to_string()))?;

    let prefix_upper = format!("{}_", prefix.to_uppercase());

    for (key, value) in std::env::vars() {
        if let Some(suffix) = key.strip_prefix(&prefix_upper) {
            let config_key = suffix.to_lowercase();
            if table.contains_key(&config_key) {
                let existing = &table[&config_key];
                let new_value = coerce_env_value(&value, existing);
                table.insert(config_key, new_value);
            }
        }
    }

    let patched =
        toml::to_string(&table).map_err(|e| ConfigError::ParseError(e.to_string()))?;
    toml::from_str::<T>(&patched).map_err(|e| ConfigError::ParseError(e.to_string()))
}

/// Search standard directories for a config file named `name`.
///
/// Search order:
/// 1. Current working directory
/// 2. `~/.config/{name}`
/// 3. `/etc/{name}`
///
/// Returns the first path that exists, or `None`.
pub fn find_config_file(name: &str) -> Option<PathBuf> {
    let candidates: Vec<PathBuf> = vec![
        PathBuf::from(name),
        dirs::config_dir()
            .map(|d| d.join(name))
            .unwrap_or_default(),
        PathBuf::from("/etc").join(name),
    ];

    candidates.into_iter().find(|p| p.exists())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Best-effort coercion of an env-var string into the TOML type of `existing`.
fn coerce_env_value(raw: &str, existing: &toml::Value) -> toml::Value {
    match existing {
        toml::Value::Integer(_) => raw
            .parse::<i64>()
            .map(toml::Value::Integer)
            .unwrap_or_else(|_| toml::Value::String(raw.to_owned())),
        toml::Value::Float(_) => raw
            .parse::<f64>()
            .map(toml::Value::Float)
            .unwrap_or_else(|_| toml::Value::String(raw.to_owned())),
        toml::Value::Boolean(_) => raw
            .parse::<bool>()
            .map(toml::Value::Boolean)
            .unwrap_or_else(|_| toml::Value::String(raw.to_owned())),
        _ => toml::Value::String(raw.to_owned()),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestCfg {
        name: String,
        port: u16,
        debug: Option<bool>,
    }

    #[test]
    fn load_toml_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "name = \"app\"\nport = 8080").unwrap();

        let cfg: TestCfg = load_config(&path).unwrap();
        assert_eq!(cfg.name, "app");
        assert_eq!(cfg.port, 8080);
        assert_eq!(cfg.debug, None);
    }

    #[test]
    fn load_missing_file_returns_not_found() {
        let res = load_config::<TestCfg>(Path::new("/nonexistent/config.toml"));
        assert!(matches!(res, Err(ConfigError::NotFound(_))));
    }

    #[test]
    fn load_invalid_toml_returns_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.toml");
        std::fs::write(&path, "not valid toml {{{{").unwrap();

        let res = load_config::<TestCfg>(&path);
        assert!(matches!(res, Err(ConfigError::ParseError(_))));
    }

    #[test]
    fn env_overlay_overrides_field() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("env.toml");
        std::fs::write(&path, "name = \"original\"\nport = 3000").unwrap();

        unsafe {
            std::env::set_var("TESTPREFIX_NAME", "overridden");
        }

        let cfg: TestCfg = load_config_with_env(&path, "TESTPREFIX").unwrap();
        assert_eq!(cfg.name, "overridden");
        assert_eq!(cfg.port, 3000);

        unsafe {
            std::env::remove_var("TESTPREFIX_NAME");
        }
    }

    #[test]
    fn find_config_file_returns_none_for_missing() {
        assert!(find_config_file("nonexistent_phenotype_cfg_12345.toml").is_none());
    }
}
