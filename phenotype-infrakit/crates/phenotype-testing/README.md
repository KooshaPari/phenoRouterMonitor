# Phenotype Testing Library

Shared testing infrastructure for the Phenotype ecosystem.

## Overview

The `phenotype-testing` crate provides common testing utilities for all Phenotype ecosystem crates, including:

- **Test Fixtures**: Standardized test setup/teardown patterns
- **Mock Infrastructure**: Common mocking utilities for testing
- **Chaos Testing**: Tools for testing system resilience
- **Benchmarks**: Performance testing infrastructure

## Quick Start

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
phenotype-testing = { path = "../phenotype-infrakit/crates/phenotype-testing" }
```

## Features

### Test Fixtures

```rust
use phenotype_testing::{TestEnv, TestFixture, InfrastructureFixture};

// Create a test environment
let env = TestEnv::new();
env.insert("key", "value");
assert_eq!(env.get("key"), Some("value"));

// Use the trait for reusable fixtures
struct MyFixture;

impl TestFixture for MyFixture {
    fn setup() -> Self {
        Self
    }
    fn name(&self) -> &str { "my_fixture" }
    fn id(&self) -> &str { "123" }
    fn teardown(&self) {}
}
```

### Mock Storage

```rust
use phenotype_testing::MockStorage;

// Create in-memory storage
let storage = MockStorage::new();
storage.insert("key", b"value");

// JSON support
storage.insert_json("data", &serde_json::json!({"name": "test"})).unwrap();
```

### Chaos Testing

```rust
use phenotype_testing::{ChaosRunner, ChaosScenario, ChaosType};

// Configure chaos scenarios
let runner = ChaosRunner::new();
runner.add_scenario(ChaosScenario::new(
    "network_failure",
    "Simulates network failure",
    0.1,  // 10% chance
    ChaosType::NetworkFailure,
));

// Run with chaos injection
let result = runner.run(|| {
    // Your code here
    some_operation()
});
```

### Benchmarking

```rust
use phenotype_testing::{BenchmarkConfig, BenchmarkRunner};

let config = BenchmarkConfig::new()
    .with_iterations(100)
    .with_warmup(10);

let mut runner = BenchmarkRunner::new(config);

let result = runner.run(|| {
    // Benchmark this
    compute_something()
});

println!("{}", result.format());
```

## Module Structure

- `fixture` - Test fixtures and TestEnv
- `mock` - MockStorage and MockCallTracker
- `chaos` - ChaosRunner for resilience testing
- `benchmark` - BenchmarkConfig and BenchmarkRunner

## License

MIT
