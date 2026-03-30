//! phenotype-event-sourcing
//! 
//! Append-only event store with SHA-256 hash chain integrity.

pub mod error;
pub mod event;
pub mod hash;
pub mod memory;
pub mod store;

pub use error::{EventSourcingError, Result};
pub use event::EventEnvelope;
pub use hash::{compute_event_hash, verify_event_hash};
pub use memory::InMemoryEventStore;
pub use store::EventStore;
