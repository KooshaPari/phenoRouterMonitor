//! Rate limiting utilities

pub mod error;

pub use error::{RateLimitError, Result};

/// Rate limiter
pub struct RateLimiter {
    capacity: u64,
    used: u64,
}

impl RateLimiter {
    pub fn new(capacity: u64) -> Self {
        Self { capacity, used: 0 }
    }
    
    pub fn try_acquire(&mut self) -> Result<()> {
        if self.used < self.capacity {
            self.used += 1;
            Ok(())
        } else {
            Err(RateLimitError::RateLimited)
        }
    }
}
