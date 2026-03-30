//! Test infrastructure for phenotype crates.

pub use phenotype_error_core::Error;

/// Result type for test operations.
pub type Result<T> = std::result::Result<T, TestError>;

/// Common errors that can occur in tests.
pub type TestError = Error;
