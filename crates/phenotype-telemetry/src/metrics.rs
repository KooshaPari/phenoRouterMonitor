//! Metrics collection and recording for Phenotype services.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// Atomic counter for metrics.
#[derive(Clone)]
pub struct Counter {
    value: Arc<AtomicU64>,
}

impl Counter {
    /// Create a new counter starting at zero.
    pub fn new() -> Self {
        Self {
            value: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Increment the counter by 1.
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Add a value to the counter.
    pub fn add(&self, value: u64) {
        self.value.fetch_add(value, Ordering::Relaxed);
    }

    /// Get the current counter value.
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Reset the counter to zero.
    pub fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

/// Histogram for recording distribution of values.
#[derive(Clone)]
pub struct Histogram {
    values: Arc<RwLock<Vec<f64>>>,
}

impl Histogram {
    /// Create a new histogram.
    pub fn new() -> Self {
        Self {
            values: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record a value in the histogram.
    pub fn record(&self, value: f64) {
        if let Ok(mut values) = self.values.write() {
            values.push(value);
        }
    }

    /// Get the count of recorded values.
    pub fn count(&self) -> usize {
        self.values.read().map(|v| v.len()).unwrap_or(0)
    }

    /// Get the mean of recorded values.
    pub fn mean(&self) -> f64 {
        let values = self.values.read().unwrap_or_else(|e| e.into_inner());
        if values.is_empty() {
            return 0.0;
        }
        values.iter().sum::<f64>() / values.len() as f64
    }

    /// Get the minimum recorded value.
    pub fn min(&self) -> Option<f64> {
        self.values
            .read()
            .ok()
            .and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    v.iter().copied().fold(None, |acc, x| {
                        Some(acc.map_or(x, |min: f64| min.min(x)))
                    })
                }
            })
    }

    /// Get the maximum recorded value.
    pub fn max(&self) -> Option<f64> {
        self.values
            .read()
            .ok()
            .and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    v.iter().copied().fold(None, |acc, x| {
                        Some(acc.map_or(x, |max: f64| max.max(x)))
                    })
                }
            })
    }

    /// Clear all recorded values.
    pub fn reset(&self) {
        if let Ok(mut values) = self.values.write() {
            values.clear();
        }
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics registry for managing counters and histograms.
pub struct Metrics {
    service_name: String,
    counters: Arc<RwLock<HashMap<String, Counter>>>,
    histograms: Arc<RwLock<HashMap<String, Histogram>>>,
}

impl Metrics {
    /// Create a new metrics registry for a service.
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            counters: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a counter metric with optional labels.
    pub fn record_counter(&self, name: &str, value: u64, _labels: &[(&str, &str)]) {
        let counter = self
            .counters
            .write()
            .ok()
            .and_then(|mut c| {
                if !c.contains_key(name) {
                    c.insert(name.to_string(), Counter::new());
                }
                c.get(name).cloned()
            });

        if let Some(counter) = counter {
            counter.add(value);
        }
    }

    /// Record a histogram metric with optional labels.
    pub fn record_histogram(&self, name: &str, value: f64, _labels: &[(&str, &str)]) {
        let histogram = self
            .histograms
            .write()
            .ok()
            .and_then(|mut h| {
                if !h.contains_key(name) {
                    h.insert(name.to_string(), Histogram::new());
                }
                h.get(name).cloned()
            });

        if let Some(histogram) = histogram {
            histogram.record(value);
        }
    }

    /// Get a counter by name.
    pub fn get_counter(&self, name: &str) -> Option<Counter> {
        self.counters.read().ok().and_then(|c| c.get(name).cloned())
    }

    /// Get a histogram by name.
    pub fn get_histogram(&self, name: &str) -> Option<Histogram> {
        self.histograms
            .read()
            .ok()
            .and_then(|h| h.get(name).cloned())
    }

    /// Export metrics as JSON.
    pub fn export_json(&self) -> serde_json::Value {
        let mut counters = serde_json::json!({});
        let mut histograms = serde_json::json!({});

        if let Ok(c) = self.counters.read() {
            for (name, counter) in c.iter() {
                counters[name] = serde_json::json!(counter.get());
            }
        }

        if let Ok(h) = self.histograms.read() {
            for (name, histogram) in h.iter() {
                histograms[name] = serde_json::json!({
                    "count": histogram.count(),
                    "mean": histogram.mean(),
                    "min": histogram.min(),
                    "max": histogram.max(),
                });
            }
        }

        serde_json::json!({
            "service": self.service_name,
            "counters": counters,
            "histograms": histograms,
        })
    }
}

/// Initialize metrics for a service.
pub fn init_metrics(service_name: &str) -> Metrics {
    Metrics::new(service_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_increment() {
        let counter = Counter::new();
        assert_eq!(counter.get(), 0);
        counter.inc();
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn test_counter_add() {
        let counter = Counter::new();
        counter.add(5);
        assert_eq!(counter.get(), 5);
        counter.add(3);
        assert_eq!(counter.get(), 8);
    }

    #[test]
    fn test_counter_reset() {
        let counter = Counter::new();
        counter.add(10);
        counter.reset();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_histogram_record() {
        let histogram = Histogram::new();
        histogram.record(1.0);
        histogram.record(2.0);
        histogram.record(3.0);
        assert_eq!(histogram.count(), 3);
    }

    #[test]
    fn test_histogram_stats() {
        let histogram = Histogram::new();
        histogram.record(1.0);
        histogram.record(5.0);
        histogram.record(9.0);

        assert_eq!(histogram.count(), 3);
        assert_eq!(histogram.mean(), 5.0);
        assert_eq!(histogram.min(), Some(1.0));
        assert_eq!(histogram.max(), Some(9.0));
    }

    #[test]
    fn test_histogram_reset() {
        let histogram = Histogram::new();
        histogram.record(1.0);
        histogram.reset();
        assert_eq!(histogram.count(), 0);
    }

    #[test]
    fn test_metrics_registry() {
        let metrics = Metrics::new("test-service");

        metrics.record_counter("requests", 5, &[]);
        metrics.record_histogram("latency", 42.5, &[]);

        let counter = metrics.get_counter("requests").unwrap();
        assert_eq!(counter.get(), 5);

        let histogram = metrics.get_histogram("latency").unwrap();
        assert_eq!(histogram.count(), 1);
    }

    #[test]
    fn test_metrics_export_json() {
        let metrics = Metrics::new("test-service");
        metrics.record_counter("requests", 10, &[]);
        metrics.record_histogram("latency", 50.0, &[]);

        let json = metrics.export_json();
        assert_eq!(json["service"], "test-service");
        assert_eq!(json["counters"]["requests"], 10);
        assert_eq!(json["histograms"]["latency"]["count"], 1);
    }
}
