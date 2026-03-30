# phenotype-router-api Quick Start Guide

A production-ready REST API server for managing a router with agents, health monitoring, and metrics export.

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
phenotype-router-api = { version = "0.2", path = "../crates/phenotype-router-api" }
tokio = { version = "1.41", features = ["full"] }
```

## Basic Usage

### 1. Create and Start Server

```rust
use phenotype_router_api::{RouterApiServer, RouterConfig};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Create config
    let config = RouterConfig::default()
        .with_id("my-router")
        .with_environment("production");

    // Create and run server
    let server = RouterApiServer::new(config);
    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();

    if let Err(e) = server.run(addr).await {
        eprintln!("Server error: {}", e);
    }
}
```

### 2. Register Agents

```rust
use phenotype_router_api::Agent;

let agent = Agent::new("agent-001", "Worker 1")
    .mark_active()
    .with_capability("process")
    .with_capability("batch")
    .with_metadata("region", "us-east-1")
    .with_metadata("version", "1.0");

server.state().add_agent(agent)?;
```

## HTTP Endpoints Reference

### Health Check

```bash
# Liveness probe (always returns 200)
curl http://localhost:3000/health

# Response:
{
  "status": "healthy",
  "components": [...],
  "timestamp": "2024-03-30T12:00:00Z",
  "version": null
}
```

### Readiness Check

```bash
# Readiness probe (200 if agents active, else degraded)
curl http://localhost:3000/ready

# Response: Same as /health, but status depends on active agents
```

### Metrics

```bash
# Prometheus-format metrics
curl http://localhost:3000/metrics

# Response:
# HELP router_total_requests Total number of requests
# TYPE router_total_requests counter
router_total_requests 42

# JSON-format metrics
curl http://localhost:3000/metrics/json

# Response:
{
  "total_requests": 42,
  "total_errors": 2,
  "latency_buckets": {},
  "requests_by_status": { "200": 40, "404": 2 },
  "requests_by_path": { "/health": 20, "/agents": 22 },
  "timestamp": "2024-03-30T12:00:00Z"
}
```

### Router Info

```bash
# Get router configuration and status
curl http://localhost:3000/router/info

# Response:
{
  "id": "my-router",
  "version": "0.2.0",
  "environment": "production",
  "timestamp": "2024-03-30T12:00:00Z",
  "active_agents": 5,
  "total_routes": 8,
  "uptime_secs": 3600,
  "metadata": {}
}
```

### Routes

```bash
# List all routes
curl http://localhost:3000/router/routes

# Response:
{
  "routes": [
    {
      "path": "/health",
      "methods": ["GET"],
      "description": "Liveness probe"
    },
    ...
  ]
}
```

### Agent Management

```bash
# List all agents
curl http://localhost:3000/agents

# Create agent
curl -X POST http://localhost:3000/agents \
  -H "Content-Type: application/json" \
  -d '{
    "id": "agent-2",
    "name": "Worker 2",
    "status": "active",
    "capabilities": ["query"],
    "metadata": {}
  }'

# Get agent
curl http://localhost:3000/agents/agent-2

# Update agent
curl -X PUT http://localhost:3000/agents/agent-2 \
  -H "Content-Type: application/json" \
  -d '{
    "id": "agent-2",
    "name": "Worker 2 Updated",
    "status": "inactive",
    "capabilities": ["query", "write"],
    "metadata": {}
  }'

# Delete agent
curl -X DELETE http://localhost:3000/agents/agent-2

# Refresh all agents (update heartbeats)
curl -X POST http://localhost:3000/agents/refresh
```

## Configuration

### RouterConfig Builder Pattern

```rust
use phenotype_router_api::RouterConfig;

let config = RouterConfig::default()
    .with_id("custom-router")
    .with_environment("staging")
    .with_max_agents(100);

// Access properties
assert_eq!(config.id, "custom-router");
assert_eq!(config.max_agents, 100);
```

### Available Config Options

```rust
pub struct RouterConfig {
    pub id: String,                          // Router UUID
    pub version: String,                     // From Cargo.toml
    pub environment: String,                 // dev, staging, prod
    pub max_agents: usize,                   // Default: 100
    pub health_check_interval_secs: u64,     // Default: 30
    pub routes: Vec<Route>,                  // Available routes
    pub metadata: HashMap<String, String>,   // Custom metadata
}
```

## State Management

### RouterState API

```rust
let state = server.state();

// Agent operations
state.add_agent(agent)?;
state.get_agent("id")?;
state.update_agent("id", agent)?;
state.remove_agent("id")?;
state.agents();  // Get all agents
state.refresh_agents();  // Update heartbeats

// Metrics
state.record_request("/path", "GET", 200);
state.record_error("timeout");
let metrics = state.metrics();

// Status
state.active_agents_count();
state.uptime_secs();
state.status();  // String like "active_agents=5/10"
```

## Error Handling

```rust
use phenotype_router_api::RouterApiError;

match server.state().add_agent(agent) {
    Ok(_) => println!("Agent added"),
    Err(RouterApiError::ValidationError(msg)) => {
        eprintln!("Invalid agent: {}", msg);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Testing

### Unit Tests

```bash
cargo test -p phenotype-router-api
```

### Integration Tests with curl

```bash
# Start server in background
cargo run --bin my-app &
sleep 1

# Test endpoints
curl -s http://localhost:3000/health | jq '.'
curl -s http://localhost:3000/ready | jq '.'
curl -s http://localhost:3000/metrics/json | jq '.'
```

## Advanced Usage

### Custom Metrics Collector

```rust
use phenotype_router_api::MetricsCollector;
use std::sync::Arc;

struct CustomMetricsCollector { ... }

impl MetricsCollector for CustomMetricsCollector {
    fn record_request(&self, path: &str, method: &str, status: u16) { ... }
    fn record_error(&self, error_type: &str) { ... }
    fn get_metrics(&self) -> Metrics { ... }
}

let state = RouterState::with_metrics(
    config,
    Arc::new(CustomMetricsCollector::new())
);
```

### Agent Lifecycle Management

```rust
// Mark agent as active and track heartbeat
let agent = Agent::new("id", "name").mark_active();
state.add_agent(agent)?;

// Refresh heartbeats periodically
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        state.refresh_agents();
    }
});

// Monitor agent count
let count = state.active_agents_count();
```

## Deployment Checklist

- [ ] Configure router ID and environment
- [ ] Set appropriate max_agents limit
- [ ] Start server on desired port
- [ ] Register health check endpoint with orchestrator
- [ ] Monitor /metrics/json endpoint
- [ ] Implement agent registration webhook
- [ ] Set up alerts on error rates
- [ ] Configure log aggregation

## Performance Tips

1. **Metrics Recording**: Uses lock-free atomic operations (no allocations)
2. **Agent Registry**: RwLock allows concurrent reads
3. **Async Handlers**: Tokio runtime scales to thousands of requests
4. **JSON Responses**: Pre-serialized for speed
5. **Memory**: ~1KB per agent, minimal overhead

## Troubleshooting

### Port Already in Use

```rust
// Try different port
let addr: SocketAddr = "127.0.0.1:3001".parse()?;
```

### Agent Not Found

```rust
// Ensure agent ID matches exactly
if let Some(agent) = state.get_agent("exact-id") {
    println!("Found: {:?}", agent);
}
```

### Metrics Growing Unbounded

```rust
// Metrics are snapshot-based, not cumulative
// Total counts grow, but latency/status maps are reset per snapshot
let metrics = state.metrics();
println!("Success rate: {:.1}%", metrics.success_rate());
```

## See Also

- `PHENOTYPE_ROUTER_API_SUMMARY.md` - Full technical documentation
- `phenotype-router-api/src/lib.rs` - Source code with examples
- `cargo test -p phenotype-router-api` - All 46 unit tests

## Support

For issues or questions:
1. Check test cases for usage examples
2. Review error type definitions
3. Run with RUST_LOG=debug for detailed logging

---

**Version**: 0.2.0
**Last Updated**: 2024-03-30
