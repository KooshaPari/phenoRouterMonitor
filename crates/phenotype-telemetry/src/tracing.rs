//! Tracing utilities.

pub use phenotype_error_core::ErrorKind;

pub type Result<T> = std::result::Result<T, ErrorKind>;
