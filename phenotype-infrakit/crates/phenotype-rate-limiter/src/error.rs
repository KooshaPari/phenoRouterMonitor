//! Rate limiting errors

use thiserror::Error;

/// Rate limit error
#[derive(Error, Debug)]
pub enum RateLimitError {
    #[error("rate limited")]
    RateLimited,
}

/// Rate limit result
pub type Result<T> = std::result::Result<T, RateLimitError>;
