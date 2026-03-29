//! # Models Module
//!
//! Domain models, entities, value objects, and aggregates.

pub mod aggregate;
pub mod entity;
pub mod value_object;

pub use aggregate::{AggregateRoot, DomainEvent, AggregateExt};
pub use entity::{Entity, EntityExt};
pub use value_object::ValueObject;
