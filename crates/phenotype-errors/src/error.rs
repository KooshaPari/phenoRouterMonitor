//! Error types for phenotype-errors

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("internal: {0}")]
    Internal(String),
}
