//! Metrics collection utilities

use std::sync::atomic::{AtomicU64, Ordering};

/// Simple counter metric
#[derive(Debug, Default)]
pub struct Counter(AtomicU64);

impl Counter {
    /// Create a new counter
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment the counter by 1
    pub fn increment(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    /// Get the current value
    pub fn get(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}

/// Timer for measuring durations
#[derive(Debug)]
pub struct Timer {
    start: std::time::Instant,
}

impl Timer {
    /// Start a new timer
    pub fn start() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    /// Elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// Record the elapsed time to a metrics collector
    pub fn record_and_stop(self, _name: impl AsRef<str>) -> u64 {
        let ms = self.elapsed_ms();
        // super::record_timing(name, ms);
        ms
    }
}

/// Collector for aggregating multiple metrics
#[derive(Debug, Default)]
pub struct MetricsCollector {
    counters: std::collections::HashMap<String, Counter>,
}

impl MetricsCollector {
    /// Create a new collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create a counter
    pub fn counter(&mut self, name: impl Into<String>) -> &Counter {
        let name = name.into();
        if !self.counters.contains_key(&name) {
            self.counters.insert(name.clone(), Counter::new());
        }
        self.counters.get(&name).unwrap()
    }

    /// Increment a counter by name
    pub fn increment(&mut self, name: impl AsRef<str>) {
        let name = name.as_ref().to_string();
        if let Some(counter) = self.counters.get(&name) {
            counter.increment();
        } else {
            let counter = Counter::new();
            counter.increment();
            self.counters.insert(name, counter);
        }
    }
}
