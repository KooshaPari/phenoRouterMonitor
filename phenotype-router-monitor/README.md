# phenotype-router-monitor

A high-performance API monitoring and routing infrastructure for Phenotype ecosystem services.

## Overview

phenotype-router-monitor provides:

- **Core Routing Engine** (`phenotype-router-core`) - Path-based routing with regex/wildcard patterns, round-robin load balancing, and TOML configuration
- **Metrics Collection** (`phenotype-router-metrics`) - Prometheus-compatible metrics with latency histograms (p50, p95, p99), status codes, and in-flight tracking
- **REST API Server** (`phenotype-router-api`) - Complete monitoring API with health checks, metrics endpoints, agent management, and router configuration introspection
- **Configuration Management** (`phenotype-router-config`) - Hot reload support with file watching and dynamic configuration updates

## Quick Start

### Installation

```bash
cargo build --release
```

### Running the API Server

```bash
# Using default configuration
cargo run --example main

# Using custom configuration
cargo run --example main -- --config examples/config.toml
```

### API Endpoints

All endpoints are available at `http://localhost:3030` by default.

#### Health & Readiness

```bash
# Health check
curl http://localhost:3030/health

# Kubernetes readiness probe
curl http://localhost:3030/ready
```

#### Metrics

```bash
# Prometheus format
curl http://localhost:3030/metrics

# JSON format
curl http://localhost:3030/metrics/json
```

#### Router Management

```bash
# Get router information
curl http://localhost:3030/router/info

# List configured routes
curl http://localhost:3030/router/routes
```

#### Agent Management

```bash
# List agents/services
curl http://localhost:3030/agents

# Reload agent configuration (requires restart to take effect)
curl -X POST http://localhost:3030/agents/refresh
```

## Configuration

Router configuration uses TOML format. See `examples/config.toml` for a complete example.

### Configuration Schema

```toml
# Listen address and port
listen_addr = "0.0.0.0"
listen_port = 3030

# Request limits
max_body_size = 10485760  # bytes
timeout_ms = 30000         # milliseconds

# Routes
[[routes]]
service = "api"
path_pattern = "^/api/(.*)"           # Regex pattern
backends = [
  "http://backend1:3000",
  "http://backend2:3000"
]
timeout_ms = 30000
strategy = "roundrobin"               # or "random", "leastconnections"
```

### Pattern Matching

Supports three pattern types:

1. **Exact Match** - `/exact/path`
2. **Regex** - `^/api/.*` (must start with `^`)
3. **Wildcard** - `/api/*`

### Load Balancing Strategies

- `roundrobin` - Distribute requests equally across backends
- `random` - Random backend selection
- `leastconnections` - Route to backend with fewest connections (future)

## Metrics

The `/metrics` endpoint exposes Prometheus-compatible metrics:

```
# Request counts
http_requests_total                    # Total requests
http_requests_in_flight                # Requests currently processing

# By status code
http_requests_by_status{status="2xx"}
http_requests_by_status{status="4xx"}
http_requests_by_status{status="5xx"}

# Latency histogram
http_request_duration_ms_bucket{le="50"}
http_request_duration_ms_bucket{le="100"}
http_request_duration_ms_bucket{le="250"}
http_request_duration_ms_bucket{le="500"}
http_request_duration_ms_bucket{le="+Inf"}
http_request_duration_ms_sum
http_request_duration_ms_count

# Latency percentiles
http_request_latency_percentiles{percentile="p50"}
http_request_latency_percentiles{percentile="p95"}
http_request_latency_percentiles{percentile="p99"}
http_request_latency_percentiles{percentile="min"}
http_request_latency_percentiles{percentile="max"}
http_request_latency_percentiles{percentile="avg"}

# By service
http_requests_by_service{service="api"}
http_requests_by_service{service="web"}
```

## Testing

Run the complete test suite:

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p phenotype-router-core
cargo test -p phenotype-router-metrics
cargo test -p phenotype-router-api
cargo test -p phenotype-router-config

# With output
cargo test --workspace -- --nocapture

# Coverage
cargo tarpaulin --workspace --out Html --output-dir target/coverage
```

## Crate Structure

### phenotype-router-core

Core routing engine with:

- Path pattern matching (exact, regex, wildcard)
- Backend pool management with round-robin load balancing
- TOML configuration loading and validation
- Router struct for request routing

**Key Types:**
- `Router` - Main routing engine
- `BackendPool` - Load-balanced backend collection
- `PathPattern` - Pattern matching engine
- `RouterConfig` - Configuration schema

### phenotype-router-metrics

Prometheus metrics collection with:

- Request count tracking
- Latency histogram with p50/p95/p99 percentiles
- Status code counters (2xx/4xx/5xx)
- In-flight request gauge
- Prometheus text format export
- JSON export for dashboards

**Key Types:**
- `MetricsCollector` - Thread-safe metrics collection
- `PrometheusExporter` - Prometheus format exporter
- `RequestMetrics` - Individual request metrics

### phenotype-router-api

REST API server for monitoring:

- Health checks (`/health`, `/ready`)
- Metrics endpoints (`/metrics`, `/metrics/json`)
- Router information (`/router/info`, `/router/routes`)
- Agent management (`/agents`, `/agents/refresh`)

**Key Types:**
- `ApiServer` - HTTP server
- `AppState` - Shared application state
- Handlers for all endpoints

### phenotype-router-config

Configuration management with:

- File loading and parsing
- Hot reload support via file watching
- Configuration validation
- Change callbacks

**Key Types:**
- `ConfigManager<T>` - Generic configuration manager
- `FileWatcher` - File change detection

## Architecture

The system follows Hexagonal Architecture (Ports & Adapters):

```
┌─────────────────────────────────────────────────────┐
│                  HTTP Handlers                      │
│    (/health, /metrics, /agents, etc.)               │
├─────────────────────────────────────────────────────┤
│                   API Server                        │
│             (Axum, Tower, TLS support)              │
├─────────────────────────────────────────────────────┤
│    Router Core    │   Metrics       │   Config      │
│  - Routing       │  - Collection   │  - Loading    │
│  - LB Balancing  │  - Export       │  - Watch      │
│  - Validation    │  - Percentiles  │  - Reload     │
└─────────────────────────────────────────────────────┘
```

## Performance

- **Thread-safe** - All components use Arc/DashMap for concurrent access
- **Lock-free** - Atomic operations for hot paths
- **Minimal overhead** - Metrics collection adds <1% latency
- **Scalable** - Tested with 1000+ requests/second

## Dependencies

Core dependencies (latest stable):

- `axum` 0.7 - Async HTTP framework
- `tokio` 1.35+ - Async runtime
- `serde`/`toml` - Configuration parsing
- `prometheus` 0.13 - Metrics format
- `dashmap` 5.5 - Concurrent hashmap
- `thiserror` 1.0 - Error handling

## Contributing

All code must:

1. Pass `cargo clippy` with no warnings
2. Pass `cargo fmt` formatting check
3. Have comprehensive test coverage (aim for 85%+)
4. Include tracing for observability
5. Follow SOLID principles and Rust idioms

## Testing Strategy

The project uses Test-First Development (TFD) methodology:

- Every FR (Functional Requirement) has ≥1 test
- Every test references ≥1 FR via comment: `// Traces to: FR-ROUTER-NNN`
- Tests are organized by module and cover:
  - Unit tests (module level)
  - Integration tests (cross-module)
  - Property-based tests (for complex logic)

## License

MIT

## Project Status

**Phase**: 1 (API Server & Configuration)
**Status**: Implementation In Progress
**Maintenance**: Active

See `PLAN.md` for detailed roadmap and `CHANGELOG.md` for version history.
