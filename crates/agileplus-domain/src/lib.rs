//! # AgilePlus Domain
//!
//! Core domain model for AgilePlus: entities, value objects, and port interfaces.
//!
//! Repository adapters typically use [`StorageError`] (or related types from
//! [`agileplus_error_core`]) as `ProjectRepository::Error`. At HTTP or CLI
//! boundaries, convert into [`ErrorKind`] for consistent logging and responses.

pub mod entities;
pub mod ports;
pub mod values;

/// Canonical AgilePlus domain and persistence errors (re-exported for adapters).
pub use agileplus_error_core::{
    ApiError, DomainError, ErrorKind, NotFoundMarker, SerializationError, StorageError, SyncError,
};

/// Convenience result for domain-rule failures in application services.
pub type DomainResult<T> = std::result::Result<T, DomainError>;
