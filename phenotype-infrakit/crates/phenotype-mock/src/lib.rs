//! phenotype-mock
//!
//! Mock trait generators and test doubles for Rust.

use std::sync::{Arc, Mutex};

/// A generic stub for mocking functions.
pub struct Stub<I, O> {
    func: Arc<Mutex<dyn Fn(I) -> O + Send + Sync + 'static>>,
    call_count: Arc<Mutex<u64>>,
    recorded_calls: Arc<Mutex<Vec<I>>>,
}

impl<I: Clone, O> Stub<I, O> {
    /// Create a new stub from a function
    pub fn new<F>(func: F) -> Self
    where
        F: Fn(I) -> O + Send + Sync + 'static,
    {
        Self {
            func: Arc::new(Mutex::new(func)),
            call_count: Arc::new(Mutex::new(0)),
            recorded_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Call the stub with an input
    pub fn call(&self, input: I) -> O {
        {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;
        }
        {
            let mut calls = self.recorded_calls.lock().unwrap();
            calls.push(input.clone());
        }
        let func = self.func.lock().unwrap();
        func(input)
    }

    /// Get the call count
    pub fn call_count(&self) -> u64 {
        *self.call_count.lock().unwrap()
    }

    /// Get the recorded calls
    pub fn recorded_calls(&self) -> Vec<I> {
        self.recorded_calls.lock().unwrap().clone()
    }

    /// Reset the stub
    pub fn reset(&self) {
        *self.call_count.lock().unwrap() = 0;
        self.recorded_calls.lock().unwrap().clear();
    }
}

impl<I: Clone, O: Default + Send + 'static> Default for Stub<I, O>
where
    I: Clone + Send + 'static,
{
    fn default() -> Self {
        Self::new(|_| O::default())
    }
}

impl<I: Clone, O> Clone for Stub<I, O> {
    fn clone(&self) -> Self {
        Self {
            func: self.func.clone(),
            call_count: self.call_count.clone(),
            recorded_calls: self.recorded_calls.clone(),
        }
    }
}

/// Create a new stub
pub fn stub<T, R>(func: impl Fn(T) -> R + Send + Sync + 'static) -> Stub<T, R>
where
    T: Clone + Send + 'static,
    R: Clone + Send + 'static,
{
    Stub::new(func)
}

/// A spy for recording function calls
#[derive(Debug, Default)]
pub struct Spy<T> {
    calls: Arc<Mutex<Vec<T>>>,
}

impl<T: Clone + Send> Spy<T> {
    /// Create a new spy
    pub fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record a call
    pub fn record(&self, call: T) {
        self.calls.lock().unwrap().push(call);
    }

    /// Get all recorded calls
    pub fn calls(&self) -> Vec<T> {
        self.calls.lock().unwrap().clone()
    }

    /// Get the number of calls
    pub fn count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    /// Clear all recorded calls
    pub fn clear(&self) {
        self.calls.lock().unwrap().clear();
    }
}

impl<T> Clone for Spy<T> {
    fn clone(&self) -> Self {
        Self {
            calls: self.calls.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_basic() {
        let stub = Stub::new(|x: i32| x * 2);
        assert_eq!(stub.call(5), 10);
        assert_eq!(stub.call_count(), 1);
    }

    #[test]
    fn test_stub_records_calls() {
        let stub = Stub::new(|x: i32| x * 2);
        stub.call(1);
        stub.call(2);
        stub.call(3);
        assert_eq!(stub.recorded_calls(), vec![1, 2, 3]);
    }

    #[test]
    fn test_stub_reset() {
        let stub = Stub::new(|x: i32| x * 2);
        stub.call(5);
        stub.reset();
        assert_eq!(stub.call_count(), 0);
        assert!(stub.recorded_calls().is_empty());
    }

    #[test]
    fn test_spy_records_calls() {
        let spy = Spy::new();
        spy.record(1);
        spy.record(2);
        assert_eq!(spy.count(), 2);
        assert_eq!(spy.calls(), vec![1, 2]);
    }

    #[test]
    fn test_spy_clear() {
        let spy = Spy::new();
        spy.record(1);
        spy.clear();
        assert_eq!(spy.count(), 0);
    }
}
