//! Error types for agileplus-sync.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Store error: {0}")]
    Store(String),

    #[error("NATS error: {0}")]
    Nats(#[from] async_nats::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Conflict detected for entity {entity_type}/{entity_id}")]
    ConflictDetected {
        entity_type: String,
        entity_id: i64,
    },

    #[error("Resolution failed: {0}")]
    ResolutionFailed(String),

    #[error("Entity not found: {entity_type}/{entity_id}")]
    EntityNotFound {
        entity_type: String,
        entity_id: i64,
    },
}
