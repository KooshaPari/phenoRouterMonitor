use crate::traits::NotFoundMarker;
use thiserror::Error;

/// Storage-layer failures shared across AgilePlus persistence adapters.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StorageError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("connection failed: {0}")]
    Connection(String),
}

impl NotFoundMarker for StorageError {
    fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
}

impl From<StorageError> for phenotype_error_core::ErrorKind {
    fn from(e: StorageError) -> Self {
        match e {
            StorageError::NotFound(m) => Self::not_found(m),
            StorageError::Storage(m) | StorageError::Connection(m) => Self::storage(m),
        }
    }
}
