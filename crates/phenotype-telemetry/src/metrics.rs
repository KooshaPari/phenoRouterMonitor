//! Metrics for Phenotype.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Simple metrics counter.
pub struct Counter {
    value: Arc<AtomicU64>,
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

impl Counter {
    pub fn new() -> Self {
        Self { value: Arc::new(AtomicU64::new(0)) }
    }
    pub fn inc(&self) { self.value.fetch_add(1, Ordering::Relaxed); }
    pub fn get(&self) -> u64 { self.value.load(Ordering::Relaxed) }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-PHENO-010
    #[test]
    fn counter_starts_at_zero() {
        let c = Counter::new();
        assert_eq!(c.get(), 0);
    }

    // Traces to: FR-PHENO-010
    #[test]
    fn counter_default_starts_at_zero() {
        let c = Counter::default();
        assert_eq!(c.get(), 0);
    }

    // Traces to: FR-PHENO-010
    #[test]
    fn counter_increments() {
        let c = Counter::new();
        c.inc();
        c.inc();
        c.inc();
        assert_eq!(c.get(), 3);
    }

    // Traces to: FR-PHENO-010
    #[test]
    fn counter_clone_via_arc_shares_state() {
        let c = Counter::new();
        // The inner Arc means cloning the struct shares state
        let c2 = Counter {
            value: Arc::clone(&c.value),
        };
        c.inc();
        c2.inc();
        assert_eq!(c.get(), 2);
        assert_eq!(c2.get(), 2);
    }

    // Traces to: FR-PHENO-010
    #[test]
    fn counter_concurrent_increments() {
        let c = Counter::new();
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let val = Arc::clone(&c.value);
                std::thread::spawn(move || {
                    for _ in 0..100 {
                        val.fetch_add(1, Ordering::Relaxed);
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(c.get(), 1000);
    }
}
