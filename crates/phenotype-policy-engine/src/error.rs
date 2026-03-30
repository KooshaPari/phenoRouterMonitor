//! Error types for phenotype-policy-engine

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for policy operations
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum Error {
    #[error("policy error: {0}")]
    Policy(String),
}
