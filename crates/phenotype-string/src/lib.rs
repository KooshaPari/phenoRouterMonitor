//! phenotype-string: Comprehensive string manipulation utilities
//!
//! This crate provides pure, stateless utilities for string manipulation including:
//! - Compression/decompression with zstd
//! - Unicode normalization (NFD, NFC)
//! - Case conversion (snake_case, camelCase, PascalCase, kebab-case)
//! - String trimming and slugification
//!
//! All functions are pure and thread-safe.

use thiserror::Error;

/// Error type for string operations
#[derive(Debug, Error)]
pub enum Error {
    /// Invalid input string
    #[error("Invalid string: {0}")]
    Invalid(String),

    /// Compression error
    #[error("Compression error: {0}")]
    CompressionError(String),

    /// Decompression error
    #[error("Decompression error: {0}")]
    DecompressionError(String),
}

/// Result type for string operations
pub type Result<T> = std::result::Result<T, Error>;

pub mod case;
pub mod compression;
pub mod normalization;
pub mod builder;

// Re-export commonly used items
pub use case::{to_camel_case, to_pascal_case, to_kebab_case, to_snake_case, CaseConverter};
pub use compression::{compress, decompress};
pub use normalization::{normalize_nfc, normalize_nfd, slugify, trim_whitespace};
pub use builder::StringBuilder;
