//! # Phenotype Event Sourcing
//!
//! Event sourcing primitives for phenotype-infrakit.

pub mod error;
pub mod hash;
pub mod memory;
pub mod snapshot;
pub mod store;

pub use error::{Error, EventSourcingError};
pub use hash::*;
pub use memory::*;
pub use snapshot::*;
pub use store::*;
