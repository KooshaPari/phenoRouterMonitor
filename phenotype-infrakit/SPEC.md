# Phenotype InfraKit Specification

> Detailed specification for shared infrastructure crates

## Overview

InfraKit provides foundational infrastructure for the Phenotype ecosystem through a collection of focused, composable crates.

## Architecture Principles

1. **Single Responsibility**: Each crate does one thing well
2. **Zero-Cost Abstractions**: Compile-time optimizations where possible
3. **Async-First**: Built for async/await patterns
4. **Ergonomic APIs**: Developer-friendly interfaces
5. **Composability**: Crates work together seamlessly

## Crate Specifications

### phenotype-error-core

Error handling foundation with context propagation.

```rust
pub struct Error {
    kind: ErrorKind,
    message: String,
    context: Vec<ContextFrame>,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
    backtrace: Option<Backtrace>,
}

pub trait Context: Sized {
    fn context<C>(self, context: C) -> Result<Self, Error>
    where
        C: Into<Cow<'static, str>>;
    
    fn with_context<C, F>(self, f: F) -> Result<Self, Error>
    where
        C: Into<Cow<'static, str>>,
        F: FnOnce() -> C;
}
```

**Features:**
- Structured error types
- Context propagation
- Backtrace capture
- Error chaining
- Display and Debug formatting

### phenotype-config-core

Configuration loading with multiple sources.

```rust
pub struct ConfigLoader {
    sources: Vec<Source>,
    cache: HashMap<String, Value>,
}

pub enum Source {
    File(PathBuf, Format),
    Env { prefix: String, separator: String },
    Embedded(&'static str, Format),
    Remote { url: String, refresh: Duration },
}

pub enum Format {
    Toml,
    Yaml,
    Json,
    Ron,
}
```

**Features:**
- Multiple source types
- Hot-reload support
- Secret masking
- Validation integration
- Environment variable mapping

### phenotype-health

Health check abstraction for services.

```rust
pub struct HealthChecker {
    probes: Vec<Box<dyn Probe>>,
    timeout: Duration,
}

pub trait Probe: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self) -> HealthStatus;
    fn interval(&self) -> Duration;
}

pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}
```

**Features:**
- Multiple probe types
- Timeout handling
- Status aggregation
- Custom probe implementation
- HTTP endpoint integration

### phenotype-cache-adapter

High-performance caching with multiple backends.

```rust
pub struct Cache {
    l1: DashMap<String, CachedValue>,  // In-memory
    l2: Option<Box<dyn CacheBackend>>, // Redis, etc.
    config: CacheConfig,
}

pub struct CacheConfig {
    max_size: usize,
    ttl: Duration,
    tti: Duration,  // Time-to-idle
    eviction: EvictionPolicy,
}

pub enum EvictionPolicy {
    Lru,    // Least Recently Used
    Lfu,    // Least Frequently Used
    Fifo,   // First In First Out
    Random, // Random eviction
}
```

**Features:**
- Two-tier caching
- TTL and TTI support
- Multiple eviction policies
- Size-based limits
- Async operations

### phenotype-validation

Composable data validation.

```rust
pub trait Validator<T> {
    fn validate(&self, value: &T) -> ValidationResult;
}

pub struct ValidationResult {
    valid: bool,
    errors: Vec<ValidationError>,
}

pub struct ValidationError {
    field: String,
    message: String,
    code: String,
}

// Built-in validators
pub mod validators {
    pub struct Required;
    pub struct MinLength(usize);
    pub struct MaxLength(usize);
    pub struct Pattern(regex::Regex);
    pub struct Range<T: PartialOrd> { min: T, max: T }
    pub struct Email;
    pub struct Url;
}
```

**Features:**
- Composable validators
- Custom validation rules
- Error localization
- Schema validation
- Performance optimized

### phenotype-event-sourcing

Append-only event store with integrity guarantees.

```rust
pub struct EventStore {
    storage: Box<dyn Storage>,
    hash_chain: Sha256Chain,
}

pub struct Event {
    id: Uuid,
    aggregate_id: String,
    event_type: String,
    payload: Vec<u8>,
    timestamp: DateTime<Utc>,
    sequence: u64,
    previous_hash: Option<String>,
    hash: String,  // SHA-256 of (previous_hash + payload + timestamp)
}

pub trait Storage: Send + Sync {
    fn append(&self, event: &Event) -> Result<()>;
    fn read(&self, aggregate_id: &str) -> Result<Vec<Event>>;
    fn read_from(&self, aggregate_id: &str, sequence: u64) -> Result<Vec<Event>>;
}
```

**Features:**
- Immutable event log
- Cryptographic integrity
- Snapshot support
- Event replay
- Projections

### phenotype-state-machine

Generic finite state machine.

```rust
pub struct StateMachine<S, E, C> {
    current: S,
    transitions: Vec<Transition<S, E, C>>,
    context: C,
}

pub struct Transition<S, E, C> {
    from: S,
    to: S,
    event: E,
    guard: Option<Box<dyn Fn(&C, &E) -> bool>>,
    action: Option<Box<dyn Fn(&mut C, &E)>>,
}

impl<S: Clone, E, C> StateMachine<S, E, C> {
    pub fn transition(&mut self, event: E) -> Result<S, TransitionError>;
    pub fn current(&self) -> &S;
    pub fn can_transition(&self, event: &E) -> bool;
}
```

**Features:**
- Type-safe states and events
- Transition guards
- Entry/exit actions
- State history
- Hierarchical states

### phenotype-policy-engine

Rule-based policy evaluation.

```rust
pub struct PolicyEngine {
    policies: Vec<Policy>,
    store: Box<dyn PolicyStore>,
}

pub struct Policy {
    id: String,
    rules: Vec<Rule>,
    effect: Effect,
}

pub struct Rule {
    resource: String,
    action: String,
    condition: Condition,
}

pub enum Effect {
    Allow,
    Deny,
}

pub enum Condition {
    And(Vec<Condition>),
    Or(Vec<Condition>),
    Not(Box<Condition>),
    Equals { field: String, value: Value },
    GreaterThan { field: String, value: Value },
    In { field: String, values: Vec<Value> },
    Custom(Box<dyn Fn(&Context) -> bool>),
}
```

**Features:**
- TOML/JSON policy definitions
- Rule composition
- Context-aware evaluation
- Caching
- Audit logging

### phenotype-telemetry

Observability infrastructure.

```rust
pub struct Telemetry {
    tracer: Tracer,
    meter: Meter,
    logger: Logger,
}

pub struct Tracer {
    provider: TracerProvider,
    config: TraceConfig,
}

pub struct Meter {
    provider: MeterProvider,
    instruments: HashMap<String, Instrument>,
}

pub enum Instrument {
    Counter(Counter<u64>),
    Histogram(Histogram<f64>),
    Gauge(AsyncGauge<i64>),
    UpDownCounter(UpDownCounter<i64>),
}
```

**Features:**
- OpenTelemetry integration
- Metrics collection
- Distributed tracing
- Structured logging
- Export to multiple backends

## Integration Patterns

### Error Handling Flow

```
Application Error
    ↓
phenotype_error_core::Error
    ↓
phenotype_telemetry::record_error()
    ↓
Log / Trace / Alert
```

### Configuration Loading Flow

```
Source Priority (highest to lowest):
1. Environment variables
2. Command line arguments
3. Local config file
4. Remote config service
5. Embedded defaults
```

### Cache Invalidation Strategy

```
1. TTL-based (time-to-live)
2. Event-based (cache invalidation events)
3. Write-through (update on write)
4. Write-behind (async update)
```

## Performance Requirements

| Component | Metric | Target |
|-----------|--------|--------|
| Error creation | Time | <1μs |
| Config loading | Time | <10ms |
| Health check | Time | <100ms |
| Cache lookup | Time | <100ns |
| Event append | Time | <1ms |
| State transition | Time | <1μs |
| Policy evaluation | Time | <10μs |
| Validation | Time | <1μs |
| Metric recording | Time | <1μs |

## Security Considerations

1. **Error Messages**: No sensitive data in errors
2. **Configuration**: Secrets encrypted at rest
3. **Cache**: Isolation between tenants
4. **Events**: Immutable audit trail
5. **Policies**: Deny-by-default principle
6. **Health**: No sensitive info in health checks

## Testing Strategy

Each crate must have:
- Unit tests (>80% coverage)
- Integration tests
- Property-based tests (proptest)
- Benchmark tests
- Documentation tests

## References

- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
- [Zero to Production](https://www.zero2prod.com/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
