//! Test spy implementation for recording and verifying calls

/// A spy that records method calls for verification in tests
#[derive(Debug, Clone)]
pub struct Spy<T> {
    calls: Vec<T>,
}

impl<T> Spy<T> {
    /// Create a new spy
    pub fn new() -> Self {
        Self { calls: Vec::new() }
    }

    /// Record a call
    pub fn record(&mut self, call: T) {
        self.calls.push(call);
    }

    /// Get all recorded calls
    pub fn calls(&self) -> &[T] {
        &self.calls
    }

    /// Get the number of calls
    pub fn call_count(&self) -> usize {
        self.calls.len()
    }

    /// Clear all recorded calls
    pub fn clear(&mut self) {
        self.calls.clear();
    }
}

impl<T> Default for Spy<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Assertions for spy verification
pub trait SpyAssertions<T> {
    /// Assert that the spy was called exactly once
    fn assert_called_once(&self);

    /// Assert that the spy was called exactly n times
    fn assert_called_times(&self, times: usize);

    /// Assert that the spy was called with a specific argument
    fn assert_called_with(&self, expected: &T)
    where
        T: PartialEq;
}

impl<T: std::fmt::Debug + PartialEq> SpyAssertions<T> for Spy<T> {
    fn assert_called_once(&self) {
        assert_eq!(
            self.call_count(),
            1,
            "Expected spy to be called once, but was called {} times",
            self.call_count()
        );
    }

    fn assert_called_times(&self, times: usize) {
        assert_eq!(
            self.call_count(),
            times,
            "Expected spy to be called {} times, but was called {} times",
            times,
            self.call_count()
        );
    }

    fn assert_called_with(&self, expected: &T) {
        assert!(
            self.calls().contains(expected),
            "Expected spy to be called with {:?}",
            expected
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spy_records_calls() {
        let mut spy = Spy::new();
        spy.record("call1");
        spy.record("call2");

        assert_eq!(spy.call_count(), 2);
        assert_eq!(spy.calls(), &["call1", "call2"]);
    }

    #[test]
    fn test_spy_assertions() {
        let mut spy = Spy::new();
        spy.record("test");

        spy.assert_called_once();
        spy.assert_called_with(&"test");
    }

    #[test]
    fn test_spy_clear() {
        let mut spy = Spy::new();
        spy.record(1);
        spy.record(2);
        assert_eq!(spy.call_count(), 2);

        spy.clear();
        assert_eq!(spy.call_count(), 0);
        assert!(spy.calls().is_empty());
    }

    #[test]
    fn test_spy_called_times() {
        let mut spy = Spy::new();
        spy.record("a");
        spy.record("b");
        spy.record("c");

        spy.assert_called_times(3);
    }
}
