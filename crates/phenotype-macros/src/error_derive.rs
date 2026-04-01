//! Procedural macros for error type derivation.

// This module provides utilities for error handling patterns used in Phenotype crates.
// The actual derive macros are from `thiserror` crate.

// Re-export derive macro for convenience
#[doc(hidden)]
pub use thiserror::Error;

/// Marker trait for error types that can be used across crate boundaries.
pub trait CrossCrateError: std::error::Error + Send + Sync + 'static {}

impl<T: std::error::Error + Send + Sync + 'static> CrossCrateError for T {}
