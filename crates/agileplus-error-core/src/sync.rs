use thiserror::Error;

/// Sync, messaging, and peer-related failures for AgilePlus.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SyncError {
    #[error("store error: {0}")]
    Store(String),

    #[error("nats error: {0}")]
    Nats(String),

    #[error("serialization error: {0}")]
    Serialization(String),
}

impl From<SyncError> for phenotype_error_core::ErrorKind {
    fn from(e: SyncError) -> Self {
        match e {
            SyncError::Store(m) => Self::storage(m),
            SyncError::Nats(m) => Self::connection(m),
            SyncError::Serialization(m) => Self::serialization(m),
        }
    }
}
