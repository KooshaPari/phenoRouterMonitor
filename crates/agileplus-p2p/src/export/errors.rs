use crate::error::ConnectionError;

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Event store error: {0}")]
    EventStore(String),

    #[error("Snapshot store error: {0}")]
    SnapshotStore(String),

    #[error("Device store error: {0}")]
    DeviceStore(#[from] ConnectionError),

    #[error("Sync store error: {0}")]
    SyncStore(String),
}
