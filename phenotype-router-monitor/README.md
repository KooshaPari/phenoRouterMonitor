# phenotype-router-monitor

A comprehensive HTTP routing engine with path matching, load balancing, and monitoring capabilities.

## Features

- **Path Matching Strategies**
  - Exact: Case-sensitive exact path matching
  - Wildcard: Glob-style patterns with `*` and `?` support
  - Regex: Full regular expression matching

- **Load Balancing Strategies**
  - Round-Robin: Sequential distribution across backends
  - Random: Probabilistic backend selection
  - Least-Connections: Select backend with fewest active connections

- **Monitoring & Tracking**
  - Comprehensive routing metrics and statistics
  - Health check framework for backend monitoring
  - Decision history tracking with latency analysis
  - Per-backend and per-route analytics

- **Architecture**
  - Modular crate design with clear separation of concerns
  - Thread-safe operations with Arc and RwLock
  - Zero-copy routing decisions where possible
  - Extensible matcher and balancer traits

## Crates

### phenotype-router-core (400+ LOC)
Core routing primitives:
- `MatcherStrategy` trait with Exact, Wildcard, Regex implementations
- `LoadBalancingStrategy` trait with RoundRobin, Random, LeastConnections
- `RouteRegistry` for priority-based route management
- `Backend` and `Route` data structures
- Metrics tracking (RoutingMetrics, BackendMetrics)

### phenotype-router-monitor (600+ LOC)
High-level routing orchestration:
- `Router` - Main routing engine with load balancing
- `RouterOrchestrator` - Complex routing coordination with policies
- `HealthChecker` trait with HTTP and TCP implementations
- `DecisionTracker` - Historical decision tracking and analytics
- Full re-export of core types for convenient access

## Usage

```rust
use phenotype_router_monitor::{
    Router, RouteConfig, Backend, RouteRegistry, RoundRobin,
};
use std::sync::Arc;

// Create registry
let registry = Arc::new(RouteRegistry::new());

// Define route configuration
let config = RouteConfig {
    id: "api".to_string(),
    name: "API Gateway".to_string(),
    matcher_type: "wildcard".to_string(),
    pattern: "/api/*".to_string(),
    balancer_type: "round-robin".to_string(),
    backends: vec!["backend-1".to_string(), "backend-2".to_string()],
    priority: Some(10),
};

// Create backends
let backends = vec![
    Backend::new("backend-1".to_string(), "http://localhost:3000".to_string()),
    Backend::new("backend-2".to_string(), "http://localhost:3001".to_string()),
];

// Register route
registry.register_route(config, backends).expect("Failed to register route");

// Create router with RoundRobin load balancing
let router = Router::new(registry);

// Route requests
match router.route("/api/users") {
    Ok(backend) => println!("Route to: {}", backend.url()),
    Err(e) => eprintln!("Routing error: {}", e),
}

// Check metrics
let metrics = router.metrics();
println!("Total decisions: {}", metrics.total_decisions);
println!("Success rate: {:.2}%", metrics.success_rate());
```

## Testing

Over 50 integration tests covering:

- Path matching (exact, wildcard, regex)
- Load balancing (round-robin, random, least-connections)
- Route registry operations (register, find, remove)
- Priority-based routing
- Health status tracking
- Decision recording and analytics
- Thread safety and concurrent operations
- Error handling and edge cases

Run tests:
```bash
cargo test --all
cargo test --all -- --nocapture  # with output
```

## Architecture

```
┌────────────────────────────────────────────┐
│      phenotype-router-monitor              │
│  (Router, Orchestrator, HealthChecker)     │
├────────────────────────────────────────────┤
│      phenotype-router-core                 │
│  (Matchers, Balancers, Registry)           │
├────────────────────────────────────────────┤
│  Serde, Regex, Thiserror (dependencies)    │
└────────────────────────────────────────────┘
```

## Design Patterns

- **Strategy Pattern**: MatcherStrategy and LoadBalancingStrategy traits
- **Registry Pattern**: RouteRegistry for centralized route management
- **Trait Objects**: Polymorphic matcher and balancer selection
- **Atomic Operations**: Lock-free counters for metrics
- **Thread Safety**: Arc<RwLock<T>> for concurrent access

## Lines of Code

- **phenotype-router-core**: ~1200 LOC (including 400+ tests)
- **phenotype-router-monitor**: ~800 LOC (including 250+ tests)
- **Total**: ~2000 LOC with 50+ integration tests

All tests passing, zero clippy warnings.

## Future Enhancements

- Async/await support with Tokio
- Circuit breaker pattern for backend failures
- Request rate limiting and throttling
- Cache integration for frequently accessed routes
- Distributed tracing support
- Configuration hot-reload
