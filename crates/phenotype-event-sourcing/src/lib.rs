//! # Phenotype Event Sourcing
//!
//! Event sourcing primitives for phenotype-infrakit.

pub mod async_store;
pub mod error;
pub mod event;
pub mod hash;
pub mod memory;
pub mod snapshot;
pub mod store;

pub use error::{EventSourcingError, Result};
pub use hash::*;
pub use memory::*;
pub use snapshot::*;
pub use store::*;
