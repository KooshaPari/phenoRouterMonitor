//! AgilePlus cache layer — Dragonfly (Redis-compatible) adapter.
//!
//! Provides connection pooling, typed cache operations, projection caching,
//! rate limiting, and health checks.
//! Traceability: FR-CACHE / WP04

pub mod config;
pub mod health;
pub mod limiter;
pub mod pool;
pub mod projection;
pub mod store;

pub use config::CacheConfig;
pub use health::{CacheHealth, CacheHealthChecker};
pub use limiter::RateLimiter;
pub use pool::CachePool;
pub use projection::ProjectionCache;
pub use store::{CacheError, CacheStore, RedisCacheStore};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),
    #[error("Config error: {0}")]
    Config(String),
}
