//! Phenotype event sourcing with blake3 hash chains (3-5x faster than SHA-256).

pub mod error;
pub mod event;
pub mod hash;
pub mod memory;
pub mod snapshot;

pub use event::EventEnvelope;
pub use error::{EventSourcingError, HashError};
pub use hash::{compute_hash, verify_chain, ZERO_HASH};
pub use memory::InMemoryEventStore;
pub use snapshot::{Snapshot, SnapshotConfig};
