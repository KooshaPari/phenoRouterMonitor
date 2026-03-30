//! Phenotype event sourcing with blake3 hash chains (3-5x faster than SHA-256).
//!
//! Provides portable, type-safe event sourcing patterns:
//! - EventSourced<T>: unified trait for event-sourced aggregates
//! - EventEnvelope<T>: immutable events with blake3 hashing
//! - EventStore<T>: pluggable persistence layer
//! - EventMetadata: correlation/causation tracking
//! - EventSourcingPort: async operations (persist, load, replay)
//! - EventHandlerRegistry: auto-discovery and dispatch

pub mod error;
pub mod event;
pub mod event_sourced;
pub mod hash;
pub mod memory;
pub mod snapshot;
pub mod store;

pub use event::EventEnvelope;
pub use error::{EventSourcingError, HashError};
pub use event_sourced::{EventHandler, EventHandlerRegistry, EventMetadata, EventSourcingPort, EventSourced};
pub use hash::{compute_hash, verify_chain, ZERO_HASH};
pub use memory::InMemoryEventStore;
pub use snapshot::{Snapshot, SnapshotConfig};
pub use store::EventStore;
