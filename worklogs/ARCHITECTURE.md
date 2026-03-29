# ARCHITECTURE Worklogs

Cross-cutting architecture decisions, patterns, and library extraction work.

---

## 2026-03-29 - Aggressive Library Extraction Strategy

**Project:** [cross-repo]
**Category:** architecture
**Status:** pending
**Priority:** P0

### Summary

Planning aggressive library extraction to eliminate cross-project duplication. Identified 4 CRITICAL forks needed.

### Fork Candidates

| ID | Source | Target | LOC | Priority |
|----|--------|--------|-----|----------|
| FORK-001 | `utils/pty` | `phenotype-process` | ~750 | CRITICAL |
| FORK-002 | `error.rs` pattern | `phenotype-error` | ~400 | CRITICAL |
| FORK-003 | `utils/git` | `phenotype-git` | ~300 | MEDIUM |
| FORK-004 | `utils/config` | `phenotype-config` | ~200 | MEDIUM |

### Related
- `plans/2026-03-29-FORK_EXECUTION_PLAN-v1.md`

---

## 2026-03-29 - phenotype-error Library Extraction

**Project:** [cross-repo]
**Category:** architecture
**Status:** pending
**Priority:** P0

### Summary

Extract common error types into a shared `phenotype-error` crate to eliminate 36+ duplicate error enums.

### Scope

Pattern analysis across:
- `crates/agileplus-domain/src/error.rs`
- `crates/agileplus-api/src/error.rs`
- `crates/agileplus-sync/src/error.rs`
- `crates/agileplus-p2p/src/error.rs`
- `crates/agileplus-nats/src/error.rs`
- `crates/agileplus-events/src/lib.rs`
- `crates/agileplus-graph/src/lib.rs`
- `crates/agileplus-cache/src/lib.rs`
- `libs/nexus/src/error.rs`
- `libs/hexagonal-rs/src/lib.rs`

### Common Error Variants

```rust
pub enum CoreError {
    NotFound(String),
    Conflict(String),
    InvalidInput(String),
    Serialization(String),
    Io(String),
    Timeout,
    Unauthorized,
    Forbidden,
    Storage(String),
    Config(String),
}
```

### Next Steps
- [ ] Create `libs/phenotype-error/Cargo.toml`
- [ ] Define CoreError enum
- [ ] Add thiserror derives
- [ ] Migrate first crate (agileplus-domain)
- [ ] Add blanket From implementations
- [ ] Update all dependent crates

### Related
- FORK-002

---

## 2026-03-29 - phenotype-config Library Extraction

**Project:** [cross-repo]
**Category:** architecture
**Status:** pending
**Priority:** P1

### Summary

Extract configuration loading patterns into a shared `phenotype-config` crate.

### Current Duplications

| Crate | Config Pattern | File |
|-------|---------------|------|
| agileplus-domain | TOML, `dirs_next::home_dir()` | `src/config/loader.rs` |
| agileplus-dashboard | TOML, custom `HOME` | `src/routes.rs:137-170` |
| agileplus-telemetry | YAML | `src/config.rs` |
| agileplus-subcmds | JSON | `src/sync/config.rs` |

### Pattern to Extract

```rust
pub trait ConfigLoader: Sized {
    fn load() -> Result<Self, ConfigError>;
    fn load_from(path: &Path) -> Result<Self, ConfigError>;
    fn config_path() -> PathBuf;
}

pub fn home_dir() -> PathBuf {
    dirs_next::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".phenotype")
}

pub fn ensure_config_dir() -> Result<PathBuf, ConfigError> {
    let dir = home_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
```

### Next Steps
- [ ] Create `libs/phenotype-config/Cargo.toml`
- [ ] Define ConfigError enum
- [ ] Implement ConfigLoader trait
- [ ] Migrate from agileplus-domain

---

## 2026-03-29 - phenotype-process (PTY) Library Extraction

**Project:** [cross-repo]
**Category:** architecture
**Status:** pending
**Priority:** P0

### Summary

Extract PTY/process management into a shared `phenotype-process` crate.

### Current Location
- `utils/pty/` - 750 LOC, used by heliosCLI

### Scope

```rust
// Core functionality
pub struct PtyProcess {
    master_fd: RawFd,
    child_pid: pid_t,
    pty_width: u16,
    pty_height: u16,
}

impl PtyProcess {
    pub fn spawn(cmd: &[&str], cwd: Option<&Path>) -> Result<Self>;
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()>;
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    pub fn write(&mut self, buf: &[u8]) -> Result<usize>;
    pub fn kill(&mut self) -> Result<()>;
    pub fn wait(&mut self) -> Result<i32>;
}

pub struct PtyReader { /* ... */ }
pub struct PtyWriter { /* ... */ }
```

### Related
- FORK-001

---

## 2026-03-29 - phenotype-git Library Extraction

**Project:** [cross-repo]
**Category:** architecture
**Status:** pending
**Priority:** P1

### Summary

Extract git operations into a shared `phenotype-git` crate.

### Current Locations

| Location | Pattern | LOC |
|----------|---------|-----|
| `utils/git/` | Raw git commands | ~300 |
| `agileplus-git/` | Full git library | N/A |

### Recommendation

Evaluate whether to:
1. Fork `agileplus-git` to `phenotype-git`
2. Use `gix` directly (already a dependency)
3. Create a lightweight wrapper

### Next Steps
- [ ] Audit `utils/git` vs `agileplus-git` overlap
- [ ] Decide on approach
- [ ] Create/expose library

---

## 2026-03-29 - Health Status Consolidation

**Project:** [cross-repo]
**Category:** architecture
**Status:** pending
**Priority:** P1

### Summary

Consolidate 4 health enum definitions into a single shared type.

### Current Definitions

```rust
// agileplus-graph/src/health.rs
pub enum GraphHealth { Healthy, Unavailable }

// agileplus-cache/src/health.rs
pub enum CacheHealth { Healthy, Unavailable }

// agileplus-nats/src/health.rs
pub enum BusHealth { Connected, Disconnected }

// agileplus-domain/src/domain/service_health.rs
pub enum HealthStatus { Healthy, Degraded, Unavailable }
```

### Target: phenotype-health

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HealthLevel {
    Healthy,
    Degraded,
    Unavailable,
}

#[derive(Clone, Debug)]
pub struct HealthCheck<T = ()> {
    pub level: HealthLevel,
    pub component: String,
    pub metadata: T,
    pub timestamp: DateTime<Utc>,
}

pub trait HealthRegistry: Send + Sync {
    fn register(&self, component: &str, check: Box<dyn Fn() -> HealthLevel + Send + Sync>);
    fn check_all(&self) -> Vec<HealthCheck>;
}
```

### Related
- ADRs: 001-spec-driven-development-engine

---

## 2026-03-29 - Async Store Traits Standardization

**Project:** [cross-repo]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Standardize async store trait patterns across event, cache, and graph stores.

### Current Traits

```rust
// agileplus-events/src/store.rs
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append(&self, event: &Event) -> Result<i64, EventError>;
    async fn get_events(&self, entity_type: &str, entity_id: i64) -> Result<Vec<Event>, EventError>;
}

// agileplus-cache/src/store.rs
#[async_trait]
pub trait CacheStore: Send + Sync {
    async fn get<T>(&self, key: &str) -> Result<Option<T>, CacheError>;
    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<(), CacheError>;
}

// agileplus-graph/src/store.rs
#[async_trait]
pub trait GraphBackend: Send + Sync {
    async fn run_cypher(&self, query: &str, params: &Value) -> Result<(), GraphError>;
    async fn health_check(&self) -> Result<(), GraphError>;
}
```

### Proposal: Unified Store Trait

```rust
#[async_trait]
pub trait Store<K, V>: Send + Sync {
    type Error;
    async fn get(&self, key: &K) -> Result<Option<V>, Self::Error>;
    async fn set(&self, key: &K, value: &V) -> Result<(), Self::Error>;
    async fn delete(&self, key: &K) -> Result<(), Self::Error>;
    async fn exists(&self, key: &K) -> Result<bool, Self::Error>;
}

#[async_trait]
pub trait EventStore: Send + Sync {
    type Error;
    async fn append(&self, event: &Event) -> Result<i64, Self::Error>;
    async fn get_events(&self, entity_type: &str, entity_id: i64) -> Result<Vec<Event>, Self::Error>;
}
```

---

## 2026-03-29 - hexagonal-rs Integration Audit

**Project:** [cross-repo]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Audit `libs/hexagonal-rs` usage across crates.

### Current Usage

| Crate | Uses hexagonal-rs? |
|-------|------------------|
| agileplus-domain | Partial |
| agileplus-api | No |
| agileplus-sync | No |
| agileplus-cli | No |

### hexagonal-rs Components

```rust
pub mod core {
    pub trait Repository;
    pub trait Service;
    pub trait AggregateRoot;
}

pub mod ports {
    pub trait InboundPort;
    pub trait OutboundPort;
}

pub mod adapters {
    pub struct InMemoryRepository;
    pub struct RestInboundAdapter;
}
```

### Recommendation

1. Document which hexagonal concepts are used
2. Consider adopting full hexagonal-rs or removing partial usage
3. Standardize on one approach

---

## 2026-03-29 - Event Sourcing Architecture Review

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P1

### Summary

Review event sourcing implementation in `agileplus-events`.

### Current Implementation

```rust
// agileplus-events/src/lib.rs
pub struct Event {
    id: i64,
    entity_type: String,
    entity_id: i64,
    event_type: String,
    payload: Value,
    metadata: Value,
    created_at: DateTime<Utc>,
}
```

### Issues Identified

1. No versioning strategy for event schemas
2. Upcasting mechanism missing
3. No event replay optimization
4. Snapshot strategy not defined

### Recommendations

1. Add schema versioning to events
2. Implement upcasters for schema evolution
3. Add configurable snapshot intervals
4. Consider event sourcing framework integration

---

## 2026-03-29 - NATS JetStream Integration

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P1

### Summary

Review NATS JetStream integration for event persistence.

### Current Usage

```rust
// agileplus-nats/src/bus.rs
pub enum EventBusError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Publish error: {0}")]
    PublishError(String),
}

pub struct NatsEventBus {
    client: async_nats::Client,
    prefix: String,
}
```

### Gaps

1. No JetStream persistence configuration
2. Consumer groups not implemented
3. No dead letter queue
4. Backpressure handling not defined

### Next Steps
- [ ] Add JetStream configuration
- [ ] Implement consumer groups
- [ ] Add DLQ handling
- [ ] Document backpressure strategy

---

## 2026-03-29 - Graph Database Architecture (Neo4j)

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Review graph database architecture for knowledge graphs.

### Current Stack

```rust
// agileplus-graph/src/lib.rs
pub struct GraphStore {
    client: neo4j::Driver,
    database: String,
}

impl GraphBackend for GraphStore {
    async fn run_cypher(&self, query: &str, params: &Value) -> Result<(), GraphError>;
    async fn query_cypher(&self, query: &str, params: &Value) -> Result<Vec<Value>, GraphError>;
}
```

### Considerations

1. Connection pooling strategy
2. Cypher query validation
3. Transaction management
4. Schema indexing

### Related
- `crates/agileplus-graph/src/config.rs`

---

## 2026-03-29 - Cache Layer Architecture (Redis)

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Review Redis cache layer architecture.

### Current Implementation

```rust
// agileplus-cache/src/lib.rs
pub struct RedisCache {
    client: redis::Client,
    pool: bb8::Pool<redis::aio::MultiplexedConnection>,
}

impl CacheStore for RedisCache {
    async fn get<T>(&self, key: &str) -> Result<Option<T>, CacheError>;
    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<(), CacheError>;
}
```

### Optimization Opportunities

1. Pipeline batching for bulk operations
2. Connection pooling tuning
3. TTL strategies per cache type
4. Cache warming on startup

---

## 2026-03-29 - API Layer Architecture (Axum)

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P1

### Summary

Review REST API architecture using Axum.

### Current Structure

```rust
// crates/agileplus-api/src/lib.rs
pub async fn create_entity(
    Extension(state): Extension<ApiState>,
    Json(payload): Json<CreateEntity>,
) -> Result<Json<Entity>, ApiError>;

pub async fn get_entity(
    Extension(state): Extension<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<Entity>, ApiError>;
```

### Recommendations

1. Add OpenAPI documentation
2. Implement rate limiting
3. Add request ID tracing
4. Standardize error responses

---

## 2026-03-29 - gRPC Layer Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Review gRPC layer implementation for high-performance APIs.

### Current Implementation

| File | Description |
|------|-------------|
| `crates/agileplus-grpc/src/lib.rs` | Main gRPC service |
| `crates/agileplus-grpc/src/event_bus.rs` | Event bus integration |
| `schemas/agileplus.proto` | Proto definitions |

### Proto Structure

```protobuf
service AgilePlusService {
    rpc CreateEntity(CreateEntityRequest) returns (EntityResponse);
    rpc GetEntity(GetEntityRequest) returns (EntityResponse);
    rpc StreamEvents(StreamEventsRequest) returns (stream Event);
}
```

### Next Steps
- [ ] Review proto compatibility
- [ ] Add health check RPC
- [ ] Implement bidirectional streaming

---

## 2026-03-29 - CLI Architecture (clap)

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Review CLI architecture for agileplus-cli and subcommands.

### Current Structure

```rust
// crates/agileplus-cli/src/main.rs
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Sync,
    Spec,
    Pull,
}
```

### Opportunities

1. Shared command framework across projects
2. Shell completion generation
3. Configurable output formats
4. Interactive prompts for missing args

---

## 2026-03-29 - Telemetry Architecture (OTEL)

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Review OpenTelemetry integration for observability.

### Current Stack

| Component | Implementation |
|-----------|----------------|
| Tracing | opentelemetry + tracing |
| Metrics | metrics + prometheus |
| Logging | tracing + env_logger |

### Gaps

1. No distributed tracing correlation
2. Missing exemplar support
3. No tail-based sampling
4. Dashboard templates not defined

---

## 2026-03-29 - Secrets Management Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P1

### Summary

Review secrets management across crates.

### Current Pattern

```rust
// crates/agileplus-domain/src/config/credentials.rs
pub struct Credentials {
    token: String,
    refresh_token: Option<String>,
}

impl Credentials {
    pub fn from_env() -> Result<Self, ConfigError>;
    pub fn from_file(path: &Path) -> Result<Self, ConfigError>;
}
```

### Recommendations

1. Integrate with Doppler/Infisical
2. Add secret rotation support
3. Implement secret leasing
4. Add audit logging

---

## 2026-03-29 - Specification-Driven Development Engine

**Project:** [AgilePlus]
**Category:** architecture
**Status:** in_progress
**Priority:** P0

### Summary

ADR-001 implementation: Spec-driven development engine.

### Architecture

```
spec.md -> Parser -> AST -> Generator -> Code/Docs
              |
              v
         Validator -> Linter
```

### Components

| Component | Status | File |
|-----------|--------|------|
| Parser | Implemented | `crates/agileplus-spec/` |
| Validator | Implemented | `crates/agileplus-spec/` |
| Generator | Planned | TBD |
| Linter | Planned | TBD |

### Next Steps
- [ ] Complete generator implementation
- [ ] Add linter rules
- [ ] Integrate with CLI
- [ ] Document DSL

### Related
- `docs/specs/001-spec-driven-development-engine/spec.md`

---

## 2026-03-29 - Migration: git2 -> gix

**Project:** [cross-repo]
**Category:** architecture
**Status:** pending
**Priority:** P1

### Summary

Plan migration from `git2` to `gix` for async-first git operations.

### Current State

```toml
# Current usage
git2 = "0.18"
```

### Target State

```toml
# Target usage
gix = "0.71"
```

### Migration Steps

1. Identify all `git2` usages
2. Create `gix` wrapper with similar API
3. Run tests in parallel
4. Cut over incrementally

### Challenges

1. API differences (sync vs async)
2. Missing features in `gix`
3. Performance characteristics differ

---

## 2026-03-29 - Workspace Dependency Graph Analysis

**Project:** [cross-repo]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Analyze cargo workspace dependency graph for circular dependencies and optimization.

### Current Structure

```
agileplus-domain (no deps)
agileplus-api (depends: domain)
agileplus-sync (depends: domain, events)
agileplus-cli (depends: domain, api, sync)
```

### Tools

```bash
cargo tree -d  # Show dependency cycles
cargo metadata --format-version 1 | jq '.packages[] | .name'
```

### Next Steps
- [ ] Generate dependency graph
- [ ] Identify cycles
- [ ] Plan flattening
- [ ] Document layer boundaries

---

## 2026-03-29 - Rust Edition and MSRV Strategy

**Project:** [cross-repo]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Standardize Rust edition and minimum supported Rust version.

### Current State

| Crate | Edition | MSRV |
|-------|---------|------|
| agileplus-domain | 2021 | 1.75 |
| agileplus-api | 2021 | 1.75 |
| agileplus-cli | 2021 | 1.75 |
| heliosCLI | 2021 | 1.70 |
| thegent | 2021 | 1.75 |

### Recommendations

1. Set MSRV to 1.75 for all crates
2. Use edition 2021 consistently
3. Add MSRV CI checks
4. Document MSRV policy

---

## 2026-03-29 - Feature Flag Architecture

**Project:** [cross-repo]
**Category:** architecture
**Status:** pending
**Priority:** P3

### Summary

Design feature flag system for progressive rollouts.

### Use Cases

1. A/B testing for AI features
2. Gradual rollout of new commands
3. Beta features with opt-in
4. Region-specific functionality

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| LaunchDarkly | Full-featured, hosted | Expensive |
| Unleash | Self-hosted option | Complex |
| Custom | Full control | Build time |
| Environment vars | Simple | No targeting |

### Recommendation

Start with environment-variable based flags, evaluate Unleash for scale.

---

## 2026-03-29 - Database Migration Strategy

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Design database migration strategy for PostgreSQL/Neo4j.

### Current Tools

| Database | Migration Tool |
|----------|---------------|
| PostgreSQL | SQL migrations |
| Neo4j | Cypher scripts |
| Redis | N/A |

### Recommendations

1. Use `sqlx` migrations for PostgreSQL
2. Version Cypher scripts for Neo4j
3. Add migration CI checks
4. Document rollback procedures

---

## 2026-03-29 - Multi-tenancy Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P1

### Summary

Design multi-tenant architecture for shared deployments.

### Requirements

1. Tenant isolation at data layer
2. Tenant-scoped configuration
3. Cross-tenant operations (admin)
4. Tenant migration tools

### Approaches

| Approach | Isolation | Complexity |
|----------|-----------|------------|
| Database per tenant | Highest | High |
| Schema per tenant | Medium | Medium |
| Row-level security | Lowest | Low |

### Recommendation

Row-level security with tenant_id columns for initial implementation.

---

## 2026-03-29 - Backup and Recovery Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P1

### Summary

Design backup and recovery strategy.

### Components to Backup

| Component | Frequency | Retention |
|-----------|-----------|-----------|
| PostgreSQL | Hourly | 30 days |
| Neo4j | Daily | 30 days |
| Redis | N/A (ephemeral) | N/A |
| S3/MinIO | Daily incremental | 90 days |

### RTO/RPO Targets

- RTO: 1 hour
- RPO: 1 hour

### Tools

1. `pg_dump` / `pgBackRest` for PostgreSQL
2. Neo4j backup utility
3. S3 lifecycle policies

---

## 2026-03-29 - API Versioning Strategy

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Design API versioning strategy for REST and gRPC.

### REST API

| Version | URL Pattern | Status |
|---------|-------------|--------|
| v1 | `/api/v1/` | Deprecated |
| v2 | `/api/v2/` | Current |

### gRPC

| Version | Package | Status |
|---------|---------|--------|
| v1 | `agileplus.v1` | Current |

### Recommendation

1. URL-based versioning for REST
2. Package-based versioning for gRPC
3. Maintain deprecation timeline
4. Document breaking changes

---

## 2026-03-29 - Rate Limiting Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Design rate limiting for API protection.

### Strategies

| Strategy | Granularity | Storage |
|----------|-------------|---------|
| Fixed window | Per API key | Redis |
| Sliding window | Per API key | Redis |
| Token bucket | Per API key | Redis |

### Limits

| Tier | Requests/min | Burst |
|------|-------------|-------|
| Free | 60 | 10 |
| Pro | 600 | 100 |
| Enterprise | 6000 | 1000 |

### Implementation

```rust
pub struct RateLimiter {
    store: RedisPool,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub async fn check(&self, key: &str) -> Result<RateLimitResult, Error>;
}
```

---

## 2026-03-29 - Idempotency Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Design idempotency for safe retries.

### Approach

```rust
pub struct IdempotencyKey {
    pub key: String,
    pub created_at: DateTime<Utc>,
    pub response: Option<Response>,
}

#[derive(Clone, Copy)]
pub enum IdempotencyStatus {
    New,
    InProgress,
    Completed,
}
```

### Storage

Redis with TTL matching SLA window (24h).

### Endpoints

Add `Idempotency-Key` header to all write operations.

---

## 2026-03-29 - CQRS Pattern Adoption

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Evaluate CQRS for command/query separation.

### Current State

Monolithic services handling both commands and queries.

### Target State

```rust
// Commands
pub trait CommandHandler<C: Command> {
    type Result;
    async fn handle(&self, cmd: C) -> Self::Result;
}

// Queries
pub trait QueryHandler<Q: Query> {
    type Result;
    async fn handle(&self, query: Q) -> Self::Result;
}
```

### Benefits

1. Independent scaling
2. Optimized read/write models
3. Clearer domain boundaries

### Challenges

1. Eventual consistency
2. Increased complexity
3. Data synchronization

---

## 2026-03-29 - Outbox Pattern Implementation

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Implement transactional outbox for reliable events.

### Problem

Direct event publishing can fail after DB commit.

### Solution

```sql
CREATE TABLE outbox (
    id UUID PRIMARY KEY,
    aggregate_type TEXT NOT NULL,
    aggregate_id UUID NOT NULL,
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    processed_at TIMESTAMPTZ
);
```

### Relay Process

1. Write event to outbox (same transaction)
2. Background worker polls outbox
3. Publish to NATS
4. Mark as processed

### Benefits

1. At-least-once delivery
2. No 2PC required
3. Retry-friendly

---

## 2026-03-29 - Domain Event Schema Registry

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Design schema registry for domain events.

### Requirements

1. Schema versioning
2. Backward compatibility checks
3. Schema evolution rules
4. Registry API

### Options

| Registry | Pros | Cons |
|----------|------|------|
| Confluent Schema Registry | Mature | Kafka-focused |
| AWS Glue | Managed | AWS-only |
| Custom | Full control | Build time |

### Recommendation

Use Confluent Schema Registry with Avro/JSON Schema.

---

## 2026-03-29 - Observability Pipeline Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Design centralized observability pipeline.

### Components

```yaml
# Architecture
application -> otel-collector -> transform -> export
                                      |
                                      v
                              prometheus (metrics)
                              loki (logs)
                              tempo (traces)
```

### Configuration

```yaml
# otel-collector.yaml
receivers:
  otlp:
    protocols:
      grpc:
      http:

processors:
  batch:
  memory_limiter:
  transform:

exporters:
  prometheusremotewrite:
  loki:
  otlp:
    endpoint: tempo:4317
```

### Dashboards

1. Service health
2. Request latency
3. Error rates
4. Resource utilization

---

## 2026-03-29 - Service Mesh Evaluation (Linkerd/Istio)

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P3

### Summary

Evaluate service mesh for microservices communication.

### Options

| Mesh | Complexity | Features | Resource Cost |
|------|------------|----------|---------------|
| Linkerd | Low | mTLS, observability | ~10% per pod |
| Istio | High | Full featured | ~50% per pod |
| Cilium | Medium | eBPF-based | Low |

### Recommendation

Start with Linkerd for simplicity, evaluate based on requirements.

### Considerations

1. mTLS between services
2. Traffic splitting
3. Circuit breaking
4. observability

---

## 2026-03-29 - Build Caching Strategy

**Project:** [cross-repo]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Optimize build caching for CI/CD.

### Current State

No remote caching configured.

### Target Configuration

```toml
# .cargo/config.toml
[build]
incremental = true

[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "target-feature=+crt-static"]

[profile.dev]
opt-level = 0

[profile.release]
lto = true
codegen-units = 1
```

### sccache Setup

```bash
export RUSTC_WRAPPER=sccache
export SCCACHE_GHA_ENABLED=true
```

### Expected Improvements

- 50-80% faster rebuilds
- Reduced CI costs

---

## 2026-03-29 - Dependency Update Automation

**Project:** [cross-repo]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Implement automated dependency updates.

### Tools

| Tool | Purpose |
|------|---------|
| Renovate | PR-based updates |
| Dependabot | GitHub-native |
| cargo-outdated | Version checking |

### Configuration

```json
{
  "packageRules": [
    {
      "matchDatasources": ["crate"],
      "groupName": "all-minor-patches",
      "schedule": ["every weekend"]
    }
  ]
}
```

### Workflow

1. Bot creates PR with updates
2. CI runs tests
3. Maintainer reviews and merges
4. Changelog auto-generated

---

## 2026-03-29 - Security Scanning Integration

**Project:** [cross-repo]
**Category:** architecture
**Status:** pending
**Priority:** P1

### Summary

Integrate security scanning into CI/CD.

### Tools

| Tool | Checks |
|------|--------|
| cargo-audit | Advisory database |
| cargo-deny | License compliance |
| semgrep | Code patterns |
| trivy | Container images |

### CI Configuration

```yaml
# .github/workflows/security.yml
- name: Security audit
  run: cargo audit --deny warnings

- name: License check
  run: cargo deny check

- name: Semgrep scan
  uses: returntocorp/semgrep-action@v1
```

### Frequency

- Full scan: Daily
- PR scan: On every PR
- Container scan: On push to main

---

## 2026-03-29 - Cargo Workspace Restructuring

**Project:** [cross-repo]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Restructure cargo workspace for better organization.

### Current Structure

```
/ (workspace root)
├── Cargo.toml (workspace)
├── Cargo.lock
├── crates/
│   ├── agileplus-domain/
│   ├── agileplus-api/
│   └── ...
└── libs/
    ├── nexus/
    ├── hexagonal-rs/
    └── ...
```

### Proposed Structure

```
/ (workspace root)
├── Cargo.toml (workspace)
├── Cargo.lock
├── platform/
│   ├── agileplus/
│   ├── heliosCLI/
│   └── thegent/
├── lib/
│   ├── phenotype-error/
│   ├── phenotype-config/
│   └── phenotype-health/
└── crates/
    └── ... (remaining crates)
```

### Benefits

1. Clearer ownership
2. Better dependency boundaries
3. Easier workspace updates
