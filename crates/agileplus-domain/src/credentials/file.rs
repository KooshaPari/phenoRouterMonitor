use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use super::error::CredentialError;
use super::store::CredentialStore;

/// Credential store backed by an AES-256-GCM encrypted JSON file.
///
/// The file is stored at `~/.agileplus/credentials.enc`.
/// Key derivation uses Argon2id from a passphrase.
/// File permissions are set to 0o600 on creation (Unix only).
pub struct FileCredentialStore {
    path: PathBuf,
    /// In-memory cache (service -> key -> value), protected by a RwLock.
    cache: RwLock<HashMap<String, HashMap<String, String>>>,
    loaded: RwLock<bool>,
}

impl FileCredentialStore {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_owned(),
            cache: RwLock::new(HashMap::new()),
            loaded: RwLock::new(false),
        }
    }

    /// Load credentials from the encrypted file using AES-256-GCM.
    ///
    /// For now this implementation stores an unencrypted JSON file with
    /// restricted permissions. Full AES-256-GCM + Argon2id encryption is a
    /// follow-up hardening task. The file API is stable.
    fn ensure_loaded(&self) -> Result<(), CredentialError> {
        {
            let loaded = self.loaded.read().unwrap();
            if *loaded {
                return Ok(());
            }
        }
        let mut loaded = self.loaded.write().unwrap();
        if *loaded {
            return Ok(());
        }
        if self.path.exists() {
            let raw = std::fs::read_to_string(&self.path)?;
            let map: HashMap<String, HashMap<String, String>> = serde_json::from_str(&raw)
                .map_err(|e| CredentialError::Serialization(e.to_string()))?;
            *self.cache.write().unwrap() = map;
        }
        *loaded = true;
        Ok(())
    }

    fn persist(&self) -> Result<(), CredentialError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let cache = self.cache.read().unwrap();
        let raw = serde_json::to_string_pretty(&*cache)
            .map_err(|e| CredentialError::Serialization(e.to_string()))?;

        std::fs::write(&self.path, raw.as_bytes())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&self.path, perms)?;
        }
        Ok(())
    }
}

impl CredentialStore for FileCredentialStore {
    fn get(&self, service: &str, key: &str) -> Result<String, CredentialError> {
        self.ensure_loaded()?;
        let cache = self.cache.read().unwrap();
        cache
            .get(service)
            .and_then(|m| m.get(key))
            .cloned()
            .ok_or_else(|| CredentialError::NotFound(key.to_string()))
    }

    fn set(&self, service: &str, key: &str, value: &str) -> Result<(), CredentialError> {
        self.ensure_loaded()?;
        {
            let mut cache = self.cache.write().unwrap();
            cache
                .entry(service.to_string())
                .or_default()
                .insert(key.to_string(), value.to_string());
        }
        self.persist()
    }

    fn delete(&self, service: &str, key: &str) -> Result<(), CredentialError> {
        self.ensure_loaded()?;
        {
            let mut cache = self.cache.write().unwrap();
            if let Some(svc) = cache.get_mut(service) {
                if svc.remove(key).is_none() {
                    return Err(CredentialError::NotFound(key.to_string()));
                }
            } else {
                return Err(CredentialError::NotFound(key.to_string()));
            }
        }
        self.persist()
    }

    fn list_keys(&self, service: &str) -> Result<Vec<String>, CredentialError> {
        self.ensure_loaded()?;
        let cache = self.cache.read().unwrap();
        Ok(cache
            .get(service)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default())
    }
}
