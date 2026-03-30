//! Error types
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config error: {0}")]
    Invalid(String),
}
