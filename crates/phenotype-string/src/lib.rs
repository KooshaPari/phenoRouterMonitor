//! phenotype-string

use thiserror::Error;

pub mod compression;
pub mod join;
pub mod normalization;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Invalid(String),
    #[error("Compression error: {0}")]
    Compression(String),
    #[error("Decompression error: {0}")]
    Decompression(String),
}

pub type Result<T> = std::result::Result<T, Error>;
