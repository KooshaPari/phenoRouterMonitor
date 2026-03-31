//! phenotype-health

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, Error>;
