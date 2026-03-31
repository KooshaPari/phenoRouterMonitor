//! # Phenotype Event Sourcing
//!
//! Append-only event store with SHA-256 hash chains for auditability.

pub mod error;
pub mod event;
pub mod hash;
pub mod memory;
pub mod store;

pub use error::EventSourcingError;
pub use event::{Event, EventEnvelope};
pub use hash::{compute_event_hash, verify_hash_chain};
pub use memory::MemoryEventStore;
pub use store::EventStore;

pub type Result<T> = std::result::Result<T, EventSourcingError>;
