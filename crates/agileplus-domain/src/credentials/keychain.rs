#[cfg(feature = "keychain")]
use super::error::CredentialError;
#[cfg(feature = "keychain")]
use super::store::CredentialStore;

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
impl Default for KeychainCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "keychain")]
impl CredentialStore for KeychainCredentialStore {
    fn get(&self, service: &str, key: &str) -> Result<String, CredentialError> {
        let entry = keyring::Entry::new(&self.entry_service(service), key)
            .map_err(|e| CredentialError::BackendError(e.to_string()))?;
        entry.get_password().map_err(|e| match e {
            keyring::Error::NoEntry => CredentialError::NotFound(key.to_string()),
            other => CredentialError::BackendError(other.to_string()),
        })
    }

    fn set(&self, service: &str, key: &str, value: &str) -> Result<(), CredentialError> {
        let entry = keyring::Entry::new(&self.entry_service(service), key)
            .map_err(|e| CredentialError::BackendError(e.to_string()))?;
        entry
            .set_password(value)
            .map_err(|e| CredentialError::BackendError(e.to_string()))
    }

    fn delete(&self, service: &str, key: &str) -> Result<(), CredentialError> {
        let entry = keyring::Entry::new(&self.entry_service(service), key)
            .map_err(|e| CredentialError::BackendError(e.to_string()))?;
        entry.delete_credential().map_err(|e| match e {
            keyring::Error::NoEntry => CredentialError::NotFound(key.to_string()),
            other => CredentialError::BackendError(other.to_string()),
        })
    }

    fn list_keys(&self, _service: &str) -> Result<Vec<String>, CredentialError> {
        Ok(Vec::new())
    }
}
