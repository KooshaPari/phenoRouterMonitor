//! Stub implementations for test doubles

use std::sync::{Arc, Mutex};

/// Type alias for the stub function storage
type StubFunc<I, O> = Arc<Mutex<Box<dyn Fn(I) -> O + Send>>>;

/// A simple stub that returns a fixed value or executes a closure
#[derive(Clone)]
pub struct Stub<I, O> {
    func: StubFunc<I, O>,
    call_count: Arc<Mutex<usize>>,
    recorded_calls: Arc<Mutex<Vec<I>>>,
}

impl<I, O> Stub<I, O>
where
    I: Clone + 'static,
    O: Clone + Send + 'static,
{
    /// Create a new stub with a function
    pub fn new<F>(func: F) -> Self
    where
        F: Fn(I) -> O + Send + 'static,
    {
        Self {
            func: Arc::new(Mutex::new(Box::new(func))),
            call_count: Arc::new(Mutex::new(0)),
            recorded_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a stub that always returns a fixed value
    pub fn returns(value: O) -> Self
    where
        O: Clone,
    {
        Self::new(move |_input: I| value.clone())
    }

    /// Call the stub
    pub fn call(&self, input: I) -> O {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;

        let mut calls = self.recorded_calls.lock().unwrap();
        calls.push(input.clone());

        let func = self.func.lock().unwrap();
        (func)(input)
    }

    /// Get the number of times the stub was called
    pub fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    /// Get all recorded calls
    pub fn recorded_calls(&self) -> Vec<I> {
        self.recorded_calls.lock().unwrap().clone()
    }

    /// Reset the stub's call tracking
    pub fn reset(&self) {
        *self.call_count.lock().unwrap() = 0;
        self.recorded_calls.lock().unwrap().clear();
    }

    /// Change the stub's function
    pub fn set_func<F>(&self, func: F)
    where
        F: Fn(I) -> O + Send + 'static,
    {
        let mut f = self.func.lock().unwrap();
        *f = Box::new(func);
    }
}

impl<I, O> Default for Stub<I, O>
where
    I: Clone + Default + 'static,
    O: Clone + Default + Send + 'static,
{
    fn default() -> Self {
        Self::new(|_| Default::default())
    }
}

/// Builder for creating stubs
#[derive(Debug, Default)]
pub struct StubBuilder {
    sequence: Vec<Box<dyn std::any::Any>>,
}

impl StubBuilder {
    /// Create a new stub builder
    pub fn new() -> Self {
        Self {
            sequence: Vec::new(),
        }
    }

    /// Add a return value to the sequence
    pub fn then_return<T: std::any::Any + Clone + 'static>(mut self, value: T) -> Self {
        self.sequence.push(Box::new(value));
        self
    }

    /// Add an error to the sequence
    pub fn then_error<E: std::any::Any + Clone + 'static>(mut self, error: E) -> Self {
        self.sequence.push(Box::new(error));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_new() {
        let stub = Stub::new(|x: i32| x * 2);
        assert_eq!(stub.call(5), 10);
    }

    #[test]
    fn test_stub_returns() {
        let stub = Stub::<(), i32>::returns(42);
        assert_eq!(stub.call(()), 42);
        assert_eq!(stub.call(()), 42);
    }

    #[test]
    fn test_stub_call_count() {
        let stub = Stub::new(|x: i32| x + 1);
        stub.call(1);
        stub.call(2);
        stub.call(3);
        assert_eq!(stub.call_count(), 3);
    }

    #[test]
    fn test_stub_recorded_calls() {
        let stub = Stub::new(|x: i32| x * 2);
        stub.call(1);
        stub.call(2);
        stub.call(3);
        assert_eq!(stub.recorded_calls(), vec![1, 2, 3]);
    }

    #[test]
    fn test_stub_reset() {
        let stub = Stub::new(|x: i32| x);
        stub.call(1);
        stub.call(2);
        assert_eq!(stub.call_count(), 2);

        stub.reset();
        assert_eq!(stub.call_count(), 0);
        assert!(stub.recorded_calls().is_empty());
    }

    #[test]
    fn test_stub_set_func() {
        let stub = Stub::new(|x: i32| x + 1);
        assert_eq!(stub.call(5), 6);

        stub.set_func(|x| x * 2);
        assert_eq!(stub.call(5), 10);
    }

    #[test]
    fn test_stub_default() {
        let stub: Stub<(), i32> = Stub::default();
        assert_eq!(stub.call(()), 0);
    }
}
