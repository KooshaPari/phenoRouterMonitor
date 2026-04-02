//! End-to-end tests for the phenotype-testing library
//!
//! These tests demonstrate the integration of all testing utilities
//! and their use in real-world scenarios.

use phenotype_testing::prelude::*;

// ============================================================================
// E2E: Complete Test Workflow
// ============================================================================

/// Demonstrates a complete test workflow using all testing utilities
#[test]
fn test_complete_workflow_with_all_utilities() {
    // 1. Create a test environment
    let mut env = TestEnv::new();
    assert_eq!(env.name(), "test");
    assert!(!env.id().is_empty());

    // 2. Add test data
    env.insert("user_id", "user123");
    env.insert("token", "abc123");
    env.set_var("TEST_MODE", "integration");

    // 3. Create an infrastructure fixture for file operations
    let mut fixture = InfrastructureFixture::new();
    fixture.set("config", "value");

    // 4. Create mock storage for external services
    let storage = MockStorage::new();
    storage.insert("service:endpoint", b"response_data");

    // 5. Verify all components work together
    assert_eq!(env.get("user_id"), Some("user123"));
    assert_eq!(fixture.get("config"), Some("value"));
    assert_eq!(storage.get("service:endpoint"), Some(b"response_data".to_vec()));

    // 6. Use call tracker to verify interactions
    let tracker = MockCallTracker::new();
    tracker.record("api_call", vec!["GET".to_string(), "/users".to_string()]);
    tracker.record("api_call", vec!["POST".to_string(), "/events".to_string()]);

    assert!(tracker.was_called("api_call"));
    assert_eq!(tracker.call_count("api_call"), 2);

    // 7. Test cleanup happens correctly
    fixture.teardown();
    env.teardown();
}

// ============================================================================
// E2E: Chaos Testing Scenario
// ============================================================================

/// Demonstrates chaos testing with network resilience
#[test]
fn test_chaos_resilience() {
    let runner = ChaosRunner::new();

    // Configure chaos scenarios
    runner.add_scenario(ChaosScenario::new(
        "network_delay",
        "Simulates network latency",
        0.5,
        ChaosType::NetworkDelay,
    ));

    runner.add_scenario(ChaosScenario::new(
        "network_failure",
        "Simulates network failure",
        0.2,
        ChaosType::NetworkFailure,
    ));

    // Test 100 operations
    let mut success_count = 0;
    let mut chaos_count = 0;

    for _ in 0..100 {
        let result = runner.run(|| {
            // Simulate some work
            let _ = (0..10).sum::<i32>();
            true
        });

        match result {
            ChaosResult::Success(_) => success_count += 1,
            ChaosResult::Failure(name, _) => {
                chaos_count += 1;
                assert!(name == "network_delay" || name == "network_failure");
            }
        }
    }

    // Verify chaos injection is working
    assert!(chaos_count > 0, "Chaos should have been injected");
    assert_eq!(success_count + chaos_count, 100);
}

// ============================================================================
// E2E: Network Chaos Testing
// ============================================================================

/// Tests network chaos with various configurations
#[test]
fn test_network_chaos_scenarios() {
    // Test with delay
    let chaos = NetworkChaos::new().with_delay(50);
    let delay = chaos.apply();
    assert_eq!(delay, Some(50));

    // Test with failure rate
    let chaos = NetworkChaos::new().with_failure_rate(1.0);
    let delay = chaos.apply();
    assert_eq!(delay, None);

    // Test combined
    let chaos = NetworkChaos::new().with_delay(100).with_failure_rate(0.0);
    let delay = chaos.apply();
    assert_eq!(delay, Some(100));
}

// ============================================================================
// E2E: Benchmark Integration
// ============================================================================

/// Demonstrates benchmark usage with real workloads
#[test]
fn test_benchmark_integration() {
    let config = BenchmarkConfig::new()
        .with_iterations(50)
        .with_warmup(5)
        .with_verbose(false);

    let mut runner = BenchmarkRunner::new(config);

    // Benchmark a simple operation
    let result = runner.run(|| {
        // Simulate some computation
        let mut sum = 0i64;
        for i in 0..1000 {
            sum += i;
        }
        sum
    });

    // Verify results
    assert_eq!(result.samples, 50);
    assert!(result.iterations_per_second > 0.0);
    assert!(result.mean > Duration::from_nanos(0));

    // Verify format output
    let formatted = result.format();
    assert!(formatted.contains("Mean"));
    assert!(formatted.contains("Ops/sec"));
}

// ============================================================================
// E2E: Fixture Patterns
// ============================================================================

/// Demonstrates various fixture patterns
#[test]
fn test_fixture_patterns() {
    // Pattern 1: Using TestEnv directly
    let env = TestEnv::new();
    assert_eq!(env.name(), "test");

    // Pattern 2: Named environment
    let env = TestEnv::with_name("api_test");
    assert_eq!(env.name(), "api_test");

    // Pattern 3: Infrastructure fixture with resources
    let mut fixture = InfrastructureFixture::new();
    fixture.insert_resource("db", "postgres://localhost/test");
    fixture.set_config("timeout", "30s");

    assert_eq!(fixture.get_resource("db"), Some("postgres://localhost/test"));
    assert_eq!(fixture.get_config("timeout"), Some("30s"));

    // Pattern 4: Temporary directory
    let fixture = InfrastructureFixture::new();
    assert!(fixture.temp_dir().exists());

    // Pattern 5: Create temp files
    let path = fixture.create_temp_file("test.txt", b"hello world").unwrap();
    assert!(path.exists());
    assert_eq!(std::fs::read(&path).unwrap(), b"hello world");
}

// ============================================================================
// E2E: Mock Call Tracking
// ============================================================================

/// Demonstrates mock call tracking patterns
#[test]
fn test_mock_call_tracking_patterns() {
    let tracker = MockCallTracker::new();

    // Record various calls
    tracker.record("connect", vec!["host1".to_string()]);
    tracker.record("query", vec!["SELECT * FROM users".to_string()]);
    tracker.record("query", vec!["SELECT * FROM orders".to_string()]);
    tracker.record("disconnect", vec![]);

    // Verify call counts
    assert_eq!(tracker.call_count("connect"), 1);
    assert_eq!(tracker.call_count("query"), 2);
    assert_eq!(tracker.call_count("disconnect"), 1);

    // Verify specific calls
    let query_calls = tracker.calls_for("query");
    assert_eq!(query_calls.len(), 2);
    assert!(query_calls[0].args[0].contains("users"));
    assert!(query_calls[1].args[0].contains("orders"));

    // Verify clear
    tracker.clear();
    assert!(!tracker.was_called("connect"));
    assert_eq!(tracker.call_count("query"), 0);
}

// ============================================================================
// E2E: Test Result Types
// ============================================================================

/// Demonstrates TestResult usage
#[test]
fn test_result_types() {
    let success = TestResult::Success;
    let failure = TestResult::Failure("Something went wrong".to_string());
    let skipped = TestResult::Skipped("Not applicable".to_string());

    assert!(success.is_success());
    assert!(!success.is_failure());
    assert!(!success.is_skipped());

    assert!(failure.is_failure());
    assert!(!failure.is_success());
    assert!(!failure.is_skipped());

    assert!(skipped.is_skipped());
    assert!(!skipped.is_success());
    assert!(!skipped.is_failure());
}

// ============================================================================
// E2E: Duration Formatting
// ============================================================================

/// Tests duration formatting utilities
#[test]
fn test_duration_formatting() {
    use phenotype_testing::benchmark::format_duration;

    // Test various durations
    assert!(format_duration(Duration::from_nanos(500)).contains("ns"));
    assert!(format_duration(Duration::from_micros(500)).contains("µs"));
    assert!(format_duration(Duration::from_millis(500)).contains("ms"));
    assert!(format_duration(Duration::from_secs(1)).contains("s"));
}

// ============================================================================
// E2E: Timer Usage
// ============================================================================

/// Demonstrates timer usage
#[test]
fn test_timer_usage() {
    let mut timer = BenchmarkTimer::new();

    // Small delay
    std::thread::sleep(Duration::from_millis(5));
    let elapsed = timer.elapsed();

    // Should have some elapsed time
    assert!(elapsed >= Duration::from_millis(5));

    // Reset
    timer.reset();
    let new_elapsed = timer.elapsed();
    assert!(new_elapsed < Duration::from_millis(1));
}
