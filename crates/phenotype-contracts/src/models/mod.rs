//! # Domain Models
//!
//! Shared domain models and value objects.
//!
//! ## Types
//!
//! - **Value Objects**: Immutable objects defined by their attributes
//! - **Entities**: Objects with identity that persists over time
//! - **Aggregates**: Cluster of related entities and value objects
//!
//! ## Design Principles
//!
//! - **Value objects** are compared by their attributes, not identity
//! - **Entities** have unique IDs and are mutable
//! - **Aggregates** define consistency boundaries

pub mod value_object;
pub mod entity;
pub mod aggregate;

pub use value_object::ValueObject;
pub use entity::{Entity, EntityExt};
pub use aggregate::AggregateRoot;
