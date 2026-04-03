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
