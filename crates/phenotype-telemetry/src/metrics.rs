//! Metrics for Phenotype.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Simple metrics counter.
pub struct Counter {
    value: Arc<AtomicU64>,
}

impl Counter {
    pub fn new() -> Self {
        Self { value: Arc::new(AtomicU64::new(0)) }
    }
    pub fn inc(&self) { self.value.fetch_add(1, Ordering::Relaxed); }
    pub fn get(&self) -> u64 { self.value.load(Ordering::Relaxed) }
}
