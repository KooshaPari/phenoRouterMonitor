use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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
