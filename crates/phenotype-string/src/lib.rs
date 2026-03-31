//! phenotype-string

use thiserror::Error;

pub mod join;
pub mod parse;
pub mod sanitize;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, Error>;
