//! Mock function implementations

use super::{MockError, Result, Verifiable};
use std::sync::{Arc, Mutex};

/// A generic mock function that can be configured with expectations
#[derive(Clone)]
pub struct MockFn<I, O> {
    calls: Arc<Mutex<Vec<I>>>,
    results: Arc<Mutex<Vec<O>>>,
    expected_calls: Arc<Mutex<usize>>,
}

impl<I, O> MockFn<I, O>
where
    I: Clone + PartialEq + std::fmt::Debug + 'static,
    O: Clone + 'static,
{
    /// Create a new mock function
    pub fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
            results: Arc::new(Mutex::new(Vec::new())),
            expected_calls: Arc::new(Mutex::new(0)),
        }
    }

    /// Configure an expectation for this mock
    pub fn expect_call(&mut self) -> MockExpectation<I, O> {
        MockExpectation {
            mock: self,
            expected_arg: None,
            return_value: None,
        }
    }

    /// Configure the mock to return a specific value
    pub fn returns(&mut self, value: O) {
        let mut results = self.results.lock().unwrap();
        results.push(value);
    }

    /// Call the mock function
    pub fn call(&self, input: I) -> O {
        let mut calls = self.calls.lock().unwrap();
        calls.push(input.clone());

        let results = self.results.lock().unwrap();
        let index = calls.len() - 1;
        if index < results.len() {
            results[index].clone()
        } else if !results.is_empty() {
            results.last().unwrap().clone()
        } else {
            panic!("No return value configured for mock call")
        }
    }

    /// Get the number of times the mock was called
    pub fn call_count(&self) -> usize {
        let calls = self.calls.lock().unwrap();
        calls.len()
    }

    /// Get all recorded calls
    pub fn recorded_calls(&self) -> Vec<I> {
        let calls = self.calls.lock().unwrap();
        calls.clone()
    }

    /// Check if a specific argument was called
    pub fn was_called_with(&self, arg: &I) -> bool {
        let calls = self.calls.lock().unwrap();
        calls.contains(arg)
    }

    /// Set the expected number of calls
    pub fn expect_calls(&mut self, count: usize) {
        let mut expected = self.expected_calls.lock().unwrap();
        *expected = count;
    }
}

impl<I, O> Verifiable for MockFn<I, O>
where
    I: Clone + PartialEq + std::fmt::Debug + 'static,
    O: Clone + 'static,
{
    fn verify(&self) -> Result<()> {
        let calls = self.calls.lock().unwrap();
        let expected = self.expected_calls.lock().unwrap();

        if *expected > 0 && calls.len() != *expected {
            return Err(MockError::WrongCallCount {
                expected: *expected,
                actual: calls.len(),
            });
        }

        Ok(())
    }
}

impl<I, O> Default for MockFn<I, O>
where
    I: Clone + PartialEq + std::fmt::Debug + 'static,
    O: Clone + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for configuring mock expectations
pub struct MockExpectation<'a, I, O>
where
    I: Clone + PartialEq + std::fmt::Debug + 'static,
    O: Clone + 'static,
{
    mock: &'a mut MockFn<I, O>,
    expected_arg: Option<I>,
    return_value: Option<O>,
}

impl<'a, I, O> MockExpectation<'a, I, O>
where
    I: Clone + PartialEq + std::fmt::Debug + 'static,
    O: Clone + 'static,
{
    /// Set the expected argument
    pub fn with(mut self, arg: I) -> Self {
        self.expected_arg = Some(arg);
        self
    }

    /// Set the return value
    pub fn returns(mut self, value: O) -> Self {
        self.return_value = Some(value);
        self
    }
}

impl<'a, I, O> Drop for MockExpectation<'a, I, O>
where
    I: Clone + PartialEq + std::fmt::Debug + 'static,
    O: Clone + 'static,
{
    fn drop(&mut self) {
        if let Some(ref value) = self.return_value {
            self.mock.returns(value.clone());
        }
    }
}

/// Result builder for mock calls
#[derive(Debug)]
pub struct MockResult<T> {
    value: T,
    times: usize,
}

impl<T> MockResult<T> {
    /// Create a new mock result
    pub fn new(value: T) -> Self {
        Self { value, times: 1 }
    }

    /// Set the number of times this result should be returned
    pub fn times(mut self, count: usize) -> Self {
        self.times = count;
        self
    }
}

impl<T> From<T> for MockResult<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// A single recorded mock call
#[derive(Debug, Clone, PartialEq)]
pub struct MockCall<I, O> {
    input: I,
    output: O,
}

impl<I, O> MockCall<I, O> {
    /// Create a new mock call
    pub fn new(input: I, output: O) -> Self {
        Self { input, output }
    }

    /// Get the input
    pub fn input(&self) -> &I {
        &self.input
    }

    /// Get the output
    pub fn output(&self) -> &O {
        &self.output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_fn_basic() {
        let mut mock = MockFn::<i32, i32>::new();
        mock.returns(42);

        assert_eq!(mock.call(10), 42);
        assert_eq!(mock.call_count(), 1);
    }

    #[test]
    fn test_mock_fn_multiple_returns() {
        let mut mock = MockFn::<i32, i32>::new();
        mock.returns(1);
        mock.returns(2);
        mock.returns(3);

        assert_eq!(mock.call(0), 1);
        assert_eq!(mock.call(0), 2);
        assert_eq!(mock.call(0), 3);
        assert_eq!(mock.call(0), 3);
    }

    #[test]
    fn test_mock_fn_recorded_calls() {
        let mut mock = MockFn::new();
        mock.returns(true);

        mock.call(1);
        mock.call(2);
        mock.call(3);

        assert_eq!(mock.recorded_calls(), vec![1, 2, 3]);
    }

    #[test]
    fn test_mock_fn_was_called_with() {
        let mut mock = MockFn::new();
        mock.returns(());

        mock.call("hello".to_string());
        mock.call("world".to_string());

        assert!(mock.was_called_with(&"hello".to_string()));
        assert!(!mock.was_called_with(&"foo".to_string()));
    }

    #[test]
    fn test_mock_expectation() {
        let mut mock = MockFn::new();
        {
            let _exp = mock.expect_call().with(5).returns(10);
        }

        assert_eq!(mock.call(5), 10);
    }

    #[test]
    fn test_mock_verify() {
        let mut mock = MockFn::new();
        mock.returns(42);
        mock.expect_calls(2);

        mock.call(1);
        mock.call(2);

        assert!(mock.verify().is_ok());
    }

    #[test]
    fn test_mock_verify_failure() {
        let mut mock = MockFn::new();
        mock.returns(42);
        mock.expect_calls(2);

        mock.call(1);

        let result = mock.verify();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MockError::WrongCallCount { expected: 2, actual: 1 }
        ));
    }
}
