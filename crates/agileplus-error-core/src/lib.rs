//! # AgilePlus Error Core
//!
//! Unified error handling for the AgilePlus ecosystem.
//!
//! This crate provides domain-specific error types and conversions
//! to the canonical `phenotype_error_core::ErrorKind`.

pub mod api;
pub mod domain;
pub mod serialization;
pub mod storage;
pub mod sync;
pub mod traits;

// Re-export domain-specific error types
pub use api::ApiError;
pub use domain::DomainError;
pub use serialization::SerializationError;
pub use storage::StorageError;
pub use sync::SyncError;
pub use traits::NotFoundMarker;
