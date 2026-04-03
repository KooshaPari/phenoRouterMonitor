//! Credential management -- OS keychain + encrypted-file fallback.
//!
//! Traceability: FR-030, FR-031 / WP15-T088

pub mod error;
pub mod factory;
pub mod file;
pub mod keychain;
pub mod keys;
pub mod memory;
pub mod store;

pub use error::CredentialError;
pub use factory::create_credential_store;
pub use file::FileCredentialStore;
#[cfg(feature = "keychain")]
pub use keychain::KeychainCredentialStore;
pub use keys::*;
pub use memory::InMemoryCredentialStore;
pub use store::CredentialStore;
