//! Chaos testing utilities for resilience testing
//!
//! This module provides tools for testing system behavior under various
//! failure conditions, including network delays, failures, and resource exhaustion.

use rand::Rng;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

// ============================================================================
// ChaosRunner - Main chaos testing orchestrator
// ============================================================================

/// Orchestrator for chaos testing scenarios
///
/// Use this to run tests under various failure conditions and verify
/// that your system behaves correctly under stress.
#[derive(Debug, Clone)]
pub struct ChaosRunner {
    enabled: Arc<AtomicBool>,
    scenarios: Arc<RwLock<Vec<ChaosScenario>>>,
    iterations: Arc<AtomicUsize>,
}

use std::sync::RwLock;

impl ChaosRunner {
    /// Create a new chaos runner
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
            scenarios: Arc::new(RwLock::new(Vec::new())),
            iterations: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Enable chaos injection
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::SeqCst);
    }

    /// Disable chaos injection
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::SeqCst);
    }

    /// Check if chaos is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    /// Add a chaos scenario
    pub fn add_scenario(&self, scenario: ChaosScenario) {
        let mut scenarios = self.scenarios.write().unwrap();
        scenarios.push(scenario);
    }

    /// Clear all scenarios
    pub fn clear_scenarios(&self) {
        let mut scenarios = self.scenarios.write().unwrap();
        scenarios.clear();
    }

    /// Get all scenarios
    pub fn scenarios(&self) -> Vec<ChaosScenario> {
        let scenarios = self.scenarios.read().unwrap();
        scenarios.clone()
    }

    /// Increment iteration counter
    pub fn record_iteration(&self) {
        self.iterations.fetch_add(1, Ordering::SeqCst);
    }

    /// Get total iterations
    pub fn iterations(&self) -> usize {
        self.iterations.load(Ordering::SeqCst)
    }

    /// Run a scenario
    pub fn run<F, R>(&self, f: F) -> ChaosResult<R>
    where
        F: Fn() -> R,
    {
        self.record_iteration();

        let scenarios = self.scenarios.read().unwrap();
        let mut rng = rand::thread_rng();

        for scenario in scenarios.iter() {
            if scenario.should_trigger(&mut rng) {
                return ChaosResult::Failure(scenario.name.clone(), scenario.description.clone());
            }
        }

        ChaosResult::Success(f())
    }

    /// Run an async scenario
    #[cfg(feature = "tokio-rt")]
    pub async fn run_async<F, Fut, R>(&self, f: F) -> ChaosResult<R>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        self.record_iteration();

        let scenarios = self.scenarios.read().unwrap();
        let mut rng = rand::thread_rng();

        for scenario in scenarios.iter() {
            if scenario.should_trigger(&mut rng) {
                return ChaosResult::Failure(scenario.name.clone(), scenario.description.clone());
            }
        }

        ChaosResult::Success(f().await)
    }
}

impl Default for ChaosRunner {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ChaosScenario - Individual chaos scenario
// ============================================================================

/// Represents a single chaos injection scenario
#[derive(Debug, Clone)]
pub struct ChaosScenario {
    /// Unique name for this scenario
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Probability of triggering (0.0 to 1.0)
    pub probability: f64,
    /// Type of chaos to inject
    pub chaos_type: ChaosType,
}

impl ChaosScenario {
    /// Create a new scenario
    pub fn new(name: impl Into<String>, description: impl Into<String>, probability: f64, chaos_type: ChaosType) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            probability: probability.clamp(0.0, 1.0),
            chaos_type,
        }
    }

    /// Check if this scenario should trigger
    pub fn should_trigger(&self, rng: &mut impl Rng) -> bool {
        rng.gen::<f64>() < self.probability
    }
}

/// Types of chaos that can be injected
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChaosType {
    /// Simulate network delay
    NetworkDelay,
    /// Simulate network failure
    NetworkFailure,
    /// Simulate resource exhaustion
    ResourceExhaustion,
    /// Simulate service unavailability
    ServiceUnavailable,
    /// Simulate data corruption
    DataCorruption,
    /// Simulate timeout
    Timeout,
    /// Custom chaos type
    Custom(String),
}

impl ChaosType {
    /// Get a human-readable name
    pub fn name(&self) -> &str {
        match self {
            ChaosType::NetworkDelay => "network_delay",
            ChaosType::NetworkFailure => "network_failure",
            ChaosType::ResourceExhaustion => "resource_exhaustion",
            ChaosType::ServiceUnavailable => "service_unavailable",
            ChaosType::DataCorruption => "data_corruption",
            ChaosType::Timeout => "timeout",
            ChaosType::Custom(name) => name,
        }
    }
}

// ============================================================================
// ChaosResult - Result of chaos-enabled operation
// ============================================================================

/// Result of a chaos-enabled operation
#[derive(Debug, Clone)]
pub enum ChaosResult<T> {
    /// Operation succeeded
    Success(T),
    /// Chaos was injected
    Failure(String, String),
}

impl<T> ChaosResult<T> {
    /// Check if chaos was injected
    pub fn is_chaos(&self) -> bool {
        matches!(self, ChaosResult::Failure(_, _))
    }

    /// Get the success value if present
    pub fn success(self) -> Option<T> {
        match self {
            ChaosResult::Success(v) => Some(v),
            ChaosResult::Failure(_, _) => None,
        }
    }

    /// Get the chaos name if chaos was injected
    pub fn chaos_name(&self) -> Option<&str> {
        match self {
            ChaosResult::Success(_) => None,
            ChaosResult::Failure(name, _) => Some(name),
        }
    }
}

// ============================================================================
// NetworkChaos - Network-specific chaos utilities
// ============================================================================

/// Network chaos utilities
pub struct NetworkChaos {
    delay_ms: Option<u64>,
    failure_rate: f64,
}

impl NetworkChaos {
    /// Create a new network chaos injector
    pub fn new() -> Self {
        Self {
            delay_ms: None,
            failure_rate: 0.0,
        }
    }

    /// Add simulated network delay
    pub fn with_delay(mut self, ms: u64) -> Self {
        self.delay_ms = Some(ms);
        self
    }

    /// Add simulated failure rate
    pub fn with_failure_rate(mut self, rate: f64) -> Self {
        self.failure_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Apply chaos and return delay if any
    pub fn apply(&self) -> Option<u64> {
        let mut rng = rand::thread_rng();

        // Check if we should fail
        if rng.gen::<f64>() < self.failure_rate {
            return None; // Simulates failure
        }

        self.delay_ms
    }
}

impl Default for NetworkChaos {
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

    #[test]
    fn test_chaos_runner_enable_disable() {
        let runner = ChaosRunner::new();
        assert!(!runner.is_enabled());
        runner.enable();
        assert!(runner.is_enabled());
        runner.disable();
        assert!(!runner.is_enabled());
    }

    #[test]
    fn test_chaos_runner_scenarios() {
        let runner = ChaosRunner::new();
        let scenario = ChaosScenario::new(
            "test",
            "Test scenario",
            1.0,
            ChaosType::NetworkFailure,
        );
        runner.add_scenario(scenario);

        let scenarios = runner.scenarios();
        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0].name, "test");
    }

    #[test]
    fn test_chaos_runner_iterations() {
        let runner = ChaosRunner::new();
        assert_eq!(runner.iterations(), 0);
        runner.record_iteration();
        runner.record_iteration();
        assert_eq!(runner.iterations(), 2);
    }

    #[test]
    fn test_chaos_runner_run_no_chaos() {
        let runner = ChaosRunner::new();
        runner.disable();

        let result = runner.run(|| 42);
        assert!(matches!(result, ChaosResult::Success(42)));
    }

    #[test]
    fn test_chaos_runner_run_with_chaos() {
        let runner = ChaosRunner::new();
        runner.enable();

        let scenario = ChaosScenario::new(
            "always_trigger",
            "Always triggers",
            1.0,
            ChaosType::NetworkFailure,
        );
        runner.add_scenario(scenario);

        let result = runner.run(|| 42);
        assert!(result.is_chaos());
        assert_eq!(result.chaos_name(), Some("always_trigger"));
    }

    #[test]
    fn test_chaos_scenario_should_trigger() {
        let scenario = ChaosScenario::new(
            "test",
            "Test",
            0.5,
            ChaosType::Timeout,
        );

        // Probability-based, so we just check it doesn't panic
        let mut rng = rand::thread_rng();
        let _ = scenario.should_trigger(&mut rng);
    }

    #[test]
    fn test_chaos_result() {
        let success: ChaosResult<i32> = ChaosResult::Success(42);
        let failure: ChaosResult<i32> = ChaosResult::Failure("test".to_string(), "desc".to_string());

        assert!(!success.is_chaos());
        assert!(failure.is_chaos());

        assert_eq!(success.success(), Some(42));
        assert_eq!(failure.success(), None);
    }

    #[test]
    fn test_network_chaos() {
        let chaos = NetworkChaos::new()
            .with_delay(100)
            .with_failure_rate(0.0);

        let delay = chaos.apply();
        assert_eq!(delay, Some(100));
    }

    #[test]
    fn test_network_chaos_failure() {
        let chaos = NetworkChaos::new()
            .with_failure_rate(1.0); // Always fail

        let delay = chaos.apply();
        assert_eq!(delay, None);
    }
}
