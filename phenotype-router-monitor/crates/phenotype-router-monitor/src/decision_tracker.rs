//! Decision tracking and history for routing decisions.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// A recorded routing decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub path: String,
    pub route_id: String,
    pub backend_id: String,
    pub timestamp: u64,
    pub success: bool,
    pub latency_ms: Option<u64>,
}

/// Tracks routing decisions over time.
pub struct DecisionTracker {
    records: Arc<Mutex<Vec<DecisionRecord>>>,
    max_records: usize,
}

impl DecisionTracker {
    pub fn new(max_records: usize) -> Self {
        Self {
            records: Arc::new(Mutex::new(Vec::new())),
            max_records,
        }
    }

    pub fn default_with_history() -> Self {
        Self::new(10000)
    }

    /// Record a routing decision.
    pub fn record(
        &self,
        path: String,
        route_id: String,
        backend_id: String,
        success: bool,
        latency_ms: Option<u64>,
    ) {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let record = DecisionRecord {
            path,
            route_id,
            backend_id,
            timestamp,
            success,
            latency_ms,
        };

        if let Ok(mut records) = self.records.lock() {
            records.push(record);

            // Trim old records if exceeding max
            if records.len() > self.max_records {
                let to_remove = records.len() - self.max_records;
                records.drain(0..to_remove);
            }
        }
    }

    /// Get all recorded decisions.
    pub fn all_records(&self) -> Vec<DecisionRecord> {
        self.records
            .lock()
            .map(|r| r.clone())
            .unwrap_or_default()
    }

    /// Get records for a specific route.
    pub fn records_for_route(&self, route_id: &str) -> Vec<DecisionRecord> {
        self.records
            .lock()
            .map(|records| {
                records
                    .iter()
                    .filter(|r| r.route_id == route_id)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get records for a specific backend.
    pub fn records_for_backend(&self, backend_id: &str) -> Vec<DecisionRecord> {
        self.records
            .lock()
            .map(|records| {
                records
                    .iter()
                    .filter(|r| r.backend_id == backend_id)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get records for a specific path.
    pub fn records_for_path(&self, path: &str) -> Vec<DecisionRecord> {
        self.records
            .lock()
            .map(|records| {
                records
                    .iter()
                    .filter(|r| r.path == path)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get the last N records.
    pub fn recent_records(&self, count: usize) -> Vec<DecisionRecord> {
        self.records
            .lock()
            .map(|records| {
                let start = if records.len() > count {
                    records.len() - count
                } else {
                    0
                };
                records[start..].to_vec()
            })
            .unwrap_or_default()
    }

    /// Get success rate for a route.
    pub fn success_rate_for_route(&self, route_id: &str) -> f64 {
        let records = self.records_for_route(route_id);
        if records.is_empty() {
            return 0.0;
        }

        let successful = records.iter().filter(|r| r.success).count();
        (successful as f64 / records.len() as f64) * 100.0
    }

    /// Get success rate for a backend.
    pub fn success_rate_for_backend(&self, backend_id: &str) -> f64 {
        let records = self.records_for_backend(backend_id);
        if records.is_empty() {
            return 0.0;
        }

        let successful = records.iter().filter(|r| r.success).count();
        (successful as f64 / records.len() as f64) * 100.0
    }

    /// Get average latency for a route.
    pub fn avg_latency_for_route(&self, route_id: &str) -> Option<f64> {
        let records = self.records_for_route(route_id);
        let with_latency: Vec<u64> = records
            .iter()
            .filter_map(|r| r.latency_ms)
            .collect();

        if with_latency.is_empty() {
            return None;
        }

        let sum: u64 = with_latency.iter().sum();
        Some(sum as f64 / with_latency.len() as f64)
    }

    /// Get average latency for a backend.
    pub fn avg_latency_for_backend(&self, backend_id: &str) -> Option<f64> {
        let records = self.records_for_backend(backend_id);
        let with_latency: Vec<u64> = records
            .iter()
            .filter_map(|r| r.latency_ms)
            .collect();

        if with_latency.is_empty() {
            return None;
        }

        let sum: u64 = with_latency.iter().sum();
        Some(sum as f64 / with_latency.len() as f64)
    }

    /// Clear all records.
    pub fn clear(&self) {
        if let Ok(mut records) = self.records.lock() {
            records.clear();
        }
    }

    /// Get record count.
    pub fn record_count(&self) -> usize {
        self.records
            .lock()
            .map(|r| r.len())
            .unwrap_or(0)
    }
}

impl Clone for DecisionTracker {
    fn clone(&self) -> Self {
        Self {
            records: self.records.clone(),
            max_records: self.max_records,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_creation() {
        let tracker = DecisionTracker::new(100);
        assert_eq!(tracker.record_count(), 0);
    }

    #[test]
    fn test_record_single_decision() {
        let tracker = DecisionTracker::new(100);
        tracker.record(
            "/api/users".to_string(),
            "api-route".to_string(),
            "backend-1".to_string(),
            true,
            Some(50),
        );

        assert_eq!(tracker.record_count(), 1);
    }

    #[test]
    fn test_record_multiple_decisions() {
        let tracker = DecisionTracker::new(100);

        for i in 0..10 {
            tracker.record(
                format!("/api/users/{}", i),
                "api-route".to_string(),
                format!("backend-{}", i % 3 + 1),
                true,
                Some(50 + i as u64),
            );
        }

        assert_eq!(tracker.record_count(), 10);
    }

    #[test]
    fn test_all_records() {
        let tracker = DecisionTracker::new(100);
        tracker.record("/api/users".to_string(), "api".to_string(), "b1".to_string(), true, None);
        tracker.record("/api/posts".to_string(), "api".to_string(), "b2".to_string(), false, None);

        let records = tracker.all_records();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_records_for_route() {
        let tracker = DecisionTracker::new(100);

        tracker.record("/api/users".to_string(), "api".to_string(), "b1".to_string(), true, None);
        tracker.record("/api/users".to_string(), "api".to_string(), "b2".to_string(), true, None);
        tracker.record("/web/index".to_string(), "web".to_string(), "b1".to_string(), true, None);

        let api_records = tracker.records_for_route("api");
        assert_eq!(api_records.len(), 2);

        let web_records = tracker.records_for_route("web");
        assert_eq!(web_records.len(), 1);
    }

    #[test]
    fn test_records_for_backend() {
        let tracker = DecisionTracker::new(100);

        tracker.record("/api/users".to_string(), "api".to_string(), "b1".to_string(), true, None);
        tracker.record("/api/posts".to_string(), "api".to_string(), "b1".to_string(), true, None);
        tracker.record("/web/index".to_string(), "web".to_string(), "b2".to_string(), true, None);

        let b1_records = tracker.records_for_backend("b1");
        assert_eq!(b1_records.len(), 2);

        let b2_records = tracker.records_for_backend("b2");
        assert_eq!(b2_records.len(), 1);
    }

    #[test]
    fn test_records_for_path() {
        let tracker = DecisionTracker::new(100);

        tracker.record("/api/users".to_string(), "api1".to_string(), "b1".to_string(), true, None);
        tracker.record("/api/users".to_string(), "api2".to_string(), "b2".to_string(), true, None);
        tracker.record("/api/posts".to_string(), "api".to_string(), "b1".to_string(), true, None);

        let records = tracker.records_for_path("/api/users");
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_recent_records() {
        let tracker = DecisionTracker::new(100);

        for i in 0..10 {
            tracker.record(
                format!("/api/{}", i),
                "api".to_string(),
                "b1".to_string(),
                true,
                None,
            );
        }

        let recent = tracker.recent_records(5);
        assert_eq!(recent.len(), 5);
    }

    #[test]
    fn test_recent_records_more_than_available() {
        let tracker = DecisionTracker::new(100);

        for i in 0..5 {
            tracker.record(
                format!("/api/{}", i),
                "api".to_string(),
                "b1".to_string(),
                true,
                None,
            );
        }

        let recent = tracker.recent_records(10);
        assert_eq!(recent.len(), 5);
    }

    #[test]
    fn test_success_rate_for_route() {
        let tracker = DecisionTracker::new(100);

        for _ in 0..8 {
            tracker.record("/api/users".to_string(), "api".to_string(), "b1".to_string(), true, None);
        }
        for _ in 0..2 {
            tracker.record("/api/users".to_string(), "api".to_string(), "b2".to_string(), false, None);
        }

        assert_eq!(tracker.success_rate_for_route("api"), 80.0);
    }

    #[test]
    fn test_success_rate_for_backend() {
        let tracker = DecisionTracker::new(100);

        for _ in 0..9 {
            tracker.record("/api/users".to_string(), "api".to_string(), "b1".to_string(), true, None);
        }
        for _ in 0..1 {
            tracker.record("/api/users".to_string(), "api".to_string(), "b1".to_string(), false, None);
        }

        assert_eq!(tracker.success_rate_for_backend("b1"), 90.0);
    }

    #[test]
    fn test_avg_latency_for_route() {
        let tracker = DecisionTracker::new(100);

        tracker.record("/api/users".to_string(), "api".to_string(), "b1".to_string(), true, Some(100));
        tracker.record("/api/users".to_string(), "api".to_string(), "b1".to_string(), true, Some(200));
        tracker.record("/api/users".to_string(), "api".to_string(), "b1".to_string(), true, Some(300));

        let avg = tracker.avg_latency_for_route("api");
        assert_eq!(avg, Some(200.0));
    }

    #[test]
    fn test_avg_latency_for_backend() {
        let tracker = DecisionTracker::new(100);

        tracker.record("/api/users".to_string(), "api".to_string(), "b1".to_string(), true, Some(100));
        tracker.record("/api/posts".to_string(), "api".to_string(), "b1".to_string(), true, Some(200));

        let avg = tracker.avg_latency_for_backend("b1");
        assert_eq!(avg, Some(150.0));
    }

    #[test]
    fn test_max_records_limit() {
        let tracker = DecisionTracker::new(5);

        for i in 0..10 {
            tracker.record(
                format!("/api/{}", i),
                "api".to_string(),
                "b1".to_string(),
                true,
                None,
            );
        }

        assert_eq!(tracker.record_count(), 5);
    }

    #[test]
    fn test_clear_records() {
        let tracker = DecisionTracker::new(100);

        for i in 0..5 {
            tracker.record(
                format!("/api/{}", i),
                "api".to_string(),
                "b1".to_string(),
                true,
                None,
            );
        }

        tracker.clear();
        assert_eq!(tracker.record_count(), 0);
    }

    #[test]
    fn test_tracker_clone() {
        let tracker1 = DecisionTracker::new(100);
        tracker1.record("/api/users".to_string(), "api".to_string(), "b1".to_string(), true, None);

        let tracker2 = tracker1.clone();

        assert_eq!(tracker1.record_count(), 1);
        assert_eq!(tracker2.record_count(), 1);

        tracker2.record("/api/posts".to_string(), "api".to_string(), "b1".to_string(), true, None);

        assert_eq!(tracker1.record_count(), 2);
        assert_eq!(tracker2.record_count(), 2);
    }

    #[test]
    fn test_tracker_thread_safe() {
        let tracker = Arc::new(DecisionTracker::new(1000));
        let mut handles = vec![];

        for t in 0..10 {
            let t_clone = tracker.clone();
            let handle = std::thread::spawn(move || {
                for i in 0..10 {
                    t_clone.record(
                        format!("/api/path-{}-{}", t, i),
                        "api".to_string(),
                        format!("b{}", t),
                        true,
                        Some(50 + i as u64),
                    );
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(tracker.record_count(), 100);
    }
}
