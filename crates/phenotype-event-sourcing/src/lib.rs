//! Phenotype event sourcing with blake3 hash chains (3-5x faster than SHA-256).

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

pub mod error;
pub mod event;
pub mod hash;
pub mod memory;
pub mod snapshot;

pub use error::{EventSourcingError, HashError};
pub use event::EventEnvelope;
pub use hash::{compute_hash, verify_chain, ZERO_HASH};
pub use memory::InMemoryEventStore;
pub use snapshot::{Snapshot, SnapshotConfig};

/// Core event store trait.
#[async_trait]
pub trait EventStore<T: Send + Sync + Serialize + DeserializeOwned> {
    /// Append an event to the store.
    async fn append(
        &self,
        entity_type: &str,
        entity_id: &str,
        event: EventEnvelope<T>,
    ) -> Result<i64, EventSourcingError>;

    /// Retrieve all events for an entity.
    async fn get_events(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<EventEnvelope<T>>, EventSourcingError>;

    /// Get the current sequence number for an entity.
    async fn get_sequence(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<i64, EventSourcingError>;
}
