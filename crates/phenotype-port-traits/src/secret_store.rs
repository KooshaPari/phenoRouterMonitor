//! Secret store port for sensitive credential management.
//!
//! Secret stores abstract the storage of sensitive credentials (API keys, passwords, etc.)
//! and should use encryption at rest and in transit.

use async_trait::async_trait;
use std::fmt::Debug;

/// Errors that can occur during secret store operations.
#[derive(Debug, Clone)]
pub enum SecretStoreError {
    NotFound,
    AccessDenied,
    StorageError(String),
    EncryptionError(String),
}

impl std::fmt::Display for SecretStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretStoreError::NotFound => write!(f, "Secret not found"),
            SecretStoreError::AccessDenied => write!(f, "Access denied"),
            SecretStoreError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            SecretStoreError::EncryptionError(msg) => write!(f, "Encryption error: {}", msg),
        }
    }
}

impl std::error::Error for SecretStoreError {}

/// Secret store port for managing sensitive credentials.
///
/// Implementations must ensure:
/// - Encryption at rest
/// - Secure deletion (overwriting memory)
/// - Access control and audit logging
///
/// # Example
///
/// ```ignore
/// impl SecretStore for VaultSecretStore {
///     type Error = SecretStoreError;
///
///     async fn get_secret(&self, key: &str) -> Result<String, Self::Error> {
///         // Retrieve from HashiCorp Vault...
///     }
///
///     async fn set_secret(&self, key: &str, value: &str) -> Result<(), Self::Error> {
///         // Store in Vault with encryption...
///     }
/// }
/// ```
#[async_trait]
pub trait SecretStore: Send + Sync + Debug {
    /// Error type returned by secret store operations.
    type Error: std::error::Error + Send + Sync + Debug;

    /// Retrieve a secret by key.
    ///
    /// Returns `Err(SecretStoreError::NotFound)` if the secret does not exist.
    async fn get_secret(&self, key: &str) -> Result<String, Self::Error>;

    /// Store a secret with encryption.
    ///
    /// Implementations should securely handle the value in memory and overwrite it after use.
    async fn set_secret(&self, key: &str, value: &str) -> Result<(), Self::Error>;

    /// Delete a secret by key.
    ///
    /// Returns success even if the key doesn't exist.
    async fn delete_secret(&self, key: &str) -> Result<(), Self::Error>;
}
