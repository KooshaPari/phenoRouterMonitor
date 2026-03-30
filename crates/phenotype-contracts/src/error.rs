//! Error types for phenotype-contracts.

pub use phenotype_errors::{ErrorKind, Error, Result};

impl ErrorKind {
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Permission(format!("unauthorized: {}", msg.into()))
    }
}
