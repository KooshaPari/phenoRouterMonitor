//! phenotype-retry
//! phenotype-retry
//!
//! Retry utilities.

pub mod error;
pub use error::*;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
}
impl Default for RetryConfig {
    fn default() -> Self {
        Self { max_attempts: 3 }
    }
}
