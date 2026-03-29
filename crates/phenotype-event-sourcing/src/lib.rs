//! phenotype-event-sourcing
//!
//! Append-only event store with SHA-256 hash chain integrity for the Phenotype ecosystem.

pub mod error;
pub mod event;
pub mod hash;
pub mod memory;
pub mod snapshot;
pub mod store;

pub use error::{EventSourcingError, EventStoreError, HashError, Result};
pub use event::EventEnvelope;
pub use memory::InMemoryEventStore;
pub use store::EventStore;
