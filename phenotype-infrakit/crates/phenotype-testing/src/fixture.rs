//! Test fixture utilities for standardized test setup
//!
//! This module provides common patterns for test setup and teardown,
//! including the `TestFixture` trait and concrete implementations.

use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// TestFixture trait
// ============================================================================

/// Test fixture trait for standardized test setup
///
/// Implement this trait to create reusable test fixtures that handle
/// setup and teardown consistently across your test suite.
///
/// # Example
///
/// ```rust
/// use phenotype_testing::TestFixture;
///
/// struct MyTestFixture {
///     data: String,
/// }
///
/// impl TestFixture for MyTestFixture {
///     fn setup() -> Self {
///         Self {
///             data: "test".to_string(),
///         }
///     }
///
///     fn name(&self) -> &str {
///         "my_test_fixture"
///     }
///
///     fn id(&self) -> &str {
///         &self.data
///     }
///
///     fn teardown(&self) {
///         // Cleanup logic here
///     }
/// }
/// ```
pub trait TestFixture: Send + Sync + 'static {
    /// Create and setup the test fixture
    fn setup() -> Self
    where
        Self: Sized;

    /// Get the fixture name
    fn name(&self) -> &str;

    /// Get the fixture ID
    fn id(&self) -> &str;

    /// Teardown and cleanup the test fixture
    fn teardown(&self);
}

// ============================================================================
// TestResult enum
// ============================================================================

/// Test result type for standardized test outcomes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestResult {
    /// Test passed successfully
    Success,
    /// Test failed with a message
    Failure(String),
    /// Test was skipped with a reason
    Skipped(String),
}

impl TestResult {
    /// Check if test succeeded
    pub fn is_success(&self) -> bool {
        matches!(self, TestResult::Success)
    }

    /// Check if test failed
    pub fn is_failure(&self) -> bool {
        matches!(self, TestResult::Failure(_))
    }

    /// Check if test was skipped
    pub fn is_skipped(&self) -> bool {
        matches!(self, TestResult::Skipped(_))
    }
}

// ============================================================================
// TestEnv - Standard test environment
// ============================================================================

/// Standard test environment with resource storage
///
/// This is a concrete implementation of `TestFixture` that provides
/// a simple key-value store for test resources.
#[derive(Debug, Clone)]
pub struct TestEnv {
    name: String,
    id: String,
    resources: HashMap<String, String>,
    config: HashMap<String, String>,
}

impl TestEnv {
    /// Create a new test fixture
    pub fn new() -> Self {
        Self {
            name: "test".to_string(),
            id: Uuid::new_v4().to_string(),
            resources: HashMap::new(),
            config: HashMap::new(),
        }
    }

    /// Create a named test fixture
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            id: Uuid::new_v4().to_string(),
            resources: HashMap::new(),
            config: HashMap::new(),
        }
    }

    /// Set a configuration value
    pub fn set_var(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.config.insert(key.into(), value.into());
    }

    /// Get a configuration value
    pub fn get_var(&self, key: &str) -> Option<&str> {
        self.config.get(key).map(|s| s.as_str())
    }

    /// Insert a resource
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.resources.insert(key.into(), value.into());
    }

    /// Get a resource
    pub fn get(&self, key: &str) -> Option<&str> {
        self.resources.get(key).map(|s| s.as_str())
    }

    /// Get the fixture ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the fixture name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Default for TestEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl TestFixture for TestEnv {
    fn setup() -> Self {
        Self::new()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn teardown(&self) {
        // Cleanup resources if needed
    }
}

// ============================================================================
// InfrastructureFixture - Infrastructure test fixture
// ============================================================================

/// Infrastructure test fixture with shared test utilities
///
/// This fixture provides additional infrastructure-related utilities
/// such as temporary directories and service mocking.
#[derive(Debug, Clone)]
pub struct InfrastructureFixture {
    name: String,
    id: String,
    resources: HashMap<String, String>,
    config: HashMap<String, String>,
    temp_dir: std::path::PathBuf,
}

impl InfrastructureFixture {
    /// Create a new infrastructure fixture
    pub fn new() -> Self {
        let id = Uuid::new_v4().to_string();
        let temp_dir = std::env::temp_dir().join(format!("phenotype-test-{}", id));

        // Ensure temp directory exists
        let _ = std::fs::create_dir_all(&temp_dir);

        Self {
            name: "infrastructure".to_string(),
            id,
            resources: HashMap::new(),
            config: HashMap::new(),
            temp_dir,
        }
    }

    /// Create with a specific name
    pub fn with_name(name: impl Into<String>) -> Self {
        let id = Uuid::new_v4().to_string();
        let temp_dir = std::env::temp_dir().join(format!("phenotype-test-{}", id));

        let _ = std::fs::create_dir_all(&temp_dir);

        Self {
            name: name.into(),
            id,
            resources: HashMap::new(),
            config: HashMap::new(),
            temp_dir,
        }
    }

    /// Get the fixture name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the fixture ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get a resource by key
    pub fn get_resource(&self, key: &str) -> Option<&str> {
        self.resources.get(key).map(|s| s.as_str())
    }

    /// Insert a resource
    pub fn insert_resource(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.resources.insert(key.into(), value.into());
    }

    /// Get a config value by key
    pub fn get_config(&self, key: &str) -> Option<&str> {
        self.config.get(key).map(|s| s.as_str())
    }

    /// Set a config value
    pub fn set_config(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.config.insert(key.into(), value.into());
    }

    /// Get a value from resources (convenience method)
    pub fn get(&self, key: &str) -> Option<&str> {
        self.resources.get(key).map(|s| s.as_str())
    }

    /// Set a value in resources
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.resources.insert(key.into(), value.into());
    }

    /// Get the temporary directory path
    pub fn temp_dir(&self) -> &std::path::PathBuf {
        &self.temp_dir
    }

    /// Create a temporary file in the fixture's temp directory
    pub fn create_temp_file(&self, name: &str, content: &[u8]) -> std::io::Result<std::path::PathBuf> {
        let path = self.temp_dir.join(name);
        std::fs::write(&path, content)?;
        Ok(path)
    }
}

impl Default for InfrastructureFixture {
    fn default() -> Self {
        Self::new()
    }
}

impl TestFixture for InfrastructureFixture {
    fn setup() -> Self {
        Self::new()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn teardown(&self) {
        // Cleanup temp directory
        let _ = std::fs::remove_dir_all(&self.temp_dir);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_success() {
        assert!(TestResult::Success.is_success());
        assert!(!TestResult::Success.is_failure());
    }

    #[test]
    fn test_result_failure() {
        let result = TestResult::Failure("error".to_string());
        assert!(result.is_failure());
        assert!(!result.is_success());
    }

    #[test]
    fn test_result_skipped() {
        let result = TestResult::Skipped("reason".to_string());
        assert!(result.is_skipped());
        assert!(!result.is_success());
    }

    #[test]
    fn test_env_new() {
        let env = TestEnv::new();
        assert_eq!(env.name(), "test");
        assert!(!env.id().is_empty());
    }

    #[test]
    fn test_env_with_name() {
        let env = TestEnv::with_name("custom");
        assert_eq!(env.name(), "custom");
    }

    #[test]
    fn test_env_set_get_var() {
        let mut env = TestEnv::new();
        env.set_var("KEY", "value");
        assert_eq!(env.get_var("KEY"), Some("value"));
    }

    #[test]
    fn test_env_insert_get() {
        let mut env = TestEnv::new();
        env.insert("resource", "data");
        assert_eq!(env.get("resource"), Some("data"));
    }

    #[test]
    fn test_env_fixture_trait() {
        let env = TestEnv::setup();
        assert_eq!(env.name(), "test");
        assert!(!env.id().is_empty());
    }

    #[test]
    fn test_infrastructure_fixture_new() {
        let fixture = InfrastructureFixture::new();
        assert_eq!(fixture.name(), "infrastructure");
        assert!(!fixture.id().is_empty());
    }

    #[test]
    fn test_infrastructure_fixture_with_name() {
        let fixture = InfrastructureFixture::with_name("custom");
        assert_eq!(fixture.name(), "custom");
    }

    #[test]
    fn test_infrastructure_fixture_resources() {
        let mut fixture = InfrastructureFixture::new();
        fixture.insert_resource("key", "value");
        assert_eq!(fixture.get_resource("key"), Some("value"));
        assert_eq!(fixture.get("key"), Some("value"));
    }

    #[test]
    fn test_infrastructure_fixture_config() {
        let mut fixture = InfrastructureFixture::new();
        fixture.set_config("config_key", "config_value");
        assert_eq!(fixture.get_config("config_key"), Some("config_value"));
    }

    #[test]
    fn test_infrastructure_fixture_temp_dir() {
        let fixture = InfrastructureFixture::new();
        assert!(fixture.temp_dir().exists());
    }
}
