//! Phenotype event sourcing with blake3 hash chains.
//!
//! Provides append-only event storage with cryptographic hash chain verification
//! for audit-heavy domains.
//!
//! # Core Types
//!
//! - [`EventEnvelope<T>`]: Immutable event wrapper with blake3 hash chain
//! - [`EventStore`]: Synchronous event store trait
//! - [`AsyncEventStore`]: Async event store trait
//! - [`InMemoryEventStore`]: Reference implementation

pub mod async_store;
pub mod error;
pub mod event;
pub mod hash;
pub mod memory;
pub mod snapshot;
pub mod store;

pub use error::{EventSourcingError, HashError, Result};
pub use event::EventEnvelope;
pub use hash::{compute_hash, verify_chain, ZERO_HASH};
pub use memory::InMemoryEventStore;
pub use snapshot::{Snapshot, SnapshotConfig};
pub use store::EventStore;
