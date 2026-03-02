//! Redis-based sliding window rate limiter.

use crate::pool::CachePool;
use redis::AsyncCommands;

#[derive(Debug, thiserror::Error)]
pub enum LimiterError {
    #[error("Rate limit error: {0}")]
    Error(String),
}

pub struct RateLimiter {
    pool: CachePool,
}

impl RateLimiter {
    pub fn new(pool: CachePool) -> Self {
        Self { pool }
    }

    /// Check if a request is allowed under the sliding window.
    pub async fn is_allowed(
        &self,
        key: &str,
        max_requests: u32,
        window_secs: u64,
    ) -> Result<bool, LimiterError> {
        let mut conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| LimiterError::Error(e.to_string()))?;

        let rate_key = format!("ratelimit:{key}");

        let count: u32 = conn
            .incr(&rate_key, 1u32)
            .await
            .map_err(|e| LimiterError::Error(e.to_string()))?;

        if count == 1 {
            let _: () = conn
                .expire(&rate_key, window_secs as i64)
                .await
                .map_err(|e| LimiterError::Error(e.to_string()))?;
        }

        Ok(count <= max_requests)
    }

    pub async fn get_remaining(
        &self,
        key: &str,
        max_requests: u32,
    ) -> Result<u32, LimiterError> {
        let mut conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| LimiterError::Error(e.to_string()))?;

        let rate_key = format!("ratelimit:{key}");
        let count: Option<u32> = conn
            .get(&rate_key)
            .await
            .map_err(|e| LimiterError::Error(e.to_string()))?;

        Ok(max_requests.saturating_sub(count.unwrap_or(0)))
    }

    pub async fn reset(&self, key: &str) -> Result<(), LimiterError> {
        let mut conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| LimiterError::Error(e.to_string()))?;

        let rate_key = format!("ratelimit:{key}");
        conn.del::<_, ()>(&rate_key)
            .await
            .map_err(|e| LimiterError::Error(e.to_string()))?;

        Ok(())
    }
}
