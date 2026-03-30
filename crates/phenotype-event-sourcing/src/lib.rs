//! # Phenotype Event Sourcing
//!
//! Event sourcing primitives for phenotype-infrakit.

pub mod error;
pub mod event;
pub mod hash;
pub mod memory;
pub mod snapshot;
pub mod store;

pub use error::{EventSourcingError, EventStoreError, HashError, Result};
pub use event::EventEnvelope;
pub use hash::{compute_event_hash, verify_event_hash};
pub use memory::InMemoryEventStore;
pub use store::{EventStore, JsonEnvelope};
