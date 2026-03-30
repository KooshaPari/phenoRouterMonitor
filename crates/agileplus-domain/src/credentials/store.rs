use super::error::CredentialError;
use super::keys;

/// Port for storing and retrieving credentials.
///
/// All methods are synchronous -- implementations use blocking I/O only.
/// This keeps the trait dyn-compatible without requiring `async_trait`.
///
/// Implementations: `KeychainCredentialStore`, `FileCredentialStore`,
/// `InMemoryCredentialStore` (tests).
pub trait CredentialStore: Send + Sync {
    /// Retrieve a credential value.
    fn get(&self, service: &str, key: &str) -> Result<String, CredentialError>;

    /// Store a credential value.
    fn set(&self, service: &str, key: &str, value: &str) -> Result<(), CredentialError>;

    /// Delete a credential.
    fn delete(&self, service: &str, key: &str) -> Result<(), CredentialError>;

    /// List all stored keys for a service.
    fn list_keys(&self, service: &str) -> Result<Vec<String>, CredentialError>;

    /// Validate whether a raw API key matches any stored API key.
    ///
    /// Uses constant-time comparison to prevent timing attacks.
    fn validate_api_key(&self, provided_key: &str) -> Result<bool, CredentialError> {
        let stored = match self.get("agileplus", keys::API_KEYS) {
            Ok(v) => v,
            Err(CredentialError::NotFound(_)) => return Ok(false),
            Err(e) => return Err(e),
        };
        let valid = stored
            .split(',')
            .map(str::trim)
            .filter(|k| !k.is_empty())
            .any(|stored_key| constant_time_eq(provided_key.as_bytes(), stored_key.as_bytes()));
        Ok(valid)
    }
}

/// Constant-time byte comparison to prevent timing-based key extraction.
pub(crate) fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
