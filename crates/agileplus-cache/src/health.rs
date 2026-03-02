//! Dragonfly/Redis health check.

use crate::pool::CachePool;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheHealth {
    Healthy,
    Unavailable,
}

pub struct CacheHealthChecker {
    pool: CachePool,
}

impl CacheHealthChecker {
    pub fn new(pool: CachePool) -> Self {
        Self { pool }
    }

    pub async fn check(&self) -> CacheHealth {
        match self.pool.get_connection().await {
            Ok(mut conn) => {
                let result: Result<String, _> = redis::cmd("PING")
                    .query_async(&mut *conn)
                    .await;
                match result {
                    Ok(pong) if pong == "PONG" => CacheHealth::Healthy,
                    _ => CacheHealth::Unavailable,
                }
            }
            Err(_) => CacheHealth::Unavailable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_enum_equality() {
        assert_eq!(CacheHealth::Healthy, CacheHealth::Healthy);
        assert_ne!(CacheHealth::Healthy, CacheHealth::Unavailable);
    }
}
