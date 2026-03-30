//! # Phenotype Event Sourcing
//!
//! Event sourcing primitives for phenotype-infrakit.

pub mod error;
pub mod event;
pub mod hash;
pub mod memory;
pub mod snapshot;
pub mod store;

// Re-export commonly used types
pub use error::{EventSourcingError, EventStoreError, HashError, Result};
pub use event::EventEnvelope;
pub use memory::InMemoryEventStore;
pub use store::EventStore;
