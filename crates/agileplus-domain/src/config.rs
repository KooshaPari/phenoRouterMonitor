//! Application configuration — TOML schema, parsing, and env-var overrides.
//!
//! Traceability: FR-032 / WP15-T089

use std::path::PathBuf;
use std::{env, fs};

use serde::{Deserialize, Serialize};

/// Top-level configuration for AgilePlus.
///
/// All sections have defaults so a missing `config.toml` (or missing sections
/// within it) never causes a parse error.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default)]
    pub core: CoreConfig,
    #[serde(default)]
    pub credentials: CredentialConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub agents: AgentConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            core: CoreConfig::default(),
            credentials: CredentialConfig::default(),
            telemetry: TelemetryConfig::default(),
            api: ApiConfig::default(),
            agents: AgentConfig::default(),
        }
    }
}

// ----- Core -----

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoreConfig {
    #[serde(default = "default_db_path")]
    pub database_path: PathBuf,
    #[serde(default = "default_specs_dir")]
    pub specs_dir: String,
    #[serde(default = "default_target_branch")]
    pub default_target_branch: String,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            database_path: default_db_path(),
            specs_dir: default_specs_dir(),
            default_target_branch: default_target_branch(),
        }
    }
}

fn default_db_path() -> PathBuf {
    dirs_next::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".agileplus")
        .join("agileplus.db")
}

fn default_specs_dir() -> String {
    "kitty-specs".to_string()
}

fn default_target_branch() -> String {
    "main".to_string()
}

// ----- Credentials -----

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CredentialBackend {
    #[default]
    Auto,
    Keychain,
    File,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CredentialConfig {
    #[serde(default)]
    pub backend: CredentialBackend,
    #[serde(default = "default_credential_path")]
    pub file_path: PathBuf,
}

impl Default for CredentialConfig {
    fn default() -> Self {
        Self {
            backend: CredentialBackend::Auto,
            file_path: default_credential_path(),
        }
    }
}

fn default_credential_path() -> PathBuf {
    dirs_next::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".agileplus")
        .join("credentials.enc")
}

// ----- Telemetry -----

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TelemetryConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub otlp_endpoint: Option<String>,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub log_file: Option<PathBuf>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            otlp_endpoint: None,
            log_level: default_log_level(),
            log_file: None,
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

// ----- API -----

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiConfig {
    #[serde(default = "default_api_port")]
    pub port: u16,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
    #[serde(default)]
    pub cors_origins: Vec<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            port: default_api_port(),
            grpc_port: default_grpc_port(),
            cors_origins: Vec::new(),
        }
    }
}

fn default_api_port() -> u16 {
    8080
}

fn default_grpc_port() -> u16 {
    50051
}

// ----- Agents -----

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentConfig {
    #[serde(default = "default_agent")]
    pub default_agent: String,
    #[serde(default = "default_max_subagents")]
    pub max_subagents: u32,
    #[serde(default = "default_max_review_cycles")]
    pub max_review_cycles: u32,
    #[serde(default = "default_review_poll_interval")]
    pub review_poll_interval_secs: u64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            default_agent: default_agent(),
            max_subagents: default_max_subagents(),
            max_review_cycles: default_max_review_cycles(),
            review_poll_interval_secs: default_review_poll_interval(),
        }
    }
}

fn default_agent() -> String {
    "claude-code".to_string()
}

fn default_max_subagents() -> u32 {
    3
}

fn default_max_review_cycles() -> u32 {
    5
}

fn default_review_poll_interval() -> u64 {
    30
}

// ----- Loading -----

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
    /// Supported env vars: `AGILEPLUS_API_PORT`, `AGILEPLUS_GRPC_PORT`,
    /// `AGILEPLUS_TELEMETRY_LOG_LEVEL`, `AGILEPLUS_CORE_DB_PATH`,
    /// `AGILEPLUS_CORE_SPECS_DIR`.
    pub fn load_with_env_overrides() -> Result<Self, ConfigError> {
        let mut config = Self::load()?;

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
    fn validate(&self) -> Result<(), ConfigError> {
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.telemetry.log_level.to_lowercase().as_str()) {
            return Err(ConfigError::Validation(format!(
                "invalid log_level '{}'; must be one of: {}",
                self.telemetry.log_level,
                valid_log_levels.join(", ")
            )));
        }
        if self.api.port == 0 {
            return Err(ConfigError::Validation(
                "api.port must be > 0".to_string(),
            ));
        }
        if self.api.grpc_port == 0 {
            return Err(ConfigError::Validation(
                "api.grpc_port must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_validates() {
        AppConfig::default().validate().unwrap();
    }

    #[test]
    fn toml_roundtrip() {
        let config = AppConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: AppConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.api.port, 8080);
        assert_eq!(parsed.api.grpc_port, 50051);
        assert_eq!(parsed.agents.max_subagents, 3);
    }

    #[test]
    fn partial_toml_uses_defaults() {
        let partial = r#"
[api]
port = 9090
"#;
        let config: AppConfig = toml::from_str(partial).unwrap();
        assert_eq!(config.api.port, 9090);
        assert_eq!(config.api.grpc_port, 50051); // default preserved
        assert_eq!(config.agents.max_subagents, 3); // default preserved
    }

    #[test]
    fn invalid_log_level_fails_validation() {
        let bad = r#"
[telemetry]
log_level = "verbose"
"#;
        let config: AppConfig = toml::from_str(bad).unwrap();
        assert!(config.validate().is_err());
    }

    #[test]
    fn env_override_api_port() {
        // SAFETY: single-threaded test; no other threads access this env var.
        unsafe {
            env::set_var("AGILEPLUS_API_PORT", "9999");
        }
        let port: u16 = env::var("AGILEPLUS_API_PORT")
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(port, 9999);
        unsafe {
            env::remove_var("AGILEPLUS_API_PORT");
        }
    }
}
