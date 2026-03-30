//! AgilePlus dashboard settings load/save (`~/.agileplus/config.toml`).
//!
//! All I/O and TOML failures map to [`phenotype_error_core::ErrorKind`] for uniform handling in HTTP handlers and the CLI.

use std::fs;
use std::path::PathBuf;

use phenotype_error_core::ErrorKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaneConfig {
    pub api_url: String,
    pub api_key: String,
    pub workspace_slug: String,
    pub project_slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub pool_size: usize,
    pub retry_budget: usize,
    pub dispatch_mode: String,
    pub default_provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub endpoint_url: String,
    #[serde(default = "default_service_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub max_retries: Option<u32>,
}

fn default_service_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub theme: String,
    pub log_level: String,
    pub data_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub plane: Option<PlaneConfig>,
    pub agents: Option<AgentConfig>,
    pub services: Option<Vec<ServiceConfig>>,
    pub dashboard: Option<DashboardConfig>,
}

impl Config {
    pub fn load() -> Result<Self, ErrorKind> {
        let config_path = Self::config_path();
        if config_path.exists() {
            let content = fs::read_to_string(&config_path).map_err(ErrorKind::io)?;
            toml::from_str(&content).map_err(|e: toml::de::Error| {
                ErrorKind::serialization(e.to_string())
            })
        } else {
            Ok(Config {
                plane: None,
                agents: None,
                services: None,
                dashboard: None,
            })
        }
    }

    pub fn save(&self) -> Result<(), ErrorKind> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(ErrorKind::io)?;
        }
        let content = toml::to_string_pretty(self).map_err(|e: toml::ser::Error| {
            ErrorKind::serialization(e.to_string())
        })?;
        fs::write(config_path, content).map_err(ErrorKind::io)?;
        Ok(())
    }

    pub fn config_path() -> PathBuf {
        std::env::var("HOME")
            .ok()
            .map(|home| PathBuf::from(home).join(".agileplus/config.toml"))
            .unwrap_or_else(|| PathBuf::from(".agileplus/config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrip_in_temp() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::env::set_var("HOME", dir.path().to_str().unwrap());
        let c = Config {
            plane: None,
            agents: None,
            services: None,
            dashboard: Some(DashboardConfig {
                theme: "dark".into(),
                log_level: "info".into(),
                data_directory: "/tmp".into(),
            }),
        };
        c.save().unwrap();
        assert!(path.exists());
        let loaded = Config::load().unwrap();
        assert!(loaded.dashboard.is_some());
        std::env::remove_var("HOME");
    }
}
