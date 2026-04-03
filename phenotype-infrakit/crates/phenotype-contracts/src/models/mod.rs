//! Domain models for Phenotype contracts.

/// Marker trait for domain entities
pub trait Entity: Send + Sync {}

/// Marker trait for value objects
pub trait ValueObject: Send + Sync {}

/// Marker trait for aggregate roots
pub trait AggregateRoot: Send + Sync {}
