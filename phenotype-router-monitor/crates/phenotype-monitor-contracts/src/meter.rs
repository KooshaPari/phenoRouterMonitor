//! Metering domain contracts

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Core usage meter trait
pub trait UsageMeter: Send + Sync {
    fn record_request(&self, req: RequestMetadata) -> crate::error::Result<()>;
    fn check_quota(&self, user_id: &str, endpoint: &str) -> crate::error::Result<QuotaStatus>;
    fn report(&self) -> crate::error::Result<UsageReport>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    pub user_id: String,
    pub endpoint: String,
    pub method: String,
    pub tokens_used: u32,
    pub latency_ms: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuotaStatus {
    Allowed { remaining: u32 },
    Exceeded { reset_at: DateTime<Utc> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageReport {
    pub user_id: String,
    pub usage: u64,
    pub limit: u64,
    pub period: String,
}
