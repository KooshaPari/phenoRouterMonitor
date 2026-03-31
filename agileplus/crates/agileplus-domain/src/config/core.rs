use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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
    "agileplus".to_string()
}

fn default_target_branch() -> String {
    "main".to_string()
}
