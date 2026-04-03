//! Rate limiting for Phenotype

use std::sync::atomic::{AtomicU64, Ordering};

/// Token bucket rate limiter
pub struct TokenBucket {
    tokens: AtomicU64,
    #[allow(dead_code)]
    max_tokens: u64,
}

impl TokenBucket {
    /// Create a new token bucket
    pub fn new(max_tokens: u64) -> Self {
        Self {
            tokens: AtomicU64::new(max_tokens),
            max_tokens,
        }
    }

    /// Try to acquire a token
    pub fn try_acquire(&self) -> bool {
        let current = self.tokens.load(Ordering::Relaxed);
        if current > 0 {
            self.tokens.fetch_sub(1, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
}
