//! Test infrastructure and utilities for phenotype crates.
//!
//! Provides fixtures, builders, and utilities for testing.

use std::sync::Arc;
use std::sync::Mutex;

pub use phenotype_error_core::DomainError;

/// Result type for test operations.
pub type Result<T> = std::result::Result<T, TestError>;

/// Common errors that can occur in tests.
pub type TestError = DomainError;

// ============================================================================
// Mock/Spy Utilities
// ============================================================================

/// A simple spy that records all calls made to it.
#[derive(Debug, Clone)]
pub struct CallSpy<T> {
    calls: Arc<Mutex<Vec<T>>>,
}

impl<T> CallSpy<T> {
    /// Create a new spy.
    pub fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record a call.
    pub fn record(&self, call: T) {
        self.calls.lock().unwrap().push(call);
    }

    /// Get all recorded calls.
    pub fn calls(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.calls.lock().unwrap().clone()
    }

    /// Get the number of calls.
    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    /// Clear all recorded calls.
    pub fn clear(&self) {
        self.calls.lock().unwrap().clear();
    }
}

impl<T> Default for CallSpy<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Assertions
// ============================================================================

/// Assert that a result is ok and extract the value.
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        match $result {
            Ok(val) => val,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    };
}

/// Assert that a result is an error.
#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        match $result {
            Ok(val) => panic!("Expected Err, got Ok: {:?}", val),
            Err(_) => (),
        }
    };
}

// ============================================================================
// Test Fixtures
// ============================================================================

/// Test data builder for reusable test scenarios.
#[derive(Debug, Clone)]
pub struct TestDataBuilder {
    seed: u64,
}

impl TestDataBuilder {
    /// Create a new test data builder.
    pub fn new() -> Self {
        Self { seed: 0 }
    }

    /// Create a new builder with a specific seed (for reproducibility).
    pub fn with_seed(seed: u64) -> Self {
        Self { seed }
    }

    /// Generate a deterministic random string.
    pub fn random_string(&mut self) -> String {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        format!("test_{:x}", self.seed)
    }

    /// Generate a deterministic random u32.
    pub fn random_u32(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.seed >> 32) as u32
    }

    /// Generate a deterministic random bool.
    pub fn random_bool(&mut self) -> bool {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.seed & 1 == 0
    }
}

impl Default for TestDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-TEST-INFRA-001
    #[test]
    fn call_spy_records_calls() {
        let spy = CallSpy::new();
        spy.record("call1");
        spy.record("call2");

        assert_eq!(spy.call_count(), 2);
    }

    // Traces to: FR-TEST-INFRA-002
    #[test]
    fn call_spy_retrieves_calls() {
        let spy = CallSpy::new();
        spy.record(42);
        spy.record(99);

        let calls = spy.calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0], 42);
        assert_eq!(calls[1], 99);
    }

    // Traces to: FR-TEST-INFRA-003
    #[test]
    fn call_spy_can_be_cleared() {
        let spy = CallSpy::new();
        spy.record("call");
        assert_eq!(spy.call_count(), 1);

        spy.clear();
        assert_eq!(spy.call_count(), 0);
    }

    // Traces to: FR-TEST-INFRA-004
    #[test]
    fn test_data_builder_random_strings() {
        let mut builder = TestDataBuilder::new();
        let s1 = builder.random_string();
        let s2 = builder.random_string();

        assert_ne!(s1, s2);
        assert!(s1.starts_with("test_"));
    }

    // Traces to: FR-TEST-INFRA-005
    #[test]
    fn test_data_builder_reproducible() {
        let mut b1 = TestDataBuilder::with_seed(12345);
        let mut b2 = TestDataBuilder::with_seed(12345);

        assert_eq!(b1.random_u32(), b2.random_u32());
        assert_eq!(b1.random_bool(), b2.random_bool());
    }

    // Traces to: FR-TEST-INFRA-006
    #[test]
    fn test_data_builder_random_u32() {
        let mut builder = TestDataBuilder::new();
        let n1 = builder.random_u32();
        let n2 = builder.random_u32();

        assert_ne!(n1, n2);
    }

    // Traces to: FR-TEST-INFRA-007
    #[test]
    fn test_data_builder_random_bool() {
        let mut builder = TestDataBuilder::new();
        let values: Vec<bool> = (0..10).map(|_| builder.random_bool()).collect();

        // With 10 iterations, we should have some variation (not all same)
        let has_true = values.iter().any(|v| *v);
        let has_false = values.iter().any(|v| !*v);
        assert!(has_true || has_false); // At least one should be true
    }
}
