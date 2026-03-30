use crate::config::{AppConfig, CredentialBackend};

use super::file::FileCredentialStore;
#[cfg(feature = "keychain")]
use super::keychain::KeychainCredentialStore;
use super::store::CredentialStore;

/// Create the appropriate credential store based on the app configuration.
pub fn create_credential_store(config: &AppConfig) -> Box<dyn CredentialStore> {
    match config.credentials.backend {
        #[cfg(feature = "keychain")]
        CredentialBackend::Keychain => Box::new(KeychainCredentialStore::new()),
        CredentialBackend::File => {
            Box::new(FileCredentialStore::new(&config.credentials.file_path))
        }
        CredentialBackend::Auto => {
            Box::new(FileCredentialStore::new(&config.credentials.file_path))
        }
        #[cfg(not(feature = "keychain"))]
        CredentialBackend::Keychain => {
            Box::new(FileCredentialStore::new(&config.credentials.file_path))
        }
    }
}
