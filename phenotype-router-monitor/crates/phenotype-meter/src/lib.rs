//! # phenotype-meter
//!
//! API metering, quota enforcement, rate limiting, and usage analytics.
//! Consolidates metering patterns from AgilePlus, thegent, and custom monitoring tools.
//!
//! ## Key Features
//! - Request classification (by user, endpoint, method, cost)
//! - Quota enforcement with per-user and per-endpoint limits
//! - Rate limiting (token bucket and sliding window)
//! - Usage analytics and reporting
//! - Cost calculation (token-based, time-based, or hybrid)
//! - Integration with phenotype-metrics for observability

pub mod meter;
pub mod quota;
pub mod rate_limit;
pub mod analytics;
pub mod cost;
pub mod classification;

pub use meter::UsageMeter;
pub use quota::QuotaStatus;
pub use rate_limit::RateLimiter;
