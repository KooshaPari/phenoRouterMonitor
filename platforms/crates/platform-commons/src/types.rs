//! Common types

use serde::{Deserialize, Serialize};

/// Platform identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub name: String,
    pub version: String,
}

/// Configuration for cross-platform features
#[derive(Debug, Clone)]
pub struct PlatformConfig {
    pub max_retries: u32,
    pub timeout_secs: u64,
}
