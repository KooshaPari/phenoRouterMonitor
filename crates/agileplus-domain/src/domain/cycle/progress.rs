//! Cycle progress summary types.

use serde::{Deserialize, Serialize};

/// Aggregate count of work packages per state across all assigned features.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WpProgressSummary {
    pub total: u32,
    pub planned: u32,
    pub in_progress: u32,
    pub done: u32,
    pub blocked: u32,
}
