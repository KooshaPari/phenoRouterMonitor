//! # Secret Port
//!
//! Outbound port for secrets management.
//!
//! ## Supported Backends
//!
//! - HashiCorp Vault
//! - AWS Secrets Manager
//! - GCP Secret Manager
//! - Environment Variables (development)
//!
//! ## Interface
//!
//! ```rust,ignore
//! use async_trait::async_trait;
//! use phenotype_contracts::ports::outbound::SecretPort;
//!
//! #[async_trait]
//! impl SecretPort for VaultAdapter {
//!     async fn get(&self, key: &str) -> Result<Option<String>, Error>;
//!     async fn set(&self, key: &str, value: &str) -> Result<(), Error>;
//!     async fn delete(&self, key: &str) -> Result<(), Error>;
//!     async fn list(&self, path: &str) -> Result<Vec<String>, Error>;
//! }
//! ```

use async_trait::async_trait;

use super::Error;

/// Secret port interface.
///
/// Operations for reading and writing secrets.
#[async_trait]
pub trait SecretPort: Send + Sync {
    /// Get a secret value
    async fn get(&self, key: &str) -> Result<Option<String>, Error>;

    /// Set a secret value
    async fn set(&self, key: &str, value: &str) -> Result<(), Error>;

    /// Delete a secret
    async fn delete(&self, key: &str) -> Result<(), Error>;

    /// List secret keys at a path
    async fn list(&self, path: &str) -> Result<Vec<String>, Error>;
}

/// Secret with metadata
#[derive(Debug, Clone)]
pub struct Secret {
    pub key: String,
    pub value: String,
    pub version: u64,
    pub created_at: std::time::SystemTime,
}

/// Secret port with version support
#[async_trait]
pub trait VersionedSecretPort: SecretPort {
    /// Get specific version of a secret
    async fn get_version(&self, key: &str, version: u64) -> Result<Option<String>, Error>;

    /// List all versions of a secret
    async fn list_versions(&self, key: &str) -> Result<Vec<u64>, Error>;
}

/// Secret rotation support
#[async_trait]
pub trait SecretRotator: Send + Sync {
    /// Rotate a secret (generate new value)
    async fn rotate(&self, key: &str) -> Result<String, Error>;

    /// Enable automatic rotation for a secret
    async fn enable_auto_rotation(&self, key: &str, interval: std::time::Duration) -> Result<(), Error>;
}
