//! # Phenotype Event Sourcing
//!
//! Event sourcing primitives for phenotype-infrakit.

pub mod error;
pub mod event;
pub mod hash;
pub mod memory;
pub mod snapshot;
pub mod store;

pub use error::EventSourcingError;
pub use event::EventEnvelope;
pub use store::EventStore;
