//! Phenotype Mock - Mock trait generators and test doubles for Rust
//!
//! This crate provides utilities for creating mock implementations of traits,
//! making it easier to write unit tests with dependency injection.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod fn_mock;
mod mock_builder;
mod stub;

pub use fn_mock::{MockCall, MockFn, MockResult};
pub use mock_builder::{MockBuilder, MockInstance};
pub use stub::{Stub, StubBuilder};

use thiserror::Error;

/// Errors that can occur when working with mocks
#[derive(Error, Debug, Clone, PartialEq)]
pub enum MockError {
    #[error("Unexpected call to mock: {0}")]
    UnexpectedCall(String),
    #[error("Expected call not made: {0}")]
    ExpectedCallNotMade(String),
    #[error("Wrong argument: expected {expected}, got {actual}")]
    WrongArgument { expected: String, actual: String },
    #[error("Wrong call count: expected {expected}, got {actual}")]
    WrongCallCount { expected: usize, actual: usize },
    #[error("No return value configured for call")]
    NoReturnValue,
    #[error("Mock verification failed: {0}")]
    VerificationFailed(String),
}

/// Result type for mock operations
pub type Result<T> = std::result::Result<T, MockError>;

/// Create a mock instance builder
pub fn builder() -> MockBuilder {
    MockBuilder::new()
}

/// Create a new stub
pub fn stub<T, R>(func: impl Fn(T) -> R + Send + 'static) -> Stub<T, R>
where
    T: Clone + 'static,
    R: Clone + Send + 'static,
{
    Stub::new(func)
}

/// Verify that a mock was called
pub fn verify_mock(mock: &impl Verifiable) -> Result<()> {
    mock.verify()
}

/// Trait for verifiable mocks
pub trait Verifiable {
    /// Verify the mock expectations
    fn verify(&self) -> Result<()>;
}

/// Thread-safe mock registry for managing multiple mocks
#[derive(Clone)]
pub struct MockRegistry {
    store: Arc<Mutex<HashMap<TypeId, Box<dyn Any + Send>>>>,
}

impl MockRegistry {
    /// Create a new mock registry
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a mock instance
    pub fn register<T: Any + Send + 'static>(&self, mock: T) {
        let mut store = self.store.lock().unwrap();
        store.insert(TypeId::of::<T>(), Box::new(mock));
    }

    /// Get a registered mock
    pub fn get<T: Any + Clone + 'static>(&self) -> Option<T> {
        let store = self.store.lock().unwrap();
        store
            .get(&TypeId::of::<T>())
            .and_then(|m| m.downcast_ref::<T>())
            .cloned()
    }

    /// Clear all registered mocks
    pub fn clear(&self) {
        let mut store = self.store.lock().unwrap();
        store.clear();
    }
}

impl Default for MockRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_registry() {
        let registry = MockRegistry::new();

        let mock_fn = MockFn::<i32, i32>::new();
        registry.register(mock_fn.clone());

        let retrieved: MockFn<i32, i32> = registry.get().unwrap();
        assert_eq!(retrieved.call_count(), 0);
    }

    #[test]
    fn test_mock_error_display() {
        let err = MockError::UnexpectedCall("test".to_string());
        assert!(err.to_string().contains("Unexpected call"));
    }
}
