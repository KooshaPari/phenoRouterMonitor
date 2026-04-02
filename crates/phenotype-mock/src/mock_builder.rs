//! Mock builder for constructing mock instances

use super::{Result, Verifiable};
use std::any::Any;
use std::collections::HashMap;

/// Builder for creating mock instances with multiple methods
#[derive(Debug, Default)]
pub struct MockBuilder {
    methods: HashMap<String, Box<dyn Any>>,
}

impl MockBuilder {
    /// Create a new mock builder
    pub fn new() -> Self {
        Self {
            methods: HashMap::new(),
        }
    }

    /// Add a method to the mock
    pub fn with_method<T: Any + 'static>(mut self, name: &str, implementation: T) -> Self {
        self.methods.insert(name.to_string(), Box::new(implementation));
        self
    }

    /// Build the mock instance
    pub fn build(self) -> MockInstance {
        MockInstance {
            methods: self.methods,
        }
    }
}

/// A constructed mock instance
#[derive(Debug)]
pub struct MockInstance {
    methods: HashMap<String, Box<dyn Any>>,
}

impl MockInstance {
    /// Create a new empty mock instance
    pub fn new() -> Self {
        Self {
            methods: HashMap::new(),
        }
    }

    /// Get a method implementation
    pub fn get_method<T: Any + Clone>(&self, name: &str) -> Option<T> {
        self.methods
            .get(name)
            .and_then(|m| m.downcast_ref::<T>())
            .cloned()
    }

    /// Set a method implementation
    pub fn set_method<T: Any + 'static>(&mut self, name: &str, implementation: T) {
        self.methods
            .insert(name.to_string(), Box::new(implementation));
    }
}

impl Default for MockInstance {
    fn default() -> Self {
        Self::new()
    }
}

impl Verifiable for MockInstance {
    fn verify(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_builder() {
        let mock = MockBuilder::new()
            .with_method("add", |x: i32| x + 1)
            .with_method("multiply", |x: i32| x * 2)
            .build();

        let add_fn: Option<fn(i32) -> i32> = mock.get_method("add");
        assert!(add_fn.is_some());
    }

    #[test]
    fn test_mock_instance_set_method() {
        let mut mock = MockInstance::new();
        mock.set_method("get", |_: ()| -> i32 { 42 });

        let result: Option<fn(()) -> i32> = mock.get_method("get");
        assert!(result.is_some());
    }
}
