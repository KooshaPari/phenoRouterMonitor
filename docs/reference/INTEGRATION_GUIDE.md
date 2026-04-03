# Integration Guide: Consuming phenotype-infrakit Crates

**Version:** 0.2.0
**Last Updated:** 2026-03-30
**Audience:** Developers integrating phenotype-infrakit crates into heliosCLI, thegent, and other Phenotype projects

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture Principles](#architecture-principles)
3. [Core Crate Reference](#core-crate-reference)
4. [Dependency Injection Patterns](#dependency-injection-patterns)
5. [Hexagonal Architecture Compliance](#hexagonal-architecture-compliance)
6. [Error Handling Strategy](#error-handling-strategy)
7. [Configuration Management](#configuration-management)
8. [Health & Observability](#health--observability)
9. [Integration Examples](#integration-examples)
10. [Testing Patterns](#testing-patterns)
11. [Troubleshooting](#troubleshooting)

---

## Overview

phenotype-infrakit is a collection of **31 production-ready Rust crates** designed to simplify building scalable, observable, and resilient services within the Phenotype ecosystem. Each crate is self-contained and follows **Hexagonal Architecture** (Ports & Adapters) principles.

### Design Philosophy

- **Minimal coupling:** Crates import only what they need; no transitive bloat.
- **Port-based contracts:** All external dependencies are abstracted behind trait ports.
- **Standalone deployable:** Each crate compiles independently and has zero cross-crate source dependencies.
- **Backward-compatible:** Workspace-level dependency pinning ensures consistent versions across consumers.

### Crate Categories

| Category | Crates | Purpose |
|----------|--------|---------|
| **Foundation** | error-core, async-traits, macros, string, validation | Core types and utilities |
| **Infrastructure** | config-core, logging, health, telemetry, time | System-level concerns |
| **Resilience** | retry, rate-limit, process | Failure handling and recovery |
| **Storage & Caching** | cache-adapter, event-sourcing, state-machine | Data persistence and replay |
| **Policy & Rules** | policy-engine, port-traits, contracts | Business rule enforcement |
| **Integration** | mcp, http-client-core, git-core | External system communication |

---

## Architecture Principles

### 1. Hexagonal Architecture (Ports & Adapters)

Every service integrating phenotype-infrakit crates must adopt **Hexagonal Architecture**:

```
┌─────────────────────────────────────────────┐
│        Driving Adapters (Inbound)           │
│    (HTTP handlers, CLI commands, RPC)       │
└────────────────┬────────────────────────────┘
                 │
         ┌───────▼────────┐
         │   Application  │
         │     Layer      │
         │  (Use Cases)   │
         └───────┬────────┘
                 │
         ┌───────▼────────────────────────┐
         │   Domain Layer (Core Logic)    │
         │   - Entities                   │
         │   - Aggregates                 │
         │   - Business Rules             │
         └───────┬────────────────────────┘
                 │
         ┌───────▼────────┐
         │     Ports      │
         │  (Interfaces)  │
         └───────┬────────┘
                 │
┌────────────────▼──────────────────────────┐
│  Driven Adapters (Outbound)               │
│  (Databases, Caches, APIs, Storage)       │
└─────────────────────────────────────────────┘
```

**In practice:**

```rust
// Domain logic depends ONLY on traits (ports)
pub struct MyUseCase {
    repo: Box<dyn RepositoryPort>,
    cache: Box<dyn CachePort>,
    logger: Box<dyn LoggerPort>,
}

// Implementations depend on concrete adapters
impl MyUseCase {
    pub fn new(
        repo: Box<dyn RepositoryPort>,
        cache: Box<dyn CachePort>,
        logger: Box<dyn LoggerPort>,
    ) -> Self {
        Self { repo, cache, logger }
    }
}
```

### 2. Independent Crates (ADR-001)

Each crate in phenotype-infrakit is **fully independent**:

- ✅ Can be used standalone
- ✅ Zero imports from sibling crates (except `phenotype-contracts` for traits)
- ✅ Compiles independently
- ✅ No hidden transitive dependencies

**Consequence:** Consumers add only the crates they need.

```toml
# Cargo.toml: Only add what you use
[dependencies]
phenotype-error-core = "0.2"     # Error types
phenotype-retry = "0.2"          # Retry logic
phenotype-cache-adapter = "0.2"  # Caching (not event sourcing; not policy engine)
```

### 3. Trait-Based Contracts (Ports)

All external interactions are defined by **traits** in `phenotype-contracts` and local modules:

```rust
// Port: cache access
pub trait CachePort: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Value>>;
    async fn set(&self, key: String, value: Value, ttl: Duration) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
}

// Port: repository access
pub trait RepositoryPort: Send + Sync {
    async fn fetch(&self, id: &str) -> Result<Entity>;
    async fn store(&self, entity: Entity) -> Result<()>;
    async fn list(&self, filter: Filter) -> Result<Vec<Entity>>;
}
```

Consumer code **never** depends on concrete adapters:

```rust
pub struct MyService {
    // Always a port (trait), never a concrete adapter
    repo: Arc<dyn RepositoryPort>,
}
```

---

## Core Crate Reference

### phenotype-error-core

**Purpose:** Unified error types for the entire ecosystem.

**Exports:**
- `ErrorKind`: Enum with 11 standard error variants
- `Result<T>`: Type alias `std::result::Result<T, ErrorKind>`
- Helper methods: `not_found()`, `validation()`, `timeout()`, etc.

**Usage:**

```rust
use phenotype_error_core::{ErrorKind, Result};

fn fetch_user(id: &str) -> Result<User> {
    let user = database.find(id)
        .ok_or_else(|| ErrorKind::not_found(format!("user {}", id)))?;
    Ok(user)
}

// Consumers can pattern match on error kind
match fetch_user("123") {
    Err(ErrorKind::NotFound(msg)) => eprintln!("User not found: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
    Ok(user) => println!("User: {:?}", user),
}
```

---

### phenotype-config-core

**Purpose:** Unified configuration with multiple sources.

**Exports:**
- `Config`: Key-value container backed by `HashMap<String, serde_json::Value>`
- `ConfigSource`: Trait for loading from different sources
- `builder::ConfigBuilder`: Fluent API for construction

**Usage:**

```rust
use phenotype_config_core::Config;

// Build from multiple sources
let config = Config::new()
    .with_env_prefix("MY_APP")  // Load MY_APP_* env vars
    .with_file("/etc/app.toml") // Load TOML file
    .with_defaults(vec![
        ("log_level", "info"),
        ("cache_ttl", "3600"),
    ])
    .build()?;

let log_level: String = config.get("log_level")?;
let cache_ttl: u64 = config.get("cache_ttl")?;
```

---

### phenotype-async-traits

**Purpose:** Re-exports `async_trait` macro with a unified import path.

**Usage:**

```rust
use phenotype_async_traits::async_trait;

#[async_trait]
pub trait AsyncPort {
    async fn do_work(&self) -> Result<Output>;
}

struct ConcreteImpl;

#[async_trait]
impl AsyncPort for ConcreteImpl {
    async fn do_work(&self) -> Result<Output> {
        // async implementation
        Ok(Output::default())
    }
}
```

---

### phenotype-health

**Purpose:** Health check abstraction with pluggable implementations.

**Exports:**
- `HealthChecker`: Trait for health checks
- `HealthStatus`: Enum (Healthy, Degraded, Unhealthy)
- Standard implementations: DatabaseHealth, CacheHealth, ServiceHealth

**Usage:**

```rust
use phenotype_health::{HealthChecker, HealthStatus};

pub struct AppHealthCheck {
    db: Arc<dyn DatabasePort>,
    cache: Arc<dyn CachePort>,
}

#[async_trait]
impl HealthChecker for AppHealthCheck {
    async fn check(&self) -> HealthStatus {
        match (self.db.ping().await, self.cache.ping().await) {
            (Ok(_), Ok(_)) => HealthStatus::Healthy,
            (Err(_), _) | (_, Err(_)) => HealthStatus::Unhealthy(
                "Database or cache unavailable".into()
            ),
        }
    }
}

// Expose health endpoint
async fn health_handler(checker: Arc<dyn HealthChecker>) -> Response {
    let status = checker.check().await;
    match status {
        HealthStatus::Healthy => Response::ok(),
        HealthStatus::Degraded(msg) => Response::degraded(msg),
        HealthStatus::Unhealthy(msg) => Response::unhealthy(msg),
    }
}
```

---

### phenotype-event-sourcing

**Purpose:** Event store with hash chain integrity verification.

**Key Features:**
- Append-only event log with immutability
- SHA-256 hash chain (ADR-002)
- Event replay and snapshots
- Built-in integration tests

**Usage:**

```rust
use phenotype_event_sourcing::{EventEnvelope, EventStore};

// Define domain events
#[derive(Serialize, Deserialize, Clone)]
pub struct UserCreated {
    pub user_id: String,
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserEmailUpdated {
    pub user_id: String,
    pub new_email: String,
}

// Build event store
let store = EventStore::new();

// Append events
let event = UserCreated {
    user_id: "u123".into(),
    name: "Alice".into(),
    email: "alice@example.com".into(),
};

let envelope = store.append("user-u123", &event)?;
println!("Event stored with hash: {}", envelope.hash);

// Retrieve event history
let history = store.get_history("user-u123")?;
for envelope in history {
    println!("Event: {:?}", envelope.event_type);
}

// Verify chain integrity
let verified = store.verify_chain("user-u123")?;
assert!(verified, "Chain integrity compromised");
```

---

### phenotype-cache-adapter

**Purpose:** Tiered caching (L1 in-memory, L2 external) with fallback logic.

**Features:**
- L1: LRU in-memory cache
- L2: Redis, Memcached, or custom backends
- Automatic fallback on L2 failure
- TTL and eviction policies

**Usage:**

```rust
use phenotype_cache_adapter::{CacheAdapter, CacheConfig};

let config = CacheConfig {
    l1_capacity: 1000,
    l1_ttl: Duration::from_secs(300),
    l2_enabled: true,
    l2_backend: L2Backend::Redis("redis://localhost:6379".into()),
};

let cache = CacheAdapter::new(config)?;

// Set value
cache.set("user:123", serde_json::json!(user), Duration::from_secs(600)).await?;

// Get value (tries L1, then L2, then miss)
let user = cache.get("user:123").await?;

// Delete
cache.delete("user:123").await?;
```

---

### phenotype-policy-engine

**Purpose:** Rule evaluation engine with TOML configuration (ADR-003).

**Features:**
- Load policies from TOML
- Allow/Deny/Require rule types
- Pattern matching and field evaluation
- Built-in audit logging

**Usage:**

```rust
use phenotype_policy_engine::{PolicyEngine, Policy};

// Define policy in TOML (or inline)
let policy_toml = r#"
name = "user-access-control"
description = "Controls user resource access"
enabled = true

[[rules]]
name = "admins_can_delete"
rule_type = "allow"
field = "user.role"
pattern = "admin"
severity = "critical"

[[rules]]
name = "guests_cannot_edit"
rule_type = "deny"
field = "user.role"
pattern = "guest"
severity = "high"
"#;

let engine = PolicyEngine::new();
let policy = Policy::from_toml(policy_toml)?;
engine.register("user-access", policy)?;

// Evaluate policy
let context = serde_json::json!({
    "user": { "role": "admin" },
    "action": "delete",
});

let result = engine.evaluate("user-access", &context)?;
assert!(result.allowed());
```

---

### phenotype-state-machine

**Purpose:** Forward-only state machine with guard callbacks (ADR-004).

**Features:**
- Enforces forward-only transitions
- Guard callbacks for pre-conditions
- Transition history audit trail
- State replay

**Usage:**

```rust
use phenotype_state_machine::{StateMachine, StateOrdinal};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Pending,     // ordinal: 0
    Running,     // ordinal: 1
    Completed,   // ordinal: 2
    Archived,    // ordinal: 3
}

impl StateOrdinal for TaskState {
    fn ordinal(&self) -> u32 {
        match self {
            Self::Pending => 0,
            Self::Running => 1,
            Self::Completed => 2,
            Self::Archived => 3,
        }
    }
}

let mut state_machine = StateMachine::new(TaskState::Pending);

// Define a guard: cannot complete without required fields
let guard = |from: TaskState, to: TaskState| -> bool {
    if to == TaskState::Completed {
        // Check that required fields are present (domain logic)
        true  // or false to reject transition
    } else {
        true
    }
};

// Transition
state_machine.transition(TaskState::Running, Some(&guard))?;
state_machine.transition(TaskState::Completed, Some(&guard))?;

// Attempt regression (forward-only)
let result = state_machine.transition(TaskState::Pending, Some(&guard));
assert!(result.is_err(), "Regression rejected by state machine");

// Audit trail
let history = state_machine.history();
for (from, to) in history {
    println!("Transitioned: {:?} -> {:?}", from, to);
}
```

---

### phenotype-retry

**Purpose:** Configurable retry logic with backoff strategies.

**Features:**
- Multiple backoff strategies (linear, exponential, jitter)
- Max retries and max duration
- Predicate-based retry decisions
- Built-in for transient failures

**Usage:**

```rust
use phenotype_retry::{RetryPolicy, BackoffStrategy};
use std::time::Duration;

let policy = RetryPolicy::new()
    .with_max_retries(3)
    .with_backoff(BackoffStrategy::Exponential {
        initial: Duration::from_millis(100),
        multiplier: 2.0,
        max: Duration::from_secs(10),
    })
    .with_predicate(|err: &MyError| err.is_transient());

let result = policy.execute(|| async {
    // Fallible operation (e.g., HTTP request)
    api_call().await
}).await?;
```

---

### phenotype-logging

**Purpose:** Structured logging integration with tracing.

**Features:**
- Unified tracing span management
- Log level configuration
- JSON output support
- Async sink support

**Usage:**

```rust
use phenotype_logging::{init_tracing, LogConfig};

let config = LogConfig {
    level: "debug",
    format: "json",
    output: "stdout",
};

init_tracing(&config)?;

// Use standard tracing macros
use tracing::{info, debug, error};

info!(user_id = "u123", action = "login", "User logged in");
debug!(step = 1, "Processing request");
error!(reason = "timeout", "Request failed");
```

---

## Dependency Injection Patterns

### Pattern 1: Constructor Injection (Recommended)

```rust
pub struct MyService {
    repo: Arc<dyn RepositoryPort>,
    cache: Arc<dyn CachePort>,
    logger: Arc<dyn LoggerPort>,
}

impl MyService {
    pub fn new(
        repo: Arc<dyn RepositoryPort>,
        cache: Arc<dyn CachePort>,
        logger: Arc<dyn LoggerPort>,
    ) -> Self {
        Self { repo, cache, logger }
    }

    pub async fn get_user(&self, id: &str) -> Result<User> {
        // Check cache first
        if let Ok(Some(user)) = self.cache.get(&format!("user:{}", id)).await {
            self.logger.debug("Cache hit");
            return Ok(user);
        }

        // Fall back to repository
        let user = self.repo.fetch(id).await?;
        let _ = self.cache.set(
            format!("user:{}", id),
            user.clone(),
            Duration::from_secs(3600),
        ).await;

        Ok(user)
    }
}
```

### Pattern 2: Builder Pattern

```rust
pub struct ServiceBuilder {
    repo: Option<Arc<dyn RepositoryPort>>,
    cache: Option<Arc<dyn CachePort>>,
    logger: Option<Arc<dyn LoggerPort>>,
}

impl ServiceBuilder {
    pub fn new() -> Self {
        Self {
            repo: None,
            cache: None,
            logger: None,
        }
    }

    pub fn with_repo(mut self, repo: Arc<dyn RepositoryPort>) -> Self {
        self.repo = Some(repo);
        self
    }

    pub fn with_cache(mut self, cache: Arc<dyn CachePort>) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn with_logger(mut self, logger: Arc<dyn LoggerPort>) -> Self {
        self.logger = Some(logger);
        self
    }

    pub fn build(self) -> Result<MyService> {
        let repo = self.repo.ok_or(ErrorKind::configuration("repo not provided"))?;
        let cache = self.cache.ok_or(ErrorKind::configuration("cache not provided"))?;
        let logger = self.logger.ok_or(ErrorKind::configuration("logger not provided"))?;

        Ok(MyService { repo, cache, logger })
    }
}

// Usage
let service = ServiceBuilder::new()
    .with_repo(repo)
    .with_cache(cache)
    .with_logger(logger)
    .build()?;
```

### Pattern 3: Factory Function

```rust
pub async fn build_service(config: &Config) -> Result<Arc<MyService>> {
    let repo = Arc::new(PostgresRepository::new(&config.db_url).await?);
    let cache = Arc::new(RedisCache::new(&config.redis_url).await?);
    let logger = Arc::new(TracingLogger::new());

    Ok(Arc::new(MyService::new(repo, cache, logger)))
}

// Usage in main
#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;
    let service = build_service(&config).await?;

    // Start server with service
    start_server(service).await
}
```

### Pattern 4: Dependency Container (Advanced)

```rust
pub struct Container {
    repo: Arc<dyn RepositoryPort>,
    cache: Arc<dyn CachePort>,
    logger: Arc<dyn LoggerPort>,
}

impl Container {
    pub async fn new(config: &Config) -> Result<Self> {
        let repo = Arc::new(PostgresRepository::new(&config.db_url).await?);
        let cache = Arc::new(RedisCache::new(&config.redis_url).await?);
        let logger = Arc::new(TracingLogger::new());

        Ok(Self { repo, cache, logger })
    }

    pub fn service(&self) -> Arc<MyService> {
        Arc::new(MyService::new(
            self.repo.clone(),
            self.cache.clone(),
            self.logger.clone(),
        ))
    }

    pub fn health_checker(&self) -> Arc<dyn HealthChecker> {
        Arc::new(AppHealthCheck {
            repo: self.repo.clone(),
            cache: self.cache.clone(),
        })
    }
}
```

---

## Hexagonal Architecture Compliance

### Layer Definition

1. **Domain Layer (Core):** Pure business logic, zero dependencies on adapters
2. **Application Layer (Use Cases):** Orchestrates domain logic with port abstractions
3. **Port Layer (Interfaces):** Trait definitions for external dependencies
4. **Adapter Layer:** Concrete implementations (DB, Cache, HTTP clients, etc.)

### Project Structure

```
src/
├── domain/              # Pure business logic
│   ├── entities/
│   ├── aggregates/
│   ├── value_objects/
│   └── errors.rs
├── application/         # Use cases and services
│   ├── services/
│   ├── dto/
│   └── errors.rs
├── ports/              # Trait abstractions
│   ├── repository.rs
│   ├── cache.rs
│   └── logger.rs
├── adapters/           # Concrete implementations
│   ├── repository/
│   │   ├── postgres.rs
│   │   └── memory.rs
│   ├── cache/
│   │   └── redis.rs
│   └── logger/
│       └── tracing.rs
└── main.rs             # Wiring and startup
```

### Compliance Checklist

- ✅ Domain layer has **zero** imports from adapters
- ✅ Domain only depends on trait ports
- ✅ Application layer coordinates domain + ports
- ✅ Adapters implement ports; never imported by domain
- ✅ External library imports confined to adapter layer
- ✅ Configuration injected at startup, not hardcoded

---

## Error Handling Strategy

### Using phenotype-error-core

```rust
use phenotype_error_core::{ErrorKind, Result};

fn parse_config(data: &str) -> Result<Config> {
    serde_json::from_str::<RawConfig>(data)
        .map_err(|e| ErrorKind::serialization(e.to_string()))?
        .validate()
        .map_err(|e| ErrorKind::validation(e))
}

fn fetch_with_timeout(id: &str) -> Result<Data> {
    let future = async_fetch(id);
    tokio::time::timeout(
        Duration::from_secs(5),
        future,
    )
    .await
    .map_err(|_| ErrorKind::timeout("fetch took too long"))?
}
```

### Error Propagation

```rust
pub async fn handle_request(req: Request) -> Response {
    match process_request(req).await {
        Ok(resp) => resp,
        Err(e) => {
            match e {
                ErrorKind::NotFound(msg) => Response::not_found(msg),
                ErrorKind::Validation(msg) => Response::bad_request(msg),
                ErrorKind::Timeout(_) => Response::gateway_timeout(),
                ErrorKind::Permission(_) => Response::forbidden(),
                ErrorKind::Conflict(msg) => Response::conflict(msg),
                _ => {
                    error!("Unexpected error: {}", e);
                    Response::internal_server_error()
                }
            }
        }
    }
}
```

---

## Configuration Management

### Multi-Source Config Loading

```rust
use phenotype_config_core::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Layer 1: Defaults
    let mut config = Config::new();
    config.insert("log_level".into(), "info".into());
    config.insert("cache_ttl".into(), 3600.into());

    // Layer 2: Config file
    if let Ok(file_config) = Config::from_file("config.toml") {
        config.merge(&file_config);
    }

    // Layer 3: Environment variables
    let env_config = Config::from_env_prefix("MY_APP")?;
    config.merge(&env_config);

    // Layer 4: CLI overrides
    if let Some(env_log_level) = std::env::var("LOG_LEVEL").ok() {
        config.insert("log_level".into(), env_log_level.into());
    }

    // Validate required fields
    config.get_string("db_url")?;
    config.get_string("redis_url")?;

    Ok(())
}
```

---

## Health & Observability

### Composable Health Checks

```rust
use phenotype_health::{HealthChecker, HealthStatus};
use phenotype_async_traits::async_trait;

#[async_trait]
pub struct CompositeHealthCheck {
    checkers: Vec<(String, Arc<dyn HealthChecker>)>,
}

#[async_trait]
impl HealthChecker for CompositeHealthCheck {
    async fn check(&self) -> HealthStatus {
        let mut results = vec![];
        for (name, checker) in &self.checkers {
            match checker.check().await {
                HealthStatus::Healthy => {
                    results.push((name.clone(), "healthy".to_string()));
                }
                HealthStatus::Degraded(msg) => {
                    results.push((name.clone(), format!("degraded: {}", msg)));
                }
                HealthStatus::Unhealthy(msg) => {
                    return HealthStatus::Unhealthy(
                        format!("{} check failed: {}", name, msg)
                    );
                }
            }
        }

        HealthStatus::Healthy
    }
}

// Usage
let checks = vec![
    ("database".into(), Arc::new(DbHealthCheck { ... }) as Arc<dyn HealthChecker>),
    ("cache".into(), Arc::new(CacheHealthCheck { ... })),
    ("api".into(), Arc::new(ApiHealthCheck { ... })),
];

let composite = CompositeHealthCheck { checkers: checks };
let status = composite.check().await;
```

### Structured Logging with Tracing

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(service))]  // Don't log service (large)
async fn process_user(
    service: &MyService,
    user_id: &str,
) -> Result<()> {
    info!(%user_id, "Processing user");

    let user = service.get_user(user_id).await?;

    if user.is_suspended {
        warn!(%user_id, "User is suspended");
    }

    service.process(user).await
        .map_err(|e| {
            error!(%user_id, error = ?e, "Processing failed");
            e
        })?;

    info!(%user_id, "User processed successfully");
    Ok(())
}
```

---

## Integration Examples

### Example 1: heliosCLI Integration

**Setup:** heliosCLI needs configuration, health checks, and error handling.

```rust
// src/main.rs
use phenotype_config_core::Config;
use phenotype_error_core::Result;
use phenotype_health::{HealthChecker, HealthStatus};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load config
    let config = Config::from_env_prefix("HELIOS")?;

    // 2. Initialize logging
    let log_level = config.get_string("log_level")
        .unwrap_or_else(|_| "info".into());
    init_tracing(&log_level)?;

    // 3. Build health checker
    let health = AppHealthCheck::new(&config).await?;
    let health = Arc::new(health);

    // 4. Build services
    let service = build_service(&config, health.clone()).await?;

    // 5. Start CLI
    run_cli(service).await
}

// src/health.rs
pub struct AppHealthCheck {
    db_url: String,
    cache_url: String,
}

#[async_trait]
impl HealthChecker for AppHealthCheck {
    async fn check(&self) -> HealthStatus {
        // Check database
        if !database_reachable(&self.db_url).await {
            return HealthStatus::Unhealthy("Database unreachable".into());
        }

        // Check cache
        if !cache_reachable(&self.cache_url).await {
            return HealthStatus::Degraded("Cache unavailable (fallback ok)".into());
        }

        HealthStatus::Healthy
    }
}
```

### Example 2: thegent Integration

**Setup:** thegent needs policy evaluation, event sourcing, and state management.

```rust
// src/agent/policy_enforcer.rs
use phenotype_policy_engine::PolicyEngine;
use phenotype_error_core::Result;

pub struct AgentPolicyEnforcer {
    engine: Arc<PolicyEngine>,
}

impl AgentPolicyEnforcer {
    pub async fn can_agent_execute(&self, agent: &Agent, action: &str) -> Result<bool> {
        let context = serde_json::json!({
            "agent": {
                "id": agent.id,
                "role": agent.role,
                "permissions": agent.permissions,
            },
            "action": action,
        });

        let result = self.engine.evaluate("agent-policy", &context)?;
        Ok(result.allowed())
    }
}

// src/agent/event_log.rs
use phenotype_event_sourcing::EventStore;

pub struct AgentEventLog {
    store: Arc<EventStore>,
}

#[derive(Serialize, Deserialize)]
pub struct AgentStarted { pub agent_id: String, pub timestamp: i64 }

#[derive(Serialize, Deserialize)]
pub struct AgentTaskAssigned { pub task_id: String, pub timestamp: i64 }

impl AgentEventLog {
    pub async fn record_start(&self, agent_id: &str) -> Result<()> {
        let event = AgentStarted {
            agent_id: agent_id.into(),
            timestamp: now(),
        };
        self.store.append(&format!("agent-{}", agent_id), &event)?;
        Ok(())
    }

    pub async fn record_task_assignment(&self, agent_id: &str, task_id: &str) -> Result<()> {
        let event = AgentTaskAssigned {
            task_id: task_id.into(),
            timestamp: now(),
        };
        self.store.append(&format!("agent-{}", agent_id), &event)?;
        Ok(())
    }
}

// src/agent/state.rs
use phenotype_state_machine::{StateMachine, StateOrdinal};

#[derive(Clone, Copy, Debug)]
pub enum AgentState {
    Initialized,   // ordinal 0
    Running,       // ordinal 1
    Paused,        // ordinal 2 (skip-state allowed)
    Completed,     // ordinal 3
    Failed,        // ordinal 4
}

impl StateOrdinal for AgentState {
    fn ordinal(&self) -> u32 {
        match self {
            Self::Initialized => 0,
            Self::Running => 1,
            Self::Paused => 2,
            Self::Completed => 3,
            Self::Failed => 4,
        }
    }
}

pub struct AgentStateMachine {
    machine: StateMachine<AgentState>,
}

impl AgentStateMachine {
    pub fn new() -> Self {
        let mut machine = StateMachine::new(AgentState::Initialized);
        machine.allow_skip_state(AgentState::Paused);  // Allow I->P->C transitions
        Self { machine }
    }

    pub fn transition(&mut self, to: AgentState) -> Result<()> {
        self.machine.transition(to, None)?;
        Ok(())
    }
}
```

### Example 3: Cross-Crate Communication

**Setup:** Multiple services using shared error, config, and event sourcing.

```rust
// shared/src/domain.rs
use phenotype_error_core::Result;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkflowStarted {
    pub workflow_id: String,
    pub initiator: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkflowCompleted {
    pub workflow_id: String,
    pub duration_ms: u64,
}

// service-a/src/lib.rs
use phenotype_event_sourcing::EventStore;
use phenotype_cache_adapter::CacheAdapter;
use shared::domain::{WorkflowStarted, WorkflowCompleted};

pub struct WorkflowOrchestrator {
    events: Arc<EventStore>,
    cache: Arc<CacheAdapter>,
}

impl WorkflowOrchestrator {
    pub async fn start_workflow(&self, workflow_id: &str) -> Result<()> {
        let event = WorkflowStarted {
            workflow_id: workflow_id.into(),
            initiator: "system".into(),
        };

        // Record event
        self.events.append(workflow_id, &event)?;

        // Cache workflow state
        self.cache.set(
            format!("workflow:{}", workflow_id),
            serde_json::json!({ "status": "running" }),
            Duration::from_secs(3600),
        ).await?;

        Ok(())
    }
}

// service-b/src/lib.rs
pub struct WorkflowMonitor {
    events: Arc<EventStore>,
}

impl WorkflowMonitor {
    pub async fn get_workflow_history(&self, workflow_id: &str) -> Result<Vec<String>> {
        let history = self.events.get_history(workflow_id)?;
        Ok(history.iter().map(|e| e.event_type.clone()).collect())
    }

    pub async fn verify_workflow(&self, workflow_id: &str) -> Result<bool> {
        self.events.verify_chain(workflow_id)
    }
}
```

---

## Testing Patterns

### Unit Testing with phenotype-test-infra

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use phenotype_test_infra::{TempDir, assert_err_contains};

    #[test]
    fn test_config_loading() {
        let tmp = TempDir::new("config-test").unwrap();

        // Write test config
        let config_path = tmp.path().join("config.toml");
        std::fs::write(
            &config_path,
            r#"
            db_url = "sqlite:memory"
            log_level = "debug"
            "#,
        ).unwrap();

        // Load and verify
        let config = Config::from_file(&config_path).unwrap();
        assert_eq!(config.get_string("log_level").unwrap(), "debug");
    }

    #[test]
    fn test_error_propagation() {
        let result: Result<i32> = Err(ErrorKind::validation("invalid input"));
        assert_err_contains!(result, "invalid input");
    }
}
```

### Integration Testing with Mocks

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use async_trait::async_trait;

    struct MockRepository {
        data: std::sync::Arc<std::sync::Mutex<Vec<Entity>>>,
    }

    #[async_trait]
    impl RepositoryPort for MockRepository {
        async fn fetch(&self, id: &str) -> Result<Entity> {
            let data = self.data.lock().unwrap();
            data.iter()
                .find(|e| e.id == id)
                .cloned()
                .ok_or(ErrorKind::not_found(id))
        }

        async fn store(&self, entity: Entity) -> Result<()> {
            let mut data = self.data.lock().unwrap();
            data.push(entity);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_service_with_mock_repo() {
        let repo = Arc::new(MockRepository {
            data: Arc::new(std::sync::Mutex::new(vec![])),
        });

        let service = MyService::new(repo);
        let entity = Entity { id: "1".into(), name: "test".into() };

        service.store(entity.clone()).await.unwrap();
        let fetched = service.fetch("1").await.unwrap();
        assert_eq!(fetched.name, "test");
    }
}
```

---

## Troubleshooting

### Issue: "Missing trait bound `Send + Sync`"

**Cause:** Trait objects used in `Arc` must be `Send + Sync`.

**Solution:**

```rust
// ✅ Correct
pub trait MyPort: Send + Sync {
    async fn do_work(&self) -> Result<()>;
}

// ❌ Wrong
pub trait MyPort {
    async fn do_work(&self) -> Result<()>;
}
```

### Issue: "Cyclic dependency between crates"

**Cause:** Crate A imports Crate B, Crate B imports Crate A.

**Solution:** Extract shared traits to `phenotype-contracts` and have both depend on it.

```rust
// phenotype-contracts/src/lib.rs
pub trait RepositoryPort: Send + Sync {
    async fn fetch(&self, id: &str) -> Result<Entity>;
}

// crate-a/Cargo.toml
[dependencies]
phenotype-contracts = { path = "../../crates/phenotype-contracts" }

// crate-b/Cargo.toml
[dependencies]
phenotype-contracts = { path = "../../crates/phenotype-contracts" }
```

### Issue: "Arc + Mutex overhead in hot path"

**Cause:** Excessive locking in performance-critical code.

**Solution:** Use `DashMap` for sharded concurrency (like `phenotype-policy-engine`).

```rust
use dashmap::DashMap;

pub struct PolicyRegistry {
    policies: DashMap<String, Policy>,
}

impl PolicyRegistry {
    pub fn get(&self, key: &str) -> Option<Policy> {
        self.policies.get(key).map(|r| r.clone())
    }
}
```

### Issue: "Configuration not found at runtime"

**Cause:** Missing environment variable or config file.

**Solution:** Use configuration layering with defaults:

```rust
let config = Config::builder()
    .with_defaults(vec![
        ("log_level", "info"),
        ("cache_ttl", "3600"),
    ])
    .with_env_prefix("MY_APP")
    .with_file("/etc/app.toml")  // optional
    .require("db_url")  // fail if missing
    .build()?;
```

### Issue: "Event chain integrity verification fails"

**Cause:** Events modified post-storage or hash algorithm mismatch.

**Solution:** Never modify stored events; use event versioning if schema changes:

```rust
#[derive(Serialize, Deserialize)]
pub enum UserEvent {
    V1(UserEventV1),
    V2(UserEventV2),
}

pub struct UserEventV1 {
    pub id: String,
    pub name: String,
}

pub struct UserEventV2 {
    pub id: String,
    pub name: String,
    pub email: String,  // Added in v2
}
```

---

## Best Practices Summary

| Practice | Benefit | How |
|----------|---------|-----|
| **Always use trait ports** | Testability, swappable impls | Define traits in `ports/` module; inject via DI |
| **Never cross-import crates** | Modularity, compile-time benefits | Use `phenotype-contracts` for shared traits |
| **Layer errors consistently** | Debugging, user feedback | Map all errors to `ErrorKind` |
| **Inject configuration** | Flexibility, 12-factor compliance | Load config in `main()`, pass to services |
| **Audit all state changes** | Compliance, debugging | Use `phenotype-event-sourcing` for mutations |
| **Health-check every service** | Observability, auto-recovery | Implement `HealthChecker` trait |
| **Log with context** | Debuggability, traceability | Use `#[instrument]` macro on async fns |
| **Test with fixtures** | Repeatability, clarity | Use `phenotype-test-infra::TempDir` |

---

## Additional Resources

- **ADR.md** — Architecture Decision Records for the entire ecosystem
- **README.md** — phenotype-infrakit overview and quick start
- **Cargo.toml** — Workspace dependency versions and pinning strategy
- **crates/*/README.md** — Individual crate documentation

---

**Document Version:** 0.2.0
**Last Updated:** 2026-03-30
**Maintainer:** Phenotype Team
**License:** MIT
