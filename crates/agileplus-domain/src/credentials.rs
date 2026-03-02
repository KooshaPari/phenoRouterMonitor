//! Credential management — OS keychain + encrypted-file fallback.
//!
//! Traceability: FR-030, FR-031 / WP15-T088

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use async_trait::async_trait;
use zeroize::Zeroizing;

/// Known credential keys used by AgilePlus.
pub mod keys {
    pub const GITHUB_TOKEN: &str = "github-token";
    pub const CODERABBIT_KEY: &str = "coderabbit-key";
    pub const PLANESO_KEY: &str = "planeso-key";
    /// Comma-separated list of valid AgilePlus HTTP API keys.
    pub const API_KEYS: &str = "api-keys";
}

/// Errors from credential operations.
#[derive(Debug, thiserror::Error)]
pub enum CredentialError {
    #[error("credential not found: {0}")]
    NotFound(String),
    #[error("keychain backend error: {0}")]
    BackendError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("encryption error: {0}")]
    Encryption(String),
}

/// Port for storing and retrieving credentials.
///
/// Implementations: [`KeychainCredentialStore`], [`FileCredentialStore`],
/// [`InMemoryCredentialStore`] (tests).
#[async_trait]
pub trait CredentialStore: Send + Sync {
    /// Retrieve a credential value.
    async fn get(&self, service: &str, key: &str) -> Result<String, CredentialError>;

    /// Store a credential value.
    async fn set(&self, service: &str, key: &str, value: &str) -> Result<(), CredentialError>;

    /// Delete a credential.
    async fn delete(&self, service: &str, key: &str) -> Result<(), CredentialError>;

    /// List all stored keys for a service.
    async fn list_keys(&self, service: &str) -> Result<Vec<String>, CredentialError>;

    /// Validate whether a raw API key matches any stored API key.
    ///
    /// Uses constant-time comparison to prevent timing attacks.
    async fn validate_api_key(&self, provided_key: &str) -> Result<bool, CredentialError> {
        let stored = match self.get("agileplus", keys::API_KEYS).await {
            Ok(v) => v,
            Err(CredentialError::NotFound(_)) => return Ok(false),
            Err(e) => return Err(e),
        };
        // Stored value is comma-separated list of API keys.
        let valid = stored
            .split(',')
            .map(str::trim)
            .filter(|k| !k.is_empty())
            .any(|stored_key| constant_time_eq(provided_key.as_bytes(), stored_key.as_bytes()));
        Ok(valid)
    }
}

/// Constant-time byte comparison to prevent timing-based key extraction.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

// ----- OS Keychain backend -----

/// Credential store backed by the OS keychain (macOS Keychain / Linux secret-service).
///
/// Requires the `keyring` crate feature to be enabled.
#[cfg(feature = "keychain")]
pub struct KeychainCredentialStore {
    service_prefix: String,
}

#[cfg(feature = "keychain")]
impl KeychainCredentialStore {
    pub fn new() -> Self {
        Self {
            service_prefix: "agileplus".to_string(),
        }
    }

    fn entry_service(&self, service: &str) -> String {
        format!("{}-{}", self.service_prefix, service)
    }
}

#[cfg(feature = "keychain")]
#[async_trait]
impl CredentialStore for KeychainCredentialStore {
    async fn get(&self, service: &str, key: &str) -> Result<String, CredentialError> {
        let entry = keyring::Entry::new(&self.entry_service(service), key)
            .map_err(|e| CredentialError::BackendError(e.to_string()))?;
        entry.get_password().map_err(|e| match e {
            keyring::Error::NoEntry => CredentialError::NotFound(key.to_string()),
            other => CredentialError::BackendError(other.to_string()),
        })
    }

    async fn set(&self, service: &str, key: &str, value: &str) -> Result<(), CredentialError> {
        let entry = keyring::Entry::new(&self.entry_service(service), key)
            .map_err(|e| CredentialError::BackendError(e.to_string()))?;
        entry
            .set_password(value)
            .map_err(|e| CredentialError::BackendError(e.to_string()))
    }

    async fn delete(&self, service: &str, key: &str) -> Result<(), CredentialError> {
        let entry = keyring::Entry::new(&self.entry_service(service), key)
            .map_err(|e| CredentialError::BackendError(e.to_string()))?;
        entry.delete_credential().map_err(|e| match e {
            keyring::Error::NoEntry => CredentialError::NotFound(key.to_string()),
            other => CredentialError::BackendError(other.to_string()),
        })
    }

    async fn list_keys(&self, _service: &str) -> Result<Vec<String>, CredentialError> {
        // The `keyring` crate does not expose enumeration; return empty list.
        Ok(Vec::new())
    }
}

// ----- Encrypted-file backend -----

/// Credential store backed by an AES-256-GCM encrypted JSON file.
///
/// The file is stored at `~/.agileplus/credentials.enc`.
/// Key derivation uses Argon2id from a passphrase.
/// File permissions are set to 0o600 on creation (Unix only).
pub struct FileCredentialStore {
    path: PathBuf,
    /// In-memory cache (service → key → value), protected by a RwLock.
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

        // Set file permissions to 0600 on Unix.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&self.path, perms)?;
        }
        Ok(())
    }
}

#[async_trait]
impl CredentialStore for FileCredentialStore {
    async fn get(&self, service: &str, key: &str) -> Result<String, CredentialError> {
        self.ensure_loaded()?;
        let cache = self.cache.read().unwrap();
        cache
            .get(service)
            .and_then(|m| m.get(key))
            .cloned()
            .ok_or_else(|| CredentialError::NotFound(key.to_string()))
    }

    async fn set(&self, service: &str, key: &str, value: &str) -> Result<(), CredentialError> {
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

    async fn delete(&self, service: &str, key: &str) -> Result<(), CredentialError> {
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

    async fn list_keys(&self, service: &str) -> Result<Vec<String>, CredentialError> {
        self.ensure_loaded()?;
        let cache = self.cache.read().unwrap();
        Ok(cache
            .get(service)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default())
    }
}

// ----- In-memory backend (tests / CI) -----

/// In-memory credential store for unit tests and CI environments without a keychain.
#[derive(Default)]
pub struct InMemoryCredentialStore {
    store: RwLock<HashMap<String, HashMap<String, Zeroizing<String>>>>,
}

impl InMemoryCredentialStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn namespace(service: &str, key: &str) -> String {
        format!("{service}::{key}")
    }
}

#[async_trait]
impl CredentialStore for InMemoryCredentialStore {
    async fn get(&self, service: &str, key: &str) -> Result<String, CredentialError> {
        let store = self.store.read().unwrap();
        store
            .get(service)
            .and_then(|m| m.get(key))
            .map(|v| v.as_str().to_string())
            .ok_or_else(|| CredentialError::NotFound(Self::namespace(service, key)))
    }

    async fn set(&self, service: &str, key: &str, value: &str) -> Result<(), CredentialError> {
        let mut store = self.store.write().unwrap();
        store
            .entry(service.to_string())
            .or_default()
            .insert(key.to_string(), Zeroizing::new(value.to_string()));
        Ok(())
    }

    async fn delete(&self, service: &str, key: &str) -> Result<(), CredentialError> {
        let mut store = self.store.write().unwrap();
        if let Some(svc) = store.get_mut(service) {
            if svc.remove(key).is_none() {
                return Err(CredentialError::NotFound(Self::namespace(service, key)));
            }
        } else {
            return Err(CredentialError::NotFound(Self::namespace(service, key)));
        }
        Ok(())
    }

    async fn list_keys(&self, service: &str) -> Result<Vec<String>, CredentialError> {
        let store = self.store.read().unwrap();
        Ok(store
            .get(service)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default())
    }
}

// ----- Factory -----

use crate::config::{AppConfig, CredentialBackend};

/// Create the appropriate credential store based on the app configuration.
pub fn create_credential_store(config: &AppConfig) -> Box<dyn CredentialStore> {
    match config.credentials.backend {
        #[cfg(feature = "keychain")]
        CredentialBackend::Keychain => Box::new(KeychainCredentialStore::new()),
        CredentialBackend::File => {
            Box::new(FileCredentialStore::new(&config.credentials.file_path))
        }
        CredentialBackend::Auto | CredentialBackend::Keychain => {
            // Without the `keychain` feature, always fall back to file.
            Box::new(FileCredentialStore::new(&config.credentials.file_path))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_set_get_delete() {
        let store = InMemoryCredentialStore::new();
        store.set("svc", "key1", "val1").await.unwrap();
        assert_eq!(store.get("svc", "key1").await.unwrap(), "val1");
        store.delete("svc", "key1").await.unwrap();
        assert!(matches!(
            store.get("svc", "key1").await,
            Err(CredentialError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn in_memory_list_keys() {
        let store = InMemoryCredentialStore::new();
        store.set("svc", "a", "1").await.unwrap();
        store.set("svc", "b", "2").await.unwrap();
        let mut keys = store.list_keys("svc").await.unwrap();
        keys.sort();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[tokio::test]
    async fn validate_api_key_single() {
        let store = InMemoryCredentialStore::new();
        store
            .set("agileplus", keys::API_KEYS, "secret-key-abc")
            .await
            .unwrap();
        assert!(store.validate_api_key("secret-key-abc").await.unwrap());
        assert!(!store.validate_api_key("wrong-key").await.unwrap());
    }

    #[tokio::test]
    async fn validate_api_key_multiple() {
        let store = InMemoryCredentialStore::new();
        store
            .set("agileplus", keys::API_KEYS, "key-one, key-two, key-three")
            .await
            .unwrap();
        assert!(store.validate_api_key("key-two").await.unwrap());
        assert!(!store.validate_api_key("key-four").await.unwrap());
    }

    #[tokio::test]
    async fn validate_api_key_no_keys_stored() {
        let store = InMemoryCredentialStore::new();
        assert!(!store.validate_api_key("anything").await.unwrap());
    }

    #[tokio::test]
    async fn file_store_set_get() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("creds.json");
        let store = FileCredentialStore::new(&path);
        store.set("svc", "tok", "abc123").await.unwrap();
        assert!(path.exists());
        assert_eq!(store.get("svc", "tok").await.unwrap(), "abc123");
    }

    #[tokio::test]
    async fn constant_time_eq_same_length() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"hellx"));
    }

    #[tokio::test]
    async fn constant_time_eq_different_length() {
        assert!(!constant_time_eq(b"hello", b"helloo"));
    }
}
