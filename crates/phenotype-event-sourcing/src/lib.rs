//! Phenotype event sourcing with blake3 hash chains (3-5x faster than SHA-256).
//!
//! Provides portable, type-safe event sourcing patterns:
//! - EventSourced<T>: unified trait for event-sourced aggregates
//! - EventEnvelope<T>: immutable events with blake3 hashing
//! - EventStore<T>: pluggable persistence layer
//! - EventMetadata: correlation/causation tracking
//! - EventSourcingPort: async operations (persist, load, replay)
//! - EventHandlerRegistry: auto-discovery and dispatch

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

pub mod error;
pub mod event;
pub mod event_sourced;
pub mod hash;
pub mod memory;
pub mod snapshot;
pub mod store;

pub use error::{EventSourcingError, HashError};
<<<<<<< HEAD
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
=======
pub use event_sourced::{EventHandler, EventHandlerRegistry, EventMetadata, EventSourcingPort, EventSourced};
pub use hash::{compute_hash, verify_chain, ZERO_HASH};
pub use memory::InMemoryEventStore;
pub use snapshot::{Snapshot, SnapshotConfig};
pub use store::EventStore;
>>>>>>> origin/main
