//! Point-in-time metrics snapshot for reporting.

use crate::registry::Metric;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Serializable snapshot of metrics at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: u64,
    pub metrics: HashMap<String, Metric>,
}

impl MetricsSnapshot {
    /// Create a new metrics snapshot.
    pub fn new(metrics: HashMap<String, Metric>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self { timestamp, metrics }
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_creation() {
        let mut metrics = HashMap::new();
        metrics.insert(
            "test".into(),
            Metric::Counter { value: 42 },
        );

        let snapshot = MetricsSnapshot::new(metrics);
        assert_eq!(snapshot.metrics.len(), 1);
        assert!(snapshot.timestamp > 0);
    }

    #[test]
    fn snapshot_serialization() {
        let mut metrics = HashMap::new();
        metrics.insert(
            "requests".into(),
            Metric::Counter { value: 100 },
        );

        let snapshot = MetricsSnapshot::new(metrics);
        let json = snapshot.to_json().unwrap();
        assert!(json.contains("requests"));
        assert!(json.contains("100"));
    }
}
