# Libification & Optimization Audit Tasks

**Category:** LIBIFICATION | **Updated:** 2026-04-03

---

## 2026-04-03 - Phase 1: P0 Immediate Actions (~1,500 LOC)

**Project:** [cross-repo]
**Category:** libification
**Status:** in_progress
**Priority:** P0

### Task 1.1: Remove Nested Duplicate in agentapi-plusplus

**Location:** `agentapi-plusplus/agentapi-plusplus/`
**Issue:** Complete nested copy of source code (~23K LOC duplicate)
**Action:** Delete nested directory, use main source only
**LOC Savings:** ~500 LOC
**Effort:** 1 hour
**Status:** ⬜ Pending

```
- [ ] Delete agentapi-plusplus/agentapi-plusplus/ directory
- [ ] Verify all imports point to correct main source paths
- [ ] Update any relative imports in main source
```

---

### Task 1.2: Logrus → Slog Migration in cliproxyapi-plusplus

**Location:** `cliproxyapi-plusplus/`
**Issue:** Using deprecated `sirupsen/logrus` (security issues, unmaintained)
**Migration:**
```go
// BEFORE
import "github.com/sirupsen/logrus"
log.WithField("user", id).Info("request")

// AFTER  
import "log/slog"
logger.Info("request", "user", id)
```
**LOC Savings:** ~400 LOC
**Effort:** 4 hours
**Status:** ⬜ Pending

```
- [ ] Create slog wrapper/compat layer for gradual migration
- [ ] Migrate main.go / server initialization
- [ ] Migrate all log calls in api/ handlers
- [ ] Migrate all log calls in internal/ services
- [ ] Remove logrus dependency from go.mod
- [ ] Run full test suite to verify
```

---

### Task 1.3: Adopt backoff Crate for Retry Logic

**Location:** Multiple crates (agileplus-api, agileplus-redis, heliosCLI, phenotype-event-sourcing)
**Issue:** 4 separate retry implementations with inconsistent algorithms:
- `agileplus-api` - exponential with jitter (good)
- `agileplus-redis` - linear, no jitter (bad)
- `heliosCLI` - exponential with jitter (good)
- `phenotype-event-sourcing` - exponential with hard cap

**Solution:**
```rust
use backoff::{ExponentialBackoff, backoff::Backoff};
let backoff = ExponentialBackoff::default();
backoff::future::retry(backoff, operation).await?;
```
**LOC Savings:** ~163 LOC
**Effort:** 3 hours
**Status:** ⬜ Pending

```
- [ ] Add backoff crate to Cargo.toml
- [ ] Migrate agileplus-api/src/http/retry.rs
- [ ] Migrate agileplus-redis/src/retry.rs  
- [ ] Migrate platforms/heliosCLI/codex-rs/core/src/http/retry.rs
- [ ] Migrate crates/phenotype-event-sourcing/src/retry.rs
- [ ] Verify all tests pass
```

---

## 2026-04-03 - Phase 2: P1 High Priority (~2,000 LOC)

**Project:** [cross-repo]
**Category:** libification
**Status:** proposed
**Priority:** P1

### Task 2.1: Viper → Koanf Migration

**Locations:** 
- `agentapi-plusplus/` (~150 LOC)
- `cliproxyapi-plusplus/` (~350 LOC)

**Issue:** Using deprecated `spf13/viper` (case-insensitive bug risk, maintenance mode)

**Solution:**
```go
// BEFORE: Viper (problematic)
viper.SetConfigFile("config.yaml")
viper.AutomaticEnv()  // Case-insensitive = bug risk

// AFTER: koanf (modern)
k := koanf.New(".")
k.Load(koanf.YAML("config.yaml"), koanf.YAML)
k.Load(koanf.Env("APP_", ".", nil), koanf.Env)
```
**LOC Savings:** ~470 LOC
**Effort:** 8 hours (4 hours per repo)
**Status:** ⬜ Pending

```
- [ ] Add knadh/koanf/v2 to go.mod (agentapi-plusplus)
- [ ] Rewrite config loading in agentapi-plusplus
- [ ] Add knadh/koanf/v2 to go.mod (cliproxyapi-plusplus)
- [ ] Rewrite config loading in cliproxyapi-plusplus
- [ ] Remove spf13/viper from both repos
- [ ] Run integration tests
```

---

### Task 2.2: HTTP Client Resilience (retryablehttp + gobreaker)

**Locations:** Both Go repos
**Issue:** Custom retry loops without circuit breaker protection

**Solution:**
```go
import (
    "github.com/hashicorp/go-retryablehttp"
    "github.com/sony/gobreaker/v2"
)

// Standard client with resilience
client := retryablehttp.NewClient()
client.Backoff = retryablehttp.ExponentialJitterBackoff
client.RetryMax = 3

cb := gobreaker.New(gobreaker.Settings{
    Name:        "external-api",
    MaxRequests: 3,
    Interval:    10 * time.Second,
})
```
**LOC Savings:** ~270 LOC
**Effort:** 6 hours
**Status:** ⬜ Pending

```
- [ ] Add go-retryablehttp to go.mod
- [ ] Add sony/gobreaker/v2 to go.mod
- [ ] Create standard HTTP client wrapper in agentapi-plusplus
- [ ] Create standard HTTP client wrapper in cliproxyapi-plusplus
- [ ] Migrate all custom retry logic to use wrapper
- [ ] Verify circuit breaker behavior
```

---

### Task 2.3: Middleware Consolidation (phenotype-go-middleware)

**Locations:** Both Go repos
**Issue:** Duplicate CORS, rate limiting, JWT auth implementations

| Middleware | agentapi | cliproxyapi | Solution |
|------------|----------|-------------|----------|
| CORS | go-chi/cors | manual gin | Standardize on go-chi/cors |
| Rate limiting | - | manual | go-chi/httprate |
| Auth/JWT | chi/jwtauth | manual | Extract shared |

**Extract:** `phenotype-go-middleware` package

**LOC Savings:** ~305 LOC
**Effort:** 8 hours
**Status:** ⬜ Pending

```
- [ ] Create phenotype-go-middleware repo/module
- [ ] Implement CORS handler wrapper
- [ ] Implement rate limiting with go-chi/httprate
- [ ] Implement JWT auth with go-chi/jwtauth/v5
- [ ] Add to agentapi-plusplus dependencies
- [ ] Add to cliproxyapi-plusplus dependencies
- [ ] Migrate existing middleware to use shared package
```

---

### Task 2.4: Health Check Unification (libs/health-core)

**Locations:** Multiple Rust crates
**Issue:** 6 different health status enums

| Crate | Type | Variants |
|-------|------|----------|
| agileplus-graph | GraphHealth | Healthy, Unavailable |
| agileplus-cache | CacheHealth | Healthy, Unavailable |
| agileplus-nats | BusHealth | Connected, Disconnected |
| agileplus-domain | HealthStatus | Healthy, Degraded, Unavailable |

**Solution:**
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unavailable,
}

#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> Result<HealthStatus, HealthCheckError>;
}
```
**LOC Savings:** ~90 LOC
**Effort:** 4 hours
**Status:** ⬜ Pending

```
- [ ] Create libs/health-core crate
- [ ] Define unified HealthStatus enum
- [ ] Define HealthCheck trait
- [ ] Add HTTP handler middleware
- [ ] Update agileplus-graph to use health-core
- [ ] Update agileplus-cache to use health-core
- [ ] Update agileplus-nats to use health-core
- [ ] Update phenotype-event-sourcing to use health-core
```

---

## 2026-04-03 - Phase 3: P2 Medium Priority (~1,500 LOC)

**Project:** [cross-repo]
**Category:** libification
**Status:** proposed
**Priority:** P2

### Task 3.1: Repository Trait Consolidation

**Locations:** Multiple Rust crates
**Issue:** 6+ duplicate Store/Repository traits
**Existing but unused:** `libs/hexagonal-rs/src/ports/repository.rs`

**LOC Savings:** ~200 LOC
**Effort:** 6 hours
**Status:** ⬜ Pending

```
- [ ] Audit libs/hexagonal-rs for completeness
- [ ] Update libs/hexagonal-rs to edition 2024
- [ ] Add missing async trait methods
- [ ] Migrate phenotype-contracts to use hexagonal-rs
- [ ] Migrate agileplus-events to use hexagonal-rs
- [ ] Migrate agileplus-graph to use hexagonal-rs
- [ ] Migrate agileplus-cache to use hexagonal-rs
```

---

### Task 3.2: In-Memory Store Extraction (libs/test-stores)

**Locations:** 4 Rust crates
**Issue:** Duplicate `Arc<Mutex<HashMap>>` implementations

| Location | Implementation | LOC |
|----------|----------------|-----|
| agileplus-nats | InMemoryBus | 113 |
| agileplus-sync | InMemorySyncStore | 63 |
| agileplus-graph | InMemoryGraphBackend | 203 |
| agileplus-domain | InMemoryCredentialStore | 47 |

**Solution:** `libs/test-stores` with generic `HashMapStore<K, V>`

**LOC Savings:** ~320 LOC
**Effort:** 6 hours
**Status:** ⬜ Pending

```
- [ ] Create libs/test-stores crate
- [ ] Implement generic InMemoryStore<K, V> trait
- [ ] Implement HashMapStore<K, V> default impl
- [ ] Add builder pattern for test setup
- [ ] Migrate agileplus-nats to use test-stores
- [ ] Migrate agileplus-sync to use test-stores
- [ ] Migrate agileplus-graph to use test-stores
- [ ] Migrate agileplus-domain to use test-stores
```

---

### Task 3.3: Serialization Adapter Library (libs/serde-adapters)

**Locations:** Multiple Rust crates
**Issue:** Duplicate serialization boilerplate

| Pattern | Locations | Savings |
|---------|-----------|---------|
| Event serialization | 2 nested duplicates | 196 LOC |
| Encrypted fields | 3 crates | 70 LOC |
| MessagePack | 3 crates | 65 LOC |

**Solution:** `libs/serde-adapters` with:
- `encrypted.rs` - Encrypt/decrypt adapters
- `messagepack.rs` - MessagePack wrappers
- `versioned.rs` - Version-aware serialization

**LOC Savings:** ~273 LOC
**Effort:** 8 hours
**Status:** ⬜ Pending

```
- [ ] Create libs/serde-adapters crate
- [ ] Implement encrypted field adapter
- [ ] Implement MessagePack wrapper
- [ ] Implement versioned serialization helper
- [ ] Migrate phenotype-event-sourcing to use adapters
- [ ] Migrate agileplus-nats to use MessagePack adapter
- [ ] Migrate agileplus-domain to use encrypted adapter
```

---

### Task 3.4: Error Core Library (libs/phenotype-error-core)

**Locations:** Multiple Rust crates
**Issue:** 8+ custom error types with similar patterns

| Crate | Error Type | Similarity |
|-------|-----------|------------|
| event-sourcing | EventStoreError | Similar to policy |
| policy-engine | PolicyError | Similar to event |
| cache-adapter | CacheError | Different |
| evidence-ledger | LedgerError | Similar hash |

**LOC Savings:** ~150 LOC
**Effort:** 6 hours
**Status:** ⬜ Pending

```
- [ ] Create libs/phenotype-error-core crate
- [ ] Define DomainError trait
- [ ] Implement error code system
- [ ] Add severity classification
- [ ] Add error chain tracing
- [ ] Migrate event-sourcing to use error-core
- [ ] Migrate policy-engine to use error-core
- [ ] Migrate cache-adapter to use error-core
```

---

### Task 3.5: Test Fixture Consolidation (libs/test-fixtures)

**Locations:** Multiple repos
**Issue:** Duplicate test utilities

| Pattern | Current | Solution |
|---------|---------|----------|
| Auth fixtures | 2 duplicates | libs/test-fixtures |
| Mock servers | 2 duplicates | libs/test-fixtures |
| Schema fixtures | 2 duplicates | libs/test-fixtures |

**LOC Savings:** ~250 LOC
**Effort:** 8 hours
**Status:** ⬜ Pending

```
- [ ] Create libs/test-fixtures crate
- [ ] Implement AuthFixtureBuilder
- [ ] Implement MockServer with wiremock
- [ ] Implement SchemaFixture helpers
- [ ] Add to Python pheno-mcp tests
- [ ] Add to Rust crate tests
```

---

## 2026-04-03 - Phase 4: P2 Evaluation Candidates (~500 LOC potential)

**Project:** [cross-repo]
**Category:** research, evaluation
**Status:** proposed
**Priority:** P2

### Task 4.1: Evaluate casbin-rs for Policy Engine

**Location:** `phenotype-policy-engine`
**Current:** Custom implementation (~500 LOC)
**Alternative:** Wrap `casbin-rs` (8K stars, production-grade RBAC/ABAC)

**Evaluation:** 
- Cross-language policy definitions
- Built-in adapters for file/DB/API
- Active maintenance

**Potential Savings:** ~400 LOC
**Effort:** 12 hours (evaluation + implementation)
**Status:** ⬜ Pending

```
- [ ] Research casbin-rs v3.0 API
- [ ] Create phenotype-casbin-wrapper prototype
- [ ] Benchmark performance vs current implementation
- [ ] Migrate existing policy rules to casbin model.conf
- [ ] Verify RBAC/ABAC semantics match current
```

---

### Task 4.2: Evaluate LLM Orchestration Frameworks

**Location:** thegent, heliosCLI
**Current:** Custom agent loops

**Alternatives:**
| Framework | Language | Stars | Notes |
|-----------|----------|-------|-------|
| Mastra | TS | - | Superior to LangChain, native MCP |
| rig-core | Rust | - | "Vercel AI SDK for Rust" |
| langgraph-rs | Rust | - | Graph-based orchestration |
| CrewAI | Python | 21K | Multi-agent, A2A, MCP ready |

**Status:** ⬜ Pending

```
- [ ] Evaluate Mastra v1.2 for TS services
- [ ] Evaluate rig-core for Rust services
- [ ] Evaluate langgraph-rs for complex workflows
- [ ] Evaluate CrewAI for Python multi-agent
- [ ] Create recommendation ADR
```

---

### Task 4.3: Database Modernization Planning

**Current:** `rusqlite` (sync), raw queries
**Targets:** 
- `sqlx` (async) - compile-time SQL verification
- `sea-orm` (async ORM) - entity generation
- `drizzle-orm` (Python) - lightweight, Zod schema

**Potential Savings:** ~400 LOC per major service
**Status:** ⬜ Pending

```
- [ ] Audit current database access patterns
- [ ] Plan sqlx migration for phenotype-infrakit
- [ ] Plan sea-orm adoption for new services
- [ ] Evaluate drizzle for Python services
- [ ] Create migration roadmap
```

---

## Summary: Task Status Dashboard

| Phase | Tasks | Total LOC Savings | Status |
|-------|-------|-------------------|--------|
| P0 | 3 tasks | ~1,063 LOC | In Progress |
| P1 | 4 tasks | ~1,135 LOC | Proposed |
| P2 | 5 tasks | ~1,193 LOC | Proposed |
| P3 | 3 tasks | ~500 LOC | Evaluation |
| **TOTAL** | **15 tasks** | **~3,891 LOC** | - |

### Progress

- [ ] Phase 1: 0/3 tasks complete
- [ ] Phase 2: 0/4 tasks complete  
- [ ] Phase 3: 0/5 tasks complete
- [ ] Phase 4: 0/3 tasks complete

---

_Last updated: 2026-04-03_