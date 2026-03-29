//! # Secret Port
//!
//! Outbound port for secret/vault operations.

use async_trait::async_trait;

/// Secret port for secure value storage and retrieval.
#[async_trait]
pub trait SecretPort: Send + Sync {
    /// Get a secret value by key.
    async fn get(&self, key: &str) -> Result<Option<String>, SecretError>;

    /// Set a secret value.
    async fn set(&self, key: &str, value: &str) -> Result<(), SecretError>;

    /// Delete a secret.
    async fn delete(&self, key: &str) -> Result<(), SecretError>;

    /// Check if a secret exists.
    async fn exists(&self, key: &str) -> Result<bool, SecretError>;

    /// List all secret keys (metadata only, no values).
    async fn list_keys(&self) -> Result<Vec<String>, SecretError>;
}

/// Secret operation errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SecretError {
    #[error("secret not found: {0}")]
    NotFound(String),

    #[error("permission denied: {0}")]
    Permission(String),

    #[error("encryption error: {0}")]
    Encryption(String),

    #[error("connection error: {0}")]
    Connection(String),

    #[error("operation failed: {0}")]
    Operation(String),
}
