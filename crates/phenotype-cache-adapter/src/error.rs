//! Error types for phenotype-cache-adapter

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for cache operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for cache operations
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum Error {
    #[error("cache error: {0}")]
    Cache(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("not found: {0}")]
    NotFound(String),
}
