use std::path::PathBuf;
use std::{env, fs};

use super::AppConfig;

/// Errors that can occur when loading or saving configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("invalid config value: {0}")]
    Validation(String),
    #[error("environment variable parse error: {0}")]
    EnvParse(String),
}

impl AppConfig {
    /// Path to the user-level config file: `~/.agileplus/config.toml`.
    pub fn config_path() -> PathBuf {
        dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".agileplus")
            .join("config.toml")
    }

    /// Load config from disk, falling back to defaults if no file exists.
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: AppConfig = toml::from_str(&content)?;
            config.validate()?;
            Ok(config)
        } else {
            Ok(AppConfig::default())
        }
    }

    /// Load config with environment variable overrides applied after file parsing.
    ///
    /// Supported env vars: `API_PORT`, `AGILEPLUS_HTTP_PORT`,
    /// `AGILEPLUS_API_PORT`, `AGILEPLUS_GRPC_PORT`,
    /// `AGILEPLUS_TELEMETRY_LOG_LEVEL`, `AGILEPLUS_CORE_DB_PATH`,
    /// `AGILEPLUS_CORE_SPECS_DIR`.
    pub fn load_with_env_overrides() -> Result<Self, ConfigError> {
        let mut config = Self::load()?;

        if let Ok(port) = env::var("API_PORT") {
            config.api.port = port
                .parse()
                .map_err(|_| ConfigError::EnvParse(format!("API_PORT={port}")))?;
        }
        if let Ok(port) = env::var("AGILEPLUS_HTTP_PORT") {
            config.api.port = port
                .parse()
                .map_err(|_| ConfigError::EnvParse(format!("AGILEPLUS_HTTP_PORT={port}")))?;
        }
        if let Ok(port) = env::var("AGILEPLUS_API_PORT") {
            config.api.port = port
                .parse()
                .map_err(|_| ConfigError::EnvParse(format!("AGILEPLUS_API_PORT={port}")))?;
        }
        if let Ok(port) = env::var("AGILEPLUS_GRPC_PORT") {
            config.api.grpc_port = port
                .parse()
                .map_err(|_| ConfigError::EnvParse(format!("AGILEPLUS_GRPC_PORT={port}")))?;
        }
        if let Ok(level) = env::var("AGILEPLUS_TELEMETRY_LOG_LEVEL") {
            config.telemetry.log_level = level;
        }
        if let Ok(db) = env::var("AGILEPLUS_CORE_DB_PATH") {
            config.core.database_path = PathBuf::from(db);
        }
        if let Ok(dir) = env::var("AGILEPLUS_CORE_SPECS_DIR") {
            config.core.specs_dir = dir;
        }

        config.validate()?;
        Ok(config)
    }

    /// Write the current config (or a fresh default) to disk if the file does not yet exist.
    ///
    /// Returns the path that was written.
    pub fn init_default() -> Result<PathBuf, ConfigError> {
        let path = Self::config_path();
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = toml::to_string_pretty(&AppConfig::default())?;
            fs::write(&path, content)?;
        }
        Ok(path)
    }

    /// Validate that the loaded configuration is internally consistent.
    pub(crate) fn validate(&self) -> Result<(), ConfigError> {
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.telemetry.log_level.to_lowercase().as_str()) {
            return Err(ConfigError::Validation(format!(
                "invalid log_level '{}'; must be one of: {}",
                self.telemetry.log_level,
                valid_log_levels.join(", ")
            )));
        }
        if self.api.port == 0 {
            return Err(ConfigError::Validation("api.port must be > 0".to_string()));
        }
        if self.api.grpc_port == 0 {
            return Err(ConfigError::Validation(
                "api.grpc_port must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}
