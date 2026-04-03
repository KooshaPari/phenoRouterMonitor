//! Device node domain type — multi-device sync identity and vector clocks.
//!
//! Traceability: FR-051 / WP01-T005

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A device participating in multi-device AgilePlus sync.
///
/// Each device has a unique ID (generated on first run) and maintains
/// a sync vector tracking the last-seen event sequence per entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceNode {
    pub device_id: String,
    pub tailscale_ip: Option<String>,
    pub hostname: String,
    pub last_seen: DateTime<Utc>,
    /// Map of `"entity_type:entity_id"` → last synced sequence number.
    pub sync_vector: serde_json::Value,
    pub platform_version: String,
}

impl DeviceNode {
    pub fn new(device_id: impl Into<String>, hostname: impl Into<String>) -> Self {
        Self {
            device_id: device_id.into(),
            tailscale_ip: None,
            hostname: hostname.into(),
            last_seen: Utc::now(),
            sync_vector: serde_json::json!({}),
            platform_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Update the sync vector for a specific entity stream.
    pub fn update_sync_vector(&mut self, entity_type: &str, entity_id: i64, sequence: i64) {
        let key = format!("{entity_type}:{entity_id}");
        self.sync_vector[key] = serde_json::Value::Number(serde_json::Number::from(sequence));
        self.last_seen = Utc::now();
    }

    /// Get the last synced sequence for an entity stream.
    pub fn get_last_sequence(&self, entity_type: &str, entity_id: i64) -> i64 {
        let key = format!("{entity_type}:{entity_id}");
        self.sync_vector
            .get(&key)
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_device_node() {
        let d = DeviceNode::new("dev-001", "macbook");
        assert_eq!(d.device_id, "dev-001");
        assert_eq!(d.hostname, "macbook");
        assert!(d.tailscale_ip.is_none());
    }

    #[test]
    fn sync_vector_update() {
        let mut d = DeviceNode::new("dev-001", "macbook");
        d.update_sync_vector("feature", 1, 42);
        assert_eq!(d.get_last_sequence("feature", 1), 42);
        assert_eq!(d.get_last_sequence("feature", 2), 0);
    }

    #[test]
    fn device_serde_roundtrip() {
        let mut d = DeviceNode::new("dev-001", "macbook");
        d.tailscale_ip = Some("100.64.0.1".to_string());
        d.update_sync_vector("wp", 5, 100);
        let json = serde_json::to_string(&d).unwrap();
        let d2: DeviceNode = serde_json::from_str(&json).unwrap();
        assert_eq!(d2.get_last_sequence("wp", 5), 100);
        assert_eq!(d2.tailscale_ip.as_deref(), Some("100.64.0.1"));
    }
}
