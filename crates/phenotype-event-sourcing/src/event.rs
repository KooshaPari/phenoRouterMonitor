//! Event types for event sourcing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T = serde_json::Value> {
    pub sequence: i64,
    pub timestamp: DateTime<Utc>,
    pub event: T,
    pub entity_type: String,
    pub entity_id: String,
    pub user_id: Option<String>,
    pub prev_hash: String,
    pub hash: String,
}
