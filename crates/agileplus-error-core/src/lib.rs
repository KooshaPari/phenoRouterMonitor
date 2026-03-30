use thiserror::Error;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ErrorKind {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Serialization failed: {0}")]
    Serialization(String),
    #[error("Conflict detected: {0}")]
    Conflict(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("Timeout: {0}")]
    Timeout(String),
}

pub type Result<T> = std::result::Result<T, ErrorKind>;