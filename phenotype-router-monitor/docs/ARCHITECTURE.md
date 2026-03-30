# Architecture: phenotype-router-monitor

Consolidated architecture for router and API monitoring infrastructure.

## Design Principles

| Principle | Application |
|-----------|-------------|
| **Hexagonal** | Core abstractions (Router, Metrics, Meter) exposed via ports; adapters implement specific transports |
| **SOLID** | Single responsibility: each crate handles one concern (routing vs. metrics vs. metering) |
| **Lock-Free First** | Arc<Mutex<>> for counters; DashMap for concurrent hash maps; tokio async for I/O |
| **Production-Grade** | Hash chains for audit, hysteresis for stability, percentiles for observability |
| **Composable** | No inter-crate dependencies; traits define contracts; integrators wire together |

## Domain Model

### 1. Router Domain (Pareto-Efficient Routing)

**Core Abstraction:**
```rust
pub trait Router {
    fn route(&self, task: Task, context: RoutingContext) -> Result<Target>;
    fn audit(&self) -> AuditLog;
    fn statistics(&self) -> RoutingStats;
}

pub trait Executor {
    fn execute(&self, target: Target) -> Result<ExecutionResult>;
    fn cancel(&self, id: TaskId) -> Result<()>;
}
```

**Key Types:**
- `Task` — Unit of work to be routed
- `Target` — Destination (worker, service, queue)
- `RoutingContext` — Decision factors (priority, resource, risk)
- `AuditChain` — Immutable SHA-256-linked log
- `HysteresisState` — Prevents oscillation in decisions

**Modules:**
- `router.rs` — Core routing algorithm with Pareto preference
- `audit.rs` — Hash-chain implementation for trail preservation
- `hysteresis.rs` — Hysteresis-aware decision state machine
- `executor.rs` — Task execution interface and tracking
- `risk.rs` — Risk assessment and failover logic
- `orchestrator.rs` — Multi-router coordination

**Dependencies:** serde, sha2, uuid, thiserror

---

### 2. Metrics Domain (High-Performance Collection)

**Core Abstraction:**
```rust
pub trait MetricCollector {
    fn counter(&self, name: &str) -> Counter;
    fn gauge(&self, name: &str) -> Gauge;
    fn histogram(&self, name: &str, buckets: usize) -> Histogram;
    fn export(&self) -> MetricsSnapshot;
}

pub struct Counter {
    name: String,
    value: Arc<Mutex<u64>>,
}

pub struct Gauge {
    name: String,
    value: Arc<Mutex<f64>>,
}

pub struct Histogram {
    name: String,
    values: Arc<Mutex<Vec<u64>>>,
    buckets: usize,
}
```

**Key Features:**
- **Thread-Safe:** Arc<Mutex<>> for atomic updates
- **Fast Reads:** Minimal lock contention
- **Percentiles:** p50, p99, custom percentile(n)
- **JSON Export:** Serializable snapshots for observability

**Modules:**
- `lib.rs` — Counter, Gauge, Histogram implementations
- `registry.rs` — Centralized metric storage (DashMap-backed)
- `snapshot.rs` — Serializable metrics export
- `percentiles.rs` — Percentile calculation utilities

**Dependencies:** serde, serde_json, dashmap

---

### 3. Metering Domain (API Usage Tracking)

**Core Abstraction:**
```rust
pub trait UsageMeter {
    fn record_request(&self, req: RequestMetadata) -> Result<()>;
    fn check_quota(&self, user_id: &str, endpoint: &str) -> Result<QuotaStatus>;
    fn report(&self) -> UsageReport;
}

pub struct RequestMetadata {
    user_id: String,
    endpoint: String,
    method: HttpMethod,
    tokens_used: u32,
    latency_ms: u64,
    timestamp: DateTime<Utc>,
}

pub enum QuotaStatus {
    Allowed { remaining: u32 },
    Exceeded { reset_at: DateTime<Utc> },
}
```

**Key Patterns:**
- **Request Classification:** Track by user, endpoint, method, cost
- **Quota Enforcement:** Per-user, per-endpoint, per-time-window
- **Rate Limiting:** Token bucket or sliding window
- **Cost Tracking:** Token-based (LLM), time-based (compute), or hybrid

**Modules:**
- `meter.rs` — Core metering logic
- `quota.rs` — Quota management and enforcement
- `rate_limit.rs` — Rate limiting strategies
- `analytics.rs` — Usage analytics and reporting
- `cost.rs` — Cost model definitions

**Dependencies:** serde, thiserror, chrono, tokio

---

## Data Flow

### Routing Flow
```
Request
  ↓
[Router.route(task, context)]
  ├→ Evaluate Pareto preference (cost, latency, reliability)
  ├→ Apply hysteresis to avoid oscillation
  ├→ Record decision in AuditChain (SHA-256 linked)
  ↓
Target (service, queue, worker)
  ↓
[Executor.execute(target)]
  ├→ Track execution progress
  ├→ Record outcome in audit
  ├→ Emit metrics (latency, success/failure)
  ↓
ExecutionResult
```

### Metrics Flow
```
Application Code
  ↓
[MetricCollector.counter("requests_total").inc(1)]
  ├→ Arc<Mutex<>> atomic increment
  ├→ O(1) time complexity
  ↓
[Histogram.record(latency_ms)]
  ├→ Append to sorted vector
  ├→ Recalculate percentiles on read
  ↓
[MetricsRegistry.export()]
  ├→ Snapshot all metrics
  ├→ Serialize to JSON
  ↓
ObservabilitySystem (Prometheus, DataDog, etc.)
```

### Metering Flow
```
API Request
  ↓
[UsageMeter.record_request(metadata)]
  ├→ Classify (user, endpoint, cost model)
  ├→ Update counters via Metrics domain
  ├→ Check quota bounds
  ↓
[UsageMeter.check_quota(user_id, endpoint)]
  ├→ Calculate remaining allowance
  ├→ Apply rate limit window
  ↓
QuotaStatus
  ├→ Allow (remaining > 0) → HTTP 200
  ├→ Reject (remaining == 0) → HTTP 429 + Retry-After
  ↓
[UsageMeter.report()]
  ├→ Generate usage report (billing, analytics)
  ↓
BillingSystem / AnalyticsDashboard
```

---

## Crate Dependencies

```
phenotype-router-monitor/
├── phenotype-monitor-contracts      # Shared traits & types
│   └── deps: serde, thiserror
│
├── phenotype-router                 # Routing engine
│   ├── deps: contracts, serde, sha2, uuid
│   └── optdeps: pyo3 (Python FFI)
│
├── phenotype-metrics                # Metrics collection
│   ├── deps: contracts, serde, dashmap
│   └── no optdeps
│
├── phenotype-meter                  # API metering
│   ├── deps: contracts, metrics, serde, thiserror, tokio, chrono
│   └── no optdeps
│
├── phenotype-monitor-cli            # CLI tools
│   ├── deps: router, metrics, meter, tokio, clap
│   └── optional: pyo3 for router
│
└── phenotype-monitor-api            # HTTP API
    ├── deps: router, metrics, meter, axum, tokio, serde
    └── no optdeps
```

**Graph:**
- `contracts` → no deps
- `router`, `metrics` → contracts
- `meter` → contracts + metrics
- `cli`, `api` → router + metrics + meter

No circular dependencies.

---

## Extension Points (Ports)

### Router Extension
```rust
// Implement custom routing strategy
pub struct CustomRouter {
    inner: StandardRouter,
    preference_fn: Box<dyn Fn(&Task) -> f64>,
}

impl Router for CustomRouter {
    fn route(&self, task: Task, ctx: RoutingContext) -> Result<Target> {
        let score = (self.preference_fn)(&task);
        // Custom logic here
        self.inner.route(task, ctx)
    }
}
```

### Metrics Extension
```rust
// Implement custom metric export
pub struct PrometheusExporter;

impl MetricsExporter for PrometheusExporter {
    fn export(&self, snapshot: MetricsSnapshot) -> String {
        // Convert to Prometheus format
    }
}
```

### Metering Extension
```rust
// Implement custom cost model
pub struct TokenCostModel {
    input_cost: f32,
    output_cost: f32,
}

impl CostCalculator for TokenCostModel {
    fn cost(&self, request: &RequestMetadata) -> u32 {
        (request.tokens_used as f32 * self.input_cost) as u32
    }
}
```

---

## Thread Safety & Concurrency

### Metrics Tier
- **Counter:** Arc<Mutex<u64>> — safe for concurrent inc()
- **Gauge:** Arc<Mutex<f64>> — safe for concurrent set()
- **Histogram:** Arc<Mutex<Vec<u64>>> — safe for concurrent record()
- **Registry:** DashMap<String, Metric> — lock-free concurrent hash map

### Router Tier
- **AuditChain:** Arc<RwLock<Vec<AuditEntry>>> — readers don't block writers
- **HysteresisState:** Arc<Mutex<State>> — single state machine, serializable

### Meter Tier
- **QuotaStore:** DashMap<(UserId, Window), RemainingQuota> — lock-free quota tracking
- **RateLimiter:** Token bucket with atomic counters

---

## Observability

### Audit Trail
Every routing decision is recorded with:
- Timestamp
- Task ID
- Selected target
- Decision factors (cost, latency, risk)
- SHA-256 hash of previous entry (immutable chain)

Query example:
```rust
let chain = router.audit();
for entry in chain.iter() {
    println!("{} -> {} (cost={})", entry.task_id, entry.target, entry.cost);
}
```

### Metrics Export
```rust
let snapshot = metrics.export();
println!("{}", serde_json::to_string_pretty(&snapshot)?);
// {
//   "counters": { "requests_total": 1000, "errors_total": 5 },
//   "gauges": { "active_connections": 42 },
//   "histograms": { "latency_ms": { "p50": 10.5, "p99": 125.3 } }
// }
```

### Usage Reports
```rust
let report = meter.report();
println!("User {} used {}/{} quota", 
    report.user_id, 
    report.usage, 
    report.limit
);
```

---

## Testing Strategy

### Unit Tests
- Router: routing algorithm correctness, hysteresis edge cases, audit chain integrity
- Metrics: counter inc(), gauge set(), histogram percentiles
- Meter: quota enforcement, rate limiting, cost calculation

### Integration Tests
- Router + Executor: end-to-end task routing and execution
- Metrics + Router: routing decisions emit metrics correctly
- Meter + HTTP API: request metering with quota enforcement

### Benchmarks
- Routing: latency for 1K/10K/100K tasks
- Metrics: throughput for counter inc() (millions/sec)
- Metering: quota check latency under concurrent load

---

## Migration Path

### Phase 1: Extract Router
1. Create `phenotype-router` crate
2. Copy thegent-router source with git history
3. Update dependencies to workspace versions
4. Run tests, verify parity with original

### Phase 2: Extract Metrics
1. Create `phenotype-metrics` crate
2. Copy thegent-metrics source
3. Add registry.rs for centralized metric storage
4. Integration test with router

### Phase 3: New Metering Layer
1. Create `phenotype-meter` crate
2. Implement quota, rate-limit, cost abstractions
3. Integration test with both router and metrics

### Phase 4: CLI & API
1. Create `phenotype-monitor-cli` — real-time metrics + audit viewing
2. Create `phenotype-monitor-api` — axum HTTP API for metrics export

See `MIGRATION.md` for detailed steps.
