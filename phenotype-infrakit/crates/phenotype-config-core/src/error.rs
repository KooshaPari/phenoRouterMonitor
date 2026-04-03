//! Configuration errors

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("missing key: {0}")]
    MissingKey(String),
}

pub type ConfigResult<T> = Result<T, ConfigError>;
