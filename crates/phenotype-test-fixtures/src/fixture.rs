//! Test fixtures for reusable test data

use crate::{FixtureError, Result};
use std::collections::HashMap;

/// A test fixture that can be loaded and used in tests
pub trait Fixture: Sized {
    /// The type of data this fixture provides
    type Data;

    /// Load the fixture from a source
    fn load() -> Result<Self>;

    /// Get the fixture data
    fn data(&self) -> &Self::Data;
}

/// Manager for multiple fixtures
pub struct FixtureManager {
    fixtures: HashMap<String, Box<dyn std::any::Any>>,
}

impl FixtureManager {
    /// Create a new fixture manager
    pub fn new() -> Self {
        Self {
            fixtures: HashMap::new(),
        }
    }

    /// Register a fixture
    pub fn register<F: Fixture + 'static>(&mut self, name: &str, fixture: F) {
        self.fixtures.insert(name.to_string(), Box::new(fixture));
    }

    /// Get a fixture by name
    pub fn get<F: Fixture + 'static>(&self, name: &str) -> Result<&F> {
        self.fixtures
            .get(name)
            .and_then(|f| f.downcast_ref::<F>())
            .ok_or_else(|| FixtureError::NotFound(name.to_string()))
    }
}

impl Default for FixtureManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestFixture {
        data: String,
    }

    impl Fixture for TestFixture {
        type Data = String;

        fn load() -> Result<Self> {
            Ok(Self {
                data: "test".to_string(),
            })
        }

        fn data(&self) -> &Self::Data {
            &self.data
        }
    }

    #[test]
    fn test_fixture_manager() {
        let mut manager = FixtureManager::new();
        let fixture = TestFixture::load().unwrap();

        manager.register("test", fixture);

        let retrieved: &TestFixture = manager.get("test").unwrap();
        assert_eq!(retrieved.data(), "test");
    }
}
