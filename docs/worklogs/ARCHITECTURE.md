# Architecture Worklogs

**Category:** ARCHITECTURE | **Updated:** 2026-03-29 (Wave 93)

---
---

## 2026-03-30 - Standardized Port & Adapter Architecture

**Project:** [cross-repo]
**Category:** architecture, standardization
**Status:** in_progress
**Priority:** P0

### Summary

Established a canonical port hierarchy to resolve the split between `phenotype-port-interfaces` and `agileplus-domain/src/ports`. All new crates must use `phenotype-port-traits` for interface definitions to enable true plug-and-play adapter swapping across ecosystems.

### Port Hierarchy

| Layer | Responsibility | Example Trait |
|-------|----------------|---------------|
| **Core** | Fundamental IO | `AsyncReader`, `AsyncWriter` |
| **Domain** | Business Logic Ports | `Repository<T>`, `EventBus<E>` |
| **Infrastructure** | System Services | `SecretManager`, `ConfigLoader` |

### Standard Trait: Repository<T, ID>

```rust
#[async_trait]
pub trait Repository<T, ID>: Send + Sync 
where 
    T: Entity<ID>,
    ID: Identifier
{
    async fn save(&self, entity: T) -> Result<(), PortError>;
    async fn find_by_id(&self, id: &ID) -> Result<Option<T>, PortError>;
    async fn delete(&self, id: &ID) -> Result<(), PortError>;
    async fn list_all(&self) -> Result<Vec<T>, PortError>;
}
```

### 2026-03-30 - Python phenoSDK Architecture Evolution

**Project:** [python-sdk]
**Category:** architecture
**Status:** proposed
**Priority:** P1

### Summary

Proposed architectural shift for `phenosdk` to move from a monolithic package to a modular "plugin" architecture based on `FastMCP v3.5`.

### Proposed Structure

```
python/phenosdk/
├── pheno-core/          # Core abstractions & FastMCP wrapper
├── pheno-mcp/           # MCP transport & tool registry
├── pheno-shared/        # Shared Pydantic models (synced with Rust via buf)
└── pheno-plugins/       # dynamic tool collections
```

---

### Architecture Principles Observed

| Principle | Status | Implementation |
|-----------|--------|----------------|
| Hexagonal Architecture | ✅ Present | Ports/adapters separation |
| Error propagation | ✅ Consistent | thiserror + anyhow |
| Async-first | ✅ Present | tokio runtime |
| Serialization | ✅ Unified | serde ecosystem |
| Testing | ⚠️ Basic | Unit tests only |
| Documentation | ⚠️ Minimal | Inline docs only |

### Crate Dependency Graph

```
evidence-ledger
    └── (standalone - no internal deps)
           │
phenotype-cache-adapter
    ├── dashmap (concurrent maps)
    ├── moka (TTL cache)
    └── serde (serialization)
           │
phenotype-contracts
    └── serde (serialization)
           │
phenotype-event-sourcing
    ├── sha2 (chain hashing)
    ├── chrono (timestamps)
    ├── serde (serialization)
    ├── parking_lot (sync primitives)
    └── thiserror (errors)
           │
phenotype-policy-engine
    ├── serde (serialization)
    ├── thiserror (errors)
    └── [inner crate - same functionality]
           │
phenotype-state-machine
    └── (NO src/ in outer - only inner exists)
```

### Architecture Quality Assessment

#### ✅ Strengths

1. **Clean dependency graph** - No circular dependencies
2. **Minimal dependencies** - Each crate has focused purpose
3. **Consistent error handling** - thiserror + anyhow pattern
4. **Modern Rust** - Edition 2024, tokio, parking_lot
5. **Serialization-agnostic** - serde_json, serde_yaml, serde

#### ⚠️ Concerns

1. **Nested crate structure** - `crates/X/X/` pattern during rebase
2. **Incomplete state-machine** - No outer src/ directory
3. **Minimal testing** - No property-based or integration tests
4. **Limited documentation** - No rustdoc on public APIs
5. **Inner crate duplication** - phenotype-policy-engine has inner copy

### Port/Trait Architecture

#### phenotype-event-sourcing Ports

```rust
// phenotype-event-sourcing/src/store.rs
#[async_trait]
pub trait EventStore<T: Aggregate> {
    async fn append(&mut self, event: EventEnvelope<T>) -> Result<(), EventStoreError>;
    async fn get_events(&self, id: &T::Id) -> Result<Vec<EventEnvelope<T>>, EventStoreError>;
    async fn get_snapshots(&self, id: &T::Id) -> Result<Vec<Snapshot<T>>, EventStoreError>;
}
```

#### phenotype-cache-adapter Ports

```rust
// phenotype-cache-adapter/src/lib.rs
pub trait CacheBackend: Send + Sync {
    async fn get(&self, key: &str) -> Option<Vec<u8>>;
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) -> Result<(), CacheError>;
    async fn delete(&self, key: &str) -> Result<(), CacheError>;
}
```

#### Evidence Ledger Ports

```rust
// evidence-ledger/src/lib.rs
pub trait LedgerBackend: Send + Sync {
    async fn append(&self, entry: EvidenceEntry) -> Result<Hash, LedgerError>;
    async fn verify(&self, chain: &Chain) -> Result<bool, LedgerError>;
    async fn query(&self, filter: &QueryFilter) -> Result<Vec<EvidenceEntry>, LedgerError>;
}
```

### Recommended Architecture Improvements

| Improvement | Priority | Effort | Impact |
|-------------|----------|--------|--------|
| Add rustdoc to public APIs | 🟡 MEDIUM | 1 day | Quality |
| Add property-based tests | 🟡 MEDIUM | 3 days | Reliability |
| Extract shared error types | 🟠 HIGH | 2 days | DRY |
| Consolidate nested crates | 🔴 CRITICAL | 1 day | Cleanup |
| Add integration tests | 🟡 MEDIUM | 2 days | Confidence |

### Related

- Dependencies: `worklogs/DEPENDENCIES.md`
- Quality: `worklogs/QUALITY.md`

---

## 2026-03-29 - Event Sourcing Architecture Deep Dive

**Project:** phenotype-infrakit
**Category:** architecture
**Status:** completed
**Priority:** P1

### Summary

Deep analysis of `phenotype-event-sourcing` crate architecture, patterns, and opportunities.

### Module Structure

```
phenotype-event-sourcing/src/
├── lib.rs          # 20 LOC - exports
├── error.rs        # 46 LOC - domain errors
├── hash.rs         # 195 LOC - SHA-256 chain
├── event.rs        # 98 LOC - event envelope
├── snapshot.rs     # 92 LOC - snapshot logic
├── store.rs        # 64 LOC - event store trait
└── memory.rs       # 266 LOC - in-memory implementation
```

### Total: ~781 LOC

### Architecture Pattern: Aggregate Root

```rust
// Generic aggregate trait - core of event sourcing
pub trait Aggregate: Send + Sync + 'static {
    type Id: IdType;
    type Event: EventType;
    
    fn aggregate_id(&self) -> &Self::Id;
    fn apply(&mut self, event: Self::Event);
}

// Event envelope with metadata
pub struct EventEnvelope<T: Aggregate> {
    pub id: Uuid,
    pub aggregate_id: T::Id,
    pub sequence: u64,
    pub timestamp: DateTime<Utc>,
    pub payload: T::Event,
    pub metadata: EventMetadata,
}
```

### Chain Hashing Pattern

```rust
// SHA-256 chain for tamper detection
pub struct ContentHash {
    pub algorithm: HashAlgorithm,
    pub previous_hash: Option<Vec<u8>>,
    pub current_hash: Vec<u8>,
    pub timestamp: DateTime<Utc>,
}

// Chain verification
impl ContentHash {
    pub fn verify_chain(&self, events: &[EventEnvelope]) -> bool {
        // Verify each event's hash chains to previous
    }
}
```

### Snapshot Strategy

```rust
pub struct SnapshotConfig {
    pub max_events: u64,           // Snapshot every N events
    pub max_age: Duration,         // Or every N hours
    pub compaction_threshold: u64, // Compaction point
}
```

### In-Memory Implementation Quality

```rust
// crates/phenotype-event-sourcing/src/memory.rs:266 LOC
// Uses parking_lot::RwLock for better performance
// Implements: EventStore<TAggregate>

// Key features:
- Event sequence tracking
- Snapshot creation/retrieval
- Chain hash verification
- Generic over aggregate type
```

### Assessment: Well-Architected

| Aspect | Status | Notes |
|--------|--------|-------|
| Generic design | ✅ Excellent | Works with any aggregate |
| Error handling | ✅ Good | Domain-specific errors |
| Hash chaining | ✅ Excellent | SHA-256 chain integrity |
| Snapshots | ✅ Good | Configurable thresholds |
| Testing | ⚠️ Basic | Unit tests only |

### Libification Opportunity

| Component | Locations | Recommendation |
|-----------|-----------|----------------|
| `ContentHash` | hash.rs | Extract to `libs/content-hash` |
| `EventEnvelope` | event.rs | Keep as domain-specific |
| `EventStore` trait | store.rs | Keep as domain-specific |

---

## 2026-03-29 - Graph Architecture Analysis

**Project:** phenotype-infrakit
**Category:** architecture
**Status:** completed
**Priority:** P2

### Summary

Analysis of graph-related patterns in phenoinfrakit ecosystem.

### Graph Patterns Observed

| Pattern | Crate | Purpose |
|---------|-------|---------|
| Evidence Chain | `evidence-ledger` | Audit trail graph |
| Event Chain | `event-sourcing` | Aggregate event chain |
| Policy Graph | `policy-engine` | Rule dependency graph |

### Recommendation: Consider petgraph

```rust
// If graph operations expand, consider petgraph
use petgraph::{Graph, DiGraph, graph::NodeIndex};

// Benefits:
- Battle-tested in Rust ecosystem
- 10M+ weekly downloads
- Optimal algorithms (DFS, BFS, Dijkstra)
- DOT export for visualization
```

### Current: Custom Implementation

The current approach uses custom implementations that are fit-for-purpose but not as feature-rich as petgraph.

---

## 2026-03-29 - Observability Architecture Analysis

**Project:** phenotype-infrakit
**Category:** architecture
**Status:** completed
**Priority:** P1

### Summary

Analysis of observability patterns across phenoinfrakit crates.

### Current State

| Crate | Logging | Tracing | Metrics |
|-------|---------|---------|---------|
| evidence-ledger | ❌ None | ❌ None | ❌ None |
| cache-adapter | ❌ None | ❌ None | ❌ None |
| event-sourcing | ❌ None | ❌ None | ❌ None |
| policy-engine | ❌ None | ❌ None | ❌ None |

### Missing Observability

**Issue:** No structured logging, tracing, or metrics integration.

### Recommended: Add observability

```rust
// Recommended pattern for all crates
use tracing::{info, warn, error};
use metrics::{counter, histogram};

pub struct EventStore<T: Aggregate> {
    #[metric]
    events_appended: Counter,
    #[metric]
    append_duration: Histogram,
}

// Usage
info!(aggregate_id = %id, sequence = %seq, "Event appended");
```

### Integration Path

1. Add `tracing` + `tracing-subscriber` to workspace deps
2. Add `metrics` to workspace deps
3. Instrument critical paths
4. Add `tracing-fmt` layer for development

---

## 2026-03-29 - Error Propagation Architecture

**Project:** phenotype-infrakit
**Category:** architecture
**Status:** completed
**Priority:** P1

### Summary

Analysis of error handling patterns across phenoinfrakit crates.

### Current Error Types

| Crate | Error Type | Variants |
|-------|-----------|----------|
| event-sourcing | `EventStoreError` | 4 variants |
| policy-engine | `PolicyError` | 4+ variants |
| evidence-ledger | `LedgerError` | Not analyzed |
| cache-adapter | `CacheError` | Not analyzed |

### Error Propagation Pattern

```rust
// Current: Domain-specific errors with thiserror
#[derive(Error, Debug)]
pub enum EventStoreError {
    #[error("Event not found: {0}")]
    NotFound(String),
    
    #[error("Concurrent modification detected")]
    ConcurrentModification,
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Chain integrity violation: {0}")]
    ChainIntegrity(String),
}
```

### Libification Opportunity: Extract Error Core

```rust
// Proposed: libs/phenotype-error-core
pub trait DomainError: std::error::Error + Send + Sync + 'static {
    fn error_code(&self) -> &'static str;
    fn severity(&self) -> Severity;
    fn source_error(&self) -> Option<&dyn DomainError>;
}

// Crates would implement this trait
impl DomainError for EventStoreError { ... }
impl DomainError for PolicyError { ... }
```

### Benefits of Error Core

| Benefit | Impact |
|---------|--------|
| Consistent error codes | Better monitoring |
| Severity classification | Better alerting |
| Error chain tracing | Better debugging |
| Structured errors | Better analytics |

---

## 2026-03-29 - Testing Architecture Analysis

**Project:** phenotype-infrakit
**Category:** architecture
**Status:** completed
**Priority:** P2

### Summary

Analysis of testing patterns and opportunities in phenoinfrakit.

### Current Testing State

| Crate | Unit Tests | Integration Tests | Property Tests |
|-------|------------|------------------|----------------|
| event-sourcing | ⚠️ Basic | ❌ None | ❌ None |
| cache-adapter | ⚠️ Basic | ❌ None | ❌ None |
| policy-engine | ⚠️ Basic | ❌ None | ❌ None |
| evidence-ledger | ⚠️ Basic | ❌ None | ❌ None |

### Missing Testing Infrastructure

| Pattern | Status | Recommendation |
|---------|--------|----------------|
| Property-based tests | ❌ None | Add `proptest` |
| Mutation testing | ❌ None | Add `cargo-mutants` |
| Fuzzing | ❌ None | Add `cargo-fuzz` |
| Benchmark tests | ❌ None | Add `criterion` |
| Integration tests | ❌ None | Add test crates |

### Recommended Testing Stack

```toml
# Add to workspace dependencies
proptest = "1.5"
criterion = "0.5"
# cargo-mutants = "24.11"  # Run in CI only
# cargo-fuzz = "0.11"      # If fuzzing needed
```

### Example: Property-Based Event Ordering

```rust
// proptest example for event sourcing
proptest! {
    #[test]
    fn test_event_sequence_always_increments(events in any::<Vec<Event>>()) {
        let mut store = InMemoryEventStore::default();
        for event in events {
            store.append(event.clone());
            let retrieved = store.get_events(&event.id);
            prop_assert!(retrieved.last().sequence >= event.sequence - 1);
        }
    }
}
```

---

## 2026-03-29 - Cross-Crate Communication Patterns

**Project:** phenotype-infrakit
**Category:** architecture
**Status:** completed
**Priority:** P1

### Summary

Analysis of how crates communicate with each other and external systems.

### Communication Patterns Observed

| Pattern | Usage | Assessment |
|---------|-------|------------|
| Direct function calls | Within crate | ✅ Optimal |
| Trait objects | Cross-crate | ✅ Good |
| Message passing | Async operations | ⚠️ Not used |
| Event bus | Decoupled comms | ⚠️ Not used |

### Current: Tight Coupling

```
phenotype-event-sourcing
         │
         └── (no dependencies on other phenoinfrakit crates)
         
policy-engine
         │
         └── (no dependencies on other phenoinfrakit crates)
```

**Issue:** Crates are isolated - no shared communication.

### Alternative: Event Bus Pattern

```rust
// Proposed: Shared event bus
pub trait EventBus: Send + Sync {
    async fn publish(&self, event: DomainEvent) -> Result<(), BusError>;
    async fn subscribe<E: DomainEvent>(&self, handler: EventHandler<E>);
}

// Benefits:
- Loose coupling
- Async-first
- Traceable events
- Extensible
```

### Recommendation

1. **Near-term:** Keep current - crates are independently useful
2. **Long-term:** Add event bus if crates need to coordinate

---

## 2026-03-29 - Configuration Architecture Analysis

**Project:** phenotype-infrakit
**Category:** architecture
**Status:** completed
**Priority:** P2

### Summary

Analysis of configuration patterns across phenoinfrakit crates.

### Current State

| Crate | Config Source | Format | Assessment |
|-------|---------------|--------|------------|
| event-sourcing | Hardcoded | N/A | ⚠️ Limited |
| cache-adapter | Builder pattern | Code | ✅ Flexible |
| policy-engine | Builder pattern | Code | ✅ Flexible |
| evidence-ledger | Hardcoded | N/A | ⚠️ Limited |

### Missing: External Configuration

**Issue:** No support for TOML/YAML/ENV configuration files.

### Recommended: figment Integration

```rust
// Add figment for configuration
use figment::{Figment, providers::{Toml, Env, Format}};

#[derive(Deserialize)]
pub struct EventStoreConfig {
    pub max_events_per_snapshot: u64,
    pub snapshot_interval_hours: u64,
    pub chain_verification: bool,
}

impl Default for EventStoreConfig {
    fn default() -> Self {
        Self {
            max_events_per_snapshot: 100,
            snapshot_interval_hours: 24,
            chain_verification: true,
        }
    }
}

// Load from TOML with ENV override
let config: EventStoreConfig = Figment::new()
    .merge(Toml::file("event-store.toml"))
    .merge(Env::prefixed("EVENT_STORE_"))
    .extract()?;
```

### Benefits

| Benefit | Impact |
|---------|--------|
| TOML/YAML support | Flexibility |
| ENV override | Production config |
| Profiles | Dev/staging/prod |
| Validation | Type-safe config |

---

## 2026-03-29 - Concurrency Architecture Analysis

**Project:** phenotype-infrakit
**Category:** architecture
**Status:** completed
**Priority:** P1

### Summary

Analysis of concurrency patterns and synchronization primitives.

### Current Usage

| Primitive | Crate | Usage | Assessment |
|-----------|-------|-------|------------|
| `RwLock` (std) | event-sourcing | In-memory store | ⚠️ Consider parking_lot |
| `Arc` | Multiple | Shared ownership | ✅ Good |
| `Mutex` | Not used | State protection | 🔲 Consider |
| `Atomic*` | Not used | Simple counters | 🔲 Consider |

### Issue: std::sync::RwLock

```rust
// Current: std::sync::RwLock
use std::sync::RwLock;

pub struct EventStore<T: Aggregate> {
    events: RwLock<Vec<EventEnvelope<T>>>,
}

// Issues:
- Poisoning on panic
- Slower than parking_lot
- Verbose error handling
```

### Recommended: parking_lot

```rust
// Recommended: parking_lot::RwLock
use parking_lot::RwLock;

pub struct EventStore<T: Aggregate> {
    events: RwLock<Vec<EventEnvelope<T>>>,
}

// Benefits:
- No poisoning (always succeeds)
- 25% faster
- Simpler API (no .unwrap())
```

### Note: event-sourcing already uses parking_lot

**Good!** The memory.rs implementation already uses `parking_lot::RwLock`.

### Atomic Operations

Consider `std::sync::atomic` for:
- Sequence number counters
- Simple flags
- Reference counting

---

_Last updated: 2026-03-29_
**Project:** [AgilePlus]
**Category:** architecture
**Status:** in_progress
**Priority:** P1

### Summary

Identified significant architectural split between two hexagonal ecosystems: `phenotype-port-interfaces` and `agileplus-domain/ports`. Both implement similar patterns but with different names and purposes.

### Two Hexagonal Ecosystems

#### Ecosystem 1: phenotype-port-interfaces

```
libs/phenotype-shared/crates/phenotype-port-interfaces/
├── src/outbound/
│   ├── repository.rs (Repository trait, 78 LOC)
│   ├── cache.rs (Cache trait)
│   ├── logger.rs (Logger trait, 101 LOC)
│   ├── event_bus.rs
│   ├── http.rs
│   ├── filesystem.rs
│   └── config.rs
└── src/error.rs (PortError, 51 LOC)
```

#### Ecosystem 2: agileplus-domain

```
crates/agileplus-domain/src/ports/
├── mod.rs
├── observability.rs (ObservabilityPort, 850 LOC)
├── agent.rs (AgentPort)
├── vcs.rs (VcsPort)
├── storage.rs (StoragePort)
└── review.rs (ReviewPort)
```

### Overlap Analysis

| phenotype-port-interfaces | agileplus-domain | Overlap |
|--------------------------|------------------|---------|
| Repository trait | StoragePort | HIGH |
| Logger trait | ObservabilityPort | HIGH |
| Cache trait | (no direct match) | - |
| EventBus trait | (no direct match) | - |

### libs/hexagonal-rs (UNDERSUSED)

```
libs/hexagonal-rs/
├── src/
│   ├── domain/
│   ├── ports/
│   ├── application/
│   └── adapters/
├── Cargo.toml
└── README.md (full hexagonal framework, workspace: false)
```

### Action Items

- [ ] 🟡 HIGH: Audit port interfaces for consolidation
- [ ] 🟡 HIGH: Align phenotype-port-interfaces with hexagonal-rs
- [ ] 🟠 MEDIUM: Move SnapshotStore trait to phenotype-port-interfaces
- [ ] 🟠 MEDIUM: Create unified port trait hierarchy

### Related

- Duplication: `worklogs/DUPLICATION.md`
- Framework: `libs/hexagonal-rs/README.md`

---

## 2026-03-29 - Hexagonal Architecture Review & Library Extraction Plan

**Project:** [AgilePlus]
**Category:** architecture
**Status:** in_progress
**Priority:** P1

### Summary

Conducted comprehensive review of AgilePlus hexagonal architecture compliance and identified library extraction opportunities. Found that AgilePlus is ALREADY hexagonal compliant per ADR-002.

### Findings

| Finding | Status | Recommendation |
|---------|--------|----------------|
| Domain layer isolation | ✅ Compliant | No changes needed |
| Port/Adapter separation | ✅ Compliant | No changes needed |
| Error type centralization | ⚠️ Needs work | Extract to `agileplus-error-core` |
| Config loading centralization | ⚠️ Needs work | Extract to `agileplus-config-core` |
| Health status unification | ⚠️ Needs work | Extract to `agileplus-health-core` |

### Library Extraction Candidates

| Library | Priority | Effort | Files Affected |
|---------|----------|--------|---------------|
| `agileplus-error-core` | P1 | 3 days | 36+ error enums |
| `agileplus-config-core` | P1 | 1 week | 4 config loaders |
| `agileplus-health-core` | P2 | 2 days | 3 health enums |
| `agileplus-test-core` | P3 | 1 week | 4 in-memory stores |

### Tasks Completed

- [x] Reviewed hexagonal architecture compliance
- [x] Identified error type duplications
- [x] Documented config loading patterns
- [x] Created library extraction plan

### Next Steps

- [ ] Create `agileplus-error-core` crate
- [ ] Extract shared error types
- [ ] Update dependent crates
- [ ] Create `agileplus-config-core` crate

### Related

- Plan: `plans/2026-03-29-CROSS_PROJECT_DUPLICATION_PLAN-v1.md`
- ADR: `docs/adr/adr-002-hexagonal-architecture.md`
- Session: `docs/sessions/20260327-plane-fork-pm-substrate/`

---

## 2026-03-29 - Plane.so Fork Decision (G037)

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P0

### Summary

Decision made to fork Plane (plane.so, Apache 2.0) as the shared PM substrate. AgilePlus remains as the custom orchestration/control-plane layer.

### Decision Rationale

- Plane provides complete PM functionality out of the box
- Avoids duplicating PM surface in AgilePlus
- Enables focus on governance and agent orchestration
- Apache 2.0 license permits commercial use

### Architecture Impact

| Component | Role | Change |
|-----------|------|--------|
| AgilePlus | Control plane, governance, agent dispatch | Enhanced |
| Plane.so | PM substrate, issue tracking, cycles | Forked |
| TracerTM | Custom tracking (if needed) | Phase out candidate |

### Work Package Status

| WP | Description | Status |
|----|-------------|--------|
| G037-WP1 | Fork Plane repo into org GitHub | pending |
| G037-WP2 | Define AgilePlus → Plane API boundary adapter | pending |
| G037-WP3 | Migrate or quarantine duplicate PM dashboard code | pending |
| G037-WP4 | Wire existing controls into Plane | pending |
| G037-WP5 | Validate co-existence with Plane | pending |
| G037-WP6 | Archive TracerTM and TheGent from PM surface | pending |

### Related

- Spec: `.agileplus/specs/008-plane-shared-pm-substrate/`
- Session: `docs/sessions/20260327-plane-fork-pm-substrate/`

---

## 2026-03-25 - Cross-Repo Architecture Audit

**Project:** [cross-repo]
**Category:** architecture
**Status:** completed
**Priority:** P1

### Summary

Audit of architecture patterns across AgilePlus, heliosCLI, thegent, and heliosApp. Identified common patterns and divergence points.

### Key Findings

| Pattern | AgilePlus | thegent | heliosCLI | heliosApp |
|---------|-----------|---------|-----------|-----------|
| Language | Rust | Python | Rust | TypeScript |
| Architecture | Hexagonal | Modular | Layered | MVC |
| Config | TOML | YAML | TOML | JSON |
| Error handling | thiserror | thiserror | thiserror | ErrorBoundary |
| Testing | cargo test | pytest | cargo test | Vitest |

### Convergence Recommendations

1. **Error handling**: Adopt shared error-core across Rust projects
2. **Config loading**: Standardize on TOML with env overrides
3. **Testing**: Share test utilities where possible
4. **CLI patterns**: Align heliosCLI patterns with AgilePlus CLI

### Related

- Audit: `plans/2026-03-29-AUDIT_FRAMEWORK-v1.md`
- Comparison: `COMPARISON.md`

---

## 2026-03-24 - MCP Server Architecture Review

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P1

### Summary

Review of MCP server architecture in `agileplus-mcp` and `thegent`. Identified integration opportunities.

### Architecture Comparison

| Aspect | agileplus-mcp | thegent-mcp |
|--------|---------------|-------------|
| Language | Python | Python |
| Backend | gRPC | Direct |
| Tool count | 15+ | 8+ |
| Skill support | Basic | Advanced |
| Streaming | SSE | Not implemented |

### Recommendations

1. Adopt skill-based tool organization from thegent
2. Add streaming support to agileplus-mcp
3. Share common MCP utilities between projects
4. Consider unifying under `phenotype-mcp` core

### Next Steps

- [ ] Create shared `phenotype-mcp-core` library
- [ ] Migrate common utilities
- [ ] Add skill framework to agileplus-mcp

### Related

- MCP Server: `agileplus-mcp/src/agileplus_mcp/server.py`
- TheGent MCP: `thegent/src/thegent/mcp/`

---

## 2026-03-29 - libs/ Directory Architecture Analysis

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P1

### Summary

Comprehensive analysis of `libs/` directory reveals 11 mature libraries with proper hexagonal architecture that are NOT being used by the main workspace. Root cause: edition mismatch (libs: 2021, workspace: 2024).

### libs/ Directory Inventory

| Library | Location | Purpose | Integration Status |
|---------|----------|---------|-------------------|
| config-core | `libs/config-core/` | Config loading framework | 🔴 **UNUSED** — Zero imports |
| logger | `libs/logger/` | Structured logging | 🔴 **UNUSED** — Zero imports |
| tracing-lib | `libs/tracing/` | Distributed tracing | 🔴 **UNUSED** — Zero imports |
| metrics | `libs/metrics/` | Metrics collection | 🔴 **UNUSED** — Zero imports |
| hexagonal-rs | `libs/hexagonal-rs/` | Ports & Adapters framework | 🔴 **UNUSED** — Zero imports |
| hexkit | `libs/hexkit/` | HTTP/Persistence adapters | 🔴 **UNUSED** — Zero imports |
| cipher | `libs/cipher/` | Encryption utilities | 🟡 Partially used |
| gauge | `libs/gauge/` | Benchmarking | 🟡 Partially used |
| nexus | `libs/nexus/` | Service discovery | 🟡 Partially used |
| xdd-lib-rs | `libs/xdd-lib-rs/` | Data transformation | 🟡 Partially used |
| cli-framework | `libs/cli-framework/` | Command parsing | 🟡 Partially used |

### Root Cause Analysis

```
┌─────────────────────────────────────────────────────┐
│ libs/                          │ Main Workspace     │
├─────────────────────────────────────────────────────┤
│ edition = "2021"               │ edition = "2024"  │
│ workspace = false              │ workspace = true   │
│ Standalone crates              │ Unified workspace  │
└─────────────────────────────────────────────────────┘
```

### Evidence — hexagonal-rs Has Exact Patterns Needed

```rust
// libs/hexagonal-rs/src/ports/repository.rs:12-23
#[async_trait]
pub trait Repository<E: Entity> {
    async fn find(&self, id: &E::Id) -> Result<Option<E>, RepositoryError>;
    async fn save(&self, entity: &E) -> Result<(), RepositoryError>;
    async fn delete(&self, id: &E::Id) -> Result<(), RepositoryError>;
}
```

But agileplus crates define their own duplicated versions:

| Duplicated Trait | Location | LOC |
|-----------------|----------|-----|
| EventBus | `agileplus-nats/src/bus.rs:36-60` | ~25 |
| SyncMappingStore | `agileplus-sync/src/store.rs:16-41` | ~26 |
| EventStore | `agileplus-events/src/store.rs:21-53` | ~33 |
| GraphBackend | `agileplus-graph/src/store.rs:22-27` | ~6 |
| CacheStore | `agileplus-cache/src/store.rs:21-38` | ~18 |

### Evidence — config-core Has Complete Config Loading

```rust
// libs/config-core/src/lib.rs (existing implementation)
pub struct ConfigLoader<T: DeserializeOwned> {
    path: PathBuf,
    env_prefix: String,
    validator: Option<Box<dyn Fn(&T) -> Result<(), ConfigError>>>,
}
```

But crates define their own loaders:

| Duplicated Loader | Location | Format |
|------------------|----------|--------|
| TOML loader | `agileplus-domain/src/config/loader.rs:24-84` | TOML |
| YAML loader | `agileplus-telemetry/src/config.rs:126-201` | YAML |
| JSON loader | `vibe-kanban/backend/src/models/config.rs:267-374` | JSON |

### Action Items

- [ ] 🔴 **CRITICAL** Investigate edition migration path (2021 → 2024) for libs/
- [ ] 🔴 **CRITICAL** Integrate libs/hexagonal-rs to replace duplicated repository traits
- [ ] 🔴 **CRITICAL** Integrate libs/config-core to replace config loaders
- [ ] 🟡 **HIGH** Audit unused libs for deletion candidates
- [ ] 🟠 **MEDIUM** Create migration guide for adding new hexagonal modules
- [ ] 🟢 **LOW** Document libs/ conventions in ARCHITECTURE.md

### Related

- Duplication: `worklogs/DUPLICATION.md`
- Dependencies: `worklogs/DEPENDENCIES.md`

---

## 2026-03-29 - heliosCLI Architecture Patterns

**Project:** [heliosCLI]
**Category:** architecture
**Status:** completed
**Priority:** P2

### Summary

Reviewed heliosCLI architecture patterns for consistency with AgilePlus.

### Pattern Comparison

| Pattern | heliosCLI | AgilePlus | Alignment |
|---------|-----------|-----------|-----------|
| Error handling | thiserror | thiserror | ✅ Match |
| CLI parsing | clap | clap | ✅ Match |
| Async runtime | tokio | tokio | ✅ Match |
| Config format | TOML | TOML | ✅ Match |

### Recommendations

1. Consider adopting `phenotype-error` when forked
2. Standardize on `command-group` for process management
3. Add `indicatif` for progress feedback
4. Document architecture decisions as ADRs

### Next Steps

- [ ] Create ADRs for key architectural decisions
- [ ] Evaluate fork candidates for shared libraries
- [ ] Add progress feedback with indicatif

---

## 2026-03-29 - Wave 93: Canonical `repos/` layout vs docs drift + scan boundaries

**Project:** [phenotype-infrakit]
**Category:** architecture
**Status:** in_progress
**Priority:** P0

### Summary

Reconciled **documented** workspace layout with **actual** root `Cargo.toml` and filesystem. Established rules for **what counts** in duplication and dependency audits when vendored trees exist.

### Workspace truth (root `Cargo.toml`)

| Fact | Detail |
|------|--------|
| Members | `phenotype-contracts`, `phenotype-event-sourcing`, `phenotype-cache-adapter`, `phenotype-policy-engine`, `phenotype-state-machine` |
| `evidence-ledger` | **Not** a workspace member in current root manifest — treat earlier diagrams listing it as **stale** until re-added or doc updated |
| Edition | Workspace `edition.workspace` still **2021** in `[workspace.package]` — align docs that claim 2024 until migration lands |

### Nested package roots (structural debt)

As of scan date, **four** crates still contain `crates/<name>/<name>/` alongside `crates/<name>/src/`:

- `phenotype-policy-engine`
- `phenotype-state-machine`
- `phenotype-cache-adapter`
- `phenotype-contracts`

`phenotype-event-sourcing` has been **flattened** to `Cargo.toml` + `src/` only (inner duplicate directory **removed**). Any prior `diff -rq` showing differing twins is **historical** for that crate.

**Rule:** Workspace `[members]` must point at exactly one package root per crate; nested same-name folders are **migration artifacts**, not optional adapters.

### Vendored trees (exclude from “crate” metrics)

| Path | Why exclude from default audits |
|------|----------------------------------|
| `phenotype-shared-wtrees/**` | Full alternate checkouts |
| `thegent-work/**` | Embedded thegent workspace |
| `heliosCLI-wtrees/**` | Embedded CLI / codex-rs |

**Agent / CI convention:** When running `rg`, `jscpd`, or LOC dashboards, pass path filters **or** document inclusion explicitly. Otherwise duplication counts are **not comparable** across waves.

### Port layering (enforcement target)

| Layer | May depend on | Must not depend on |
|-------|----------------|--------------------|
| Domain / contracts | `serde`, `thiserror`, clock abstractions | `axum`, `redis`, `sqlx`, SDK clients |
| Application / use-cases | Domain, port traits | Concrete adapter crates |
| Adapters | Ports, IO crates, OTel | Sibling adapter-specific types leaking into domain |

### Action items (Wave 93)

- [ ] Update earlier ARCHITECTURE diagrams: remove `evidence-ledger` from member list **or** restore crate to workspace
- [ ] Collapse remaining four nested `crates/<pkg>/<pkg>/` trees (PR per crate)
- [ ] Add `docs/AGENTS.md` or `deny.toml` note: default audit scope = `crates/` + `tests/` only
- [ ] Edition 2024 migration: single tracking PR once `libs/` and workspace agree

---

_Last updated: 2026-03-29 (Wave 93)_

## 2026-03-30 - Crate Decomposition Opportunities

**Project:** [cross-repo]
**Category:** architecture
**Status:** identified
**Priority:** P1

### Summary

Analysis of oversized crates that should be decomposed into smaller, focused crates.

### Oversized Crate Analysis

#### 1. `agileplus-domain` (2,400+ LOC)

| Module | LOC | Purpose | Recommendation |
|--------|-----|---------|----------------|
| `aggregate.rs` | 450 | Aggregate root | Extract to `agileplus-aggregate` |
| `repository.rs` | 380 | Repository pattern | Move to `agileplus-repository` |
| `commands.rs` | 320 | Command handling | Move to `agileplus-commands` |
| `events.rs` | 280 | Domain events | Move to `agileplus-events-core` |
| `value_objects.rs` | 250 | Value objects | Move to `agileplus-vo` |
| `services.rs` | 200 | Domain services | Keep (small enough) |
| `mod.rs` | 520 | Module orchestration | Refactor |

**Proposed Decomposition:**

```
agileplus-domain/
├── agileplus-domain-core/     # Keep minimal (~200 LOC)
│   ├── lib.rs                 # Re-exports
│   └── domain_error.rs        # Core errors
├── agileplus-aggregate/       # NEW (~450 LOC)
├── agileplus-repository/      # NEW (~380 LOC)
├── agileplus-commands/        # NEW (~320 LOC)
└── agileplus-value-objects/   # NEW (~250 LOC)
```

#### 2. `agileplus-api` (1,800+ LOC)

| Module | LOC | Recommendation |
|--------|-----|----------------|
| `handlers.rs` | 600 | Split by resource |
| `middleware.rs` | 350 | Extract to `agileplus-middleware` |
| `responses.rs` | 250 | Extract to `agileplus-api-types` |
| `auth.rs` | 300 | Extract to `agileplus-auth` |

#### 3. `phenotype-event-sourcing` (1,600+ LOC)

| Module | LOC | Recommendation |
|--------|-----|----------------|
| `store.rs` | 400 | Keep (core) |
| `memory.rs` | 450 | Extract to `phenotype-memory-store` |
| `snapshot.rs` | 350 | Keep (core) |
| `projection.rs` | 400 | Extract to `phenotype-projections` |

### Decomposition Benefits

| Benefit | Impact |
|---------|--------|
| Faster compilation | 30-50% build time reduction |
| Better test isolation | Each crate has clear contracts |
| Improved caching | Crate-level incremental compilation |
| Clearer ownership | Teams can own specific crates |
| Easier onboarding | Smaller codebase to understand |

### Decomposition Anti-Patterns

**DO NOT decompose:**
- Single-purpose crates < 500 LOC
- Tightly coupled modules
- Crates that change together
- Crates with circular dependencies

**DO decompose:**
- Crates > 2,000 LOC
- Crates with multiple concerns
- Crates with multiple teams
- Crates with different release cadences

### Implementation Order

1. **Phase 1:** Identify boundaries (current)
2. **Phase 2:** Create stub crates with `pub use`
3. **Phase 3:** Move implementation file by file
4. **Phase 4:** Update imports across ecosystem
5. **Phase 5:** Remove re-exports

### Action Items

- [ ] Create `agileplus-aggregate` crate
- [ ] Create `agileplus-repository` crate
- [ ] Create `agileplus-api-types` crate
- [ ] Migrate code incrementally
- [ ] Update all dependent crates

---

## 2026-03-30 - Macro-Based Code Generation

**Project:** [cross-repo]
**Category:** architecture
**Status:** identified
**Priority:** P1

### Summary

Systematic analysis of repetitive code patterns that can be eliminated with derive macros or procedural macros.

### Pattern 1: ID Types (Newtype Wrappers)

**Current State:**
```rust
// 30+ newtype wrappers across codebase
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UserId(String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProjectId(String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TaskId(String);

// ... 27 more
```

**Problem:** 10-15 LOC per type × 30 types = 300-450 LOC

**Solution: `derive_more` or custom macro**
```rust
// Option 1: derive_more
use derive_more::Display;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Display)]
#[display("{_0}")]
pub struct UserId(String);

// Option 2: Custom procedural macro
#[phenotype_id]
pub struct UserId;

#[phenotype_id]
pub struct ProjectId;

#[phenotype_id]
pub struct TaskId;
```

**Savings:** ~400 LOC

### Pattern 2: Port Traits (Repetitive Async Traits)

**Current State:**
```rust
#[async_trait]
pub trait UserRepository {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, RepoError>;
    async fn find_all(&self) -> Result<Vec<User>, RepoError>;
    async fn save(&self, user: User) -> Result<(), RepoError>;
    async fn delete(&self, id: &UserId) -> Result<(), RepoError>;
}

#[async_trait]
pub trait ProjectRepository {
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepoError>;
    async fn find_all(&self) -> Result<Vec<Project>, RepoError>;
    async fn save(&self, project: Project) -> Result<(), RepoError>;
    async fn delete(&self, id: &ProjectId) -> Result<(), RepoError>;
}
```

**Solution: Generic CRUD macro**
```rust
// phenotype-macros/src/repository.rs

#[proc_macro_attribute]
pub fn phenotype_repository(
    attr: TokenStream,
    item: TokenStream
) -> TokenStream {
    // Generate CRUD methods from entity name
}

// Usage:
#[phenotype_repository(entity = "User")]
trait UserRepository {
    // Only custom methods here
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
}

// Generates: find_by_id, find_all, save, delete
```

**Savings:** ~200 LOC per repository × 10 repositories = 2,000 LOC

### Pattern 3: Event Sourcing Boilerplate

**Current State:**
```rust
impl Aggregate for TaskAggregate {
    type Id = TaskId;
    type Event = TaskEvent;
    
    fn aggregate_id(&self) -> &Self::Id { &self.id }
    
    fn apply(&mut self, event: Self::Event) {
        match event {
            TaskEvent::Created(e) => self.apply_created(e),
            TaskEvent::Updated(e) => self.apply_updated(e),
            TaskEvent::Deleted => self.apply_deleted(),
        }
    }
}
```

**Solution: Derive macro for Aggregate**
```rust
#[phenotype_aggregate]
pub struct TaskAggregate {
    id: TaskId,
    name: String,
    status: TaskStatus,
}

#[phenotype_aggregate_event]
enum TaskEvent {
    Created { name: String },
    Updated { name: String, status: TaskStatus },
    Deleted,
}
```

**Savings:** ~100 LOC per aggregate × 15 aggregates = 1,500 LOC

### Pattern 4: Command Handlers

**Current State:**
```rust
impl CommandHandler for TaskCommands {
    type Context = ApplicationContext;
    type Error = CommandError;
    
    async fn handle_create(
        ctx: &Self::Context,
        cmd: CreateTask,
    ) -> Result<TaskId, Self::Error> {
        let task = Task::new(cmd.name, cmd.project_id)?;
        ctx.repository.save(task).await?;
        Ok(task.id)
    }
    
    async fn handle_update(
        ctx: &Self::Context,
        cmd: UpdateTask,
    ) -> Result<(), Self::Error> {
        let mut task = ctx.repository.find_by_id(&cmd.id).await?
            .ok_or(CommandError::NotFound)?;
        task.update(cmd)?;
        ctx.repository.save(task).await?;
        Ok(())
    }
    // ... 8 more handlers
}
```

**Solution: Macro to generate boilerplate**
```rust
#[phenotype_command_handler]
impl TaskCommands {
    #[command]
    async fn create(&self, ctx: &Context, cmd: CreateTask) -> Result<TaskId> {
        let task = Task::new(cmd.name, cmd.project_id)?;
        ctx.repository.save(task).await?;
        Ok(task.id)
    }
    
    #[command]
    async fn update(&self, ctx: &Context, cmd: UpdateTask) -> Result<()> {
        let mut task = ctx.repository.find_by_id(&cmd.id).await?
            .ok_or(CommandError::NotFound)?;
        task.update(cmd)?;
        ctx.repository.save(task).await
    }
}
```

### Macro Crate Structure

```text
phenotype-macros/
├── Cargo.toml
└── src/
    ├── lib.rs              # Re-exports
    ├── id.rs               # #[phenotype_id] macro
    ├── repository.rs       # #[phenotype_repository] macro
    ├── aggregate.rs        # #[phenotype_aggregate] macro
    └── command.rs          # #[phenotype_command_handler] macro
```

### Total Macro Savings

| Pattern | LOC Saved | Priority |
|---------|-----------|----------|
| ID types | 400 | P1 |
| Repository traits | 2,000 | P0 |
| Aggregate | 1,500 | P1 |
| Commands | 1,000 | P2 |
| **Total** | **4,900** | |

---

## 2026-03-30 - Shared Derive Patterns

**Project:** [cross-repo]
**Category:** architecture
**Status:** identified
**Priority:** P2

### Summary

Canonical derive macro patterns that should be shared across all crates.

### Recommended Shared Derives

#### 1. `#[phenotype_entity]`

```rust
// Automatic: Clone + Debug + PartialEq + Serialize + Deserialize
#[phenotype_entity]
pub struct User {
    id: UserId,
    email: String,
    created_at: DateTime<Utc>,
}

// Equivalent to:
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
```

**Use case:** Domain entities that are stored and compared

#### 2. `#[phenotype_value_object]`

```rust
// Automatic: Clone + PartialEq + Hash + (De)Serialize + Display
#[phenotype_value_object]
pub struct Email(String);

#[phenotype_value_object]
pub struct Amount {
    value: Decimal,
    currency: Currency,
}

// Equivalent to:
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display)]
```

**Use case:** Value objects with value semantics

#### 3. `#[phenotype_command]`

```rust
// Automatic: Clone + Debug + Serialize + Deserialize + Validate
#[phenotype_command]
pub struct CreateTask {
    pub name: String,
    pub project_id: ProjectId,
    pub assignee_id: Option<UserId>,
}

// Validates: name is not empty, project_id is valid UUID
```

**Use case:** Command DTOs with built-in validation

#### 4. `#[phenotype_event]`

```rust
// Automatic: Clone + Debug + Serialize + Deserialize + EventMetadata
#[phenotype_event]
pub enum TaskEvent {
    Created { name: String },
    Updated { changes: Vec<Change> },
    Deleted,
}

// Adds: Event::metadata(), Event::timestamp()
```

**Use case:** Domain events with automatic metadata

### Derive Macro Dependencies

```rust
// phenotype-derive-macros/Cargo.toml
[dependencies]
syn = "2"
quote = "1"
proc-macro2 = "1"
 darling = "0.20"
 
[dependencies.proc-macro-yoke]
version = "0.7"
features = ["derive"]
```

### Shared Configuration

```rust
// phenotype-derive-macros/src/config.rs

/// Global configuration for Phenotype derive macros
pub struct PhenotypeDeriveConfig {
    /// Enable validation in #[phenotype_command]
    pub validate_commands: bool,
    /// Add timestamps to #[phenotype_event]
    pub auto_timestamp: bool,
    /// Use decimal for money types
    pub decimal_precision: u8,
}

impl Default for PhenotypeDeriveConfig {
    fn default() -> Self {
        Self {
            validate_commands: true,
            auto_timestamp: true,
            decimal_precision: 2,
        }
    }
}
```

### Migration Path

1. **Phase 1:** Create `phenotype-derive-macros` crate
2. **Phase 2:** Implement each derive macro
3. **Phase 3:** Add to workspace
4. **Phase 4:** Migrate crates one at a time
5. **Phase 5:** Remove old manual derives

### Action Items

- [ ] Create `phenotype-derive-macros` crate
- [ ] Implement `#[phenotype_entity]`
- [ ] Implement `#[phenotype_value_object]`
- [ ] Implement `#[phenotype_command]`
- [ ] Implement `#[phenotype_event]`
- [ ] Migrate domain crates

---

## 2026-03-30 - Generic Container Patterns

**Project:** [cross-repo]
**Category:** architecture
**Status:** identified
**Priority:** P2

### Summary

Analysis of container types that could be genericized for reuse.

### Pattern 1: Result Type Aliases

**Current:** Each crate defines its own result type
```rust
// agileplus-domain/src/lib.rs
pub type Result<T> = std::result::Result<T, DomainError>;

// agileplus-api/src/lib.rs
pub type Result<T> = std::result::Result<T, ApiError>;

// phenotype-event-sourcing/src/lib.rs
pub type Result<T> = std::result::Result<T, EventSourcingError>;
```

**Solution:** Shared result type with context
```rust
// phenotype-core/src/result.rs

pub type Result<T, E = Infallible> = std::result::Result<T, Error<E>>;

pub struct Error<E> {
    pub context: ErrorContext,
    pub error: E,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl<E> From<E> for Error<E> 
where E: std::error::Error {
    fn from(error: E) -> Self {
        Self {
            context: ErrorContext::default(),
            error,
            source: None,
        }
    }
}
```

**Savings:** ~50 LOC (one-time)

### Pattern 2: Collection Type Aliases

**Current:** Repetitive collection declarations
```rust
// Many files have:
let users: Vec<User> = vec![];
let ids: Vec<UserId> = vec![];
let names: Vec<String> = vec![];

// Or:
let mut map: HashMap<ProjectId, Project> = HashMap::new();
let mut set: HashSet<UserId> = HashSet::new();
```

**Solution:** Typed collection aliases
```rust
// phenotype-collections/src/lib.rs

pub type EntityVec<E> = Vec<E>;
pub type EntityMap<K, V> = HashMap<K, V>;
pub type EntitySet<T> = HashSet<T>;

pub type UserMap = EntityMap<UserId, User>;
pub type ProjectMap = EntityMap<ProjectId, Project>;
pub type IdSet<T> = EntitySet<Id<T>>;
```

### Pattern 3: Async Stream Helpers

**Current:** Repetitive stream transformations
```rust
// Across many files:
let stream = futures::stream::iter(items)
    .then(|item| process(item))
    .buffer_unordered(10)
    .filter_map(|result| async { result.ok() });
```

**Solution:** Reusable stream combinators
```rust
// phenotype-streams/src/lib.rs

pub trait StreamExt {
    fn process_parallel<P, Fut>(self, parallelism: usize, processor: P) -> Self
    where
        P: FnMut(Item) -> Fut,
        Fut: Future<Output = Result<Item>>;
        
    fn log_errors(self, logger: Logger) -> Self;
    
    fn with_timeout(self, duration: Duration) -> Self;
}

pub fn process_parallel<I, Item, P, Fut>(
    items: I,
    parallelism: usize,
    processor: P
) -> impl Stream<Item = Result<Item>>
where
    I: IntoIterator<Item = Item>,
    P: FnMut(Item) -> Fut,
    Fut: Future<Output = Result<Item>>,
{
    futures::stream::iter(items)
        .map(Ok)
        .try_buffer_unordered(parallelism)
}
```

### Pattern 4: Pagination

**Current:** Every service implements pagination
```rust
async fn list_users(
    page: u32,
    page_size: u32,
) -> Result<PaginatedResponse<User>> {
    let offset = (page - 1) * page_size;
    let users = repo.find_with_limit(page_size, offset).await?;
    let total = repo.count().await?;
    Ok(PaginatedResponse {
        data: users,
        page,
        page_size,
        total,
        total_pages: (total + page_size - 1) / page_size,
    })
}
```

**Solution:** Generic pagination helper
```rust
// phenotype-pagination/src/lib.rs

pub struct Paginated<T> {
    pub data: Vec<T>,
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
}

impl<T> Paginated<T> {
    pub fn new(data: Vec<T>, page: u32, page_size: u32, total: u64) -> Self {
        Self { data, page, page_size, total }
    }
    
    pub fn total_pages(&self) -> u32 {
        (self.total + self.page_size - 1) / self.page_size
    }
    
    pub fn has_next(&self) -> bool {
        self.page < self.total_pages()
    }
    
    pub fn has_prev(&self) -> bool {
        self.page > 1
    }
    
    pub async fn from_slice<F, Fut>(
        items: Vec<T>,
        page: u32,
        page_size: u32,
        counter: F,
    ) -> Result<Self>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<u64>>,
    {
        let total = counter().await?;
        Ok(Self::new(items, page, page_size, total))
    }
}
```

**Savings:** ~80 LOC per service

---

_Last updated: 2026-03-30_

## 2026-03-29 - Message Queue & Event Bus Architecture

**Project:** phenotype-infrakit
**Category:** architecture
**Status:** completed
**Priority:** P1

### Summary

Analysis of message queue and event bus patterns across phenoinfrakit and related crates.

### Current State

| Crate | Pattern | Backend | Quality |
|-------|---------|---------|---------|
| `agileplus-nats` | EventBus | NATS JetStream | High |
| `agileplus-events` | EventStore | NATS | Medium |
| `phenotype-event-sourcing` | Aggregate | Memory | High |
| `phenotype-cache-adapter` | Cache | DashMap/Moka | High |

### Missing: Cross-Crate Event Bus

**Issue:** No shared event bus for inter-crate communication.

### Recommended: Shared Event Bus

```rust
// Proposed: phenotype-event-bus crate
pub trait EventBus: Send + Sync {
    async fn publish(&self, topic: &str, event: &[u8]) -> Result<(), BusError>;
    async fn subscribe(&self, topic: &str, handler: Box<dyn EventHandler>) -> Result<Subscription>;
}

pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &[u8]) -> Result<(), HandlerError>;
}
```

### Integration Path

1. Create `phenotype-event-bus` crate
2. Implement NATS adapter
3. Add Redis adapter (via libs/redis-adapter)
4. Migrate existing event patterns

---

## 2026-03-29 - gRPC & API Architecture

**Project:** phenotype-infrakit
**Category:** architecture
**Status:** completed
**Priority:** P2

### Summary

Analysis of gRPC and API patterns for phenoinfrakit.

### Current State

| Crate | API Style | Assessment |
|-------|-----------|------------|
| `phenotype-event-sourcing` | No API | Needs HTTP/gRPC layer |
| `phenotype-policy-engine` | No API | Needs HTTP/gRPC layer |
| `evidence-ledger` | No API | Needs HTTP/gRPC layer |

### Recommended: gRPC with tonic

```toml
# For gRPC APIs
tonic = "0.15"
prost = "0.13"
tower = "0.5"
```

### Service Definition

```protobuf
// evidence-ledger.proto
service EvidenceLedger {
    rpc AppendEntry(AppendRequest) returns (AppendResponse);
    rpc VerifyChain(VerifyRequest) returns (VerifyResponse);
    rpc QueryEntries(QueryRequest) returns (stream Entry);
}
```

### API Layer Architecture

```
┌─────────────────────────────────────────────────┐
│                    API Layer                     │
├─────────────────────────────────────────────────┤
│  HTTP (axum)     │    gRPC (tonic)             │
├─────────────────────────────────────────────────┤
│           Service Implementations                 │
├─────────────────────────────────────────────────┤
│  Event Sourcing  │  Policy Engine  │  Ledger   │
└─────────────────────────────────────────────────┘
```

---

## 2026-03-29 - Deployment & Packaging Architecture

**Project:** phenotype-infrakit
**Category:** architecture
**Status:** completed
**Priority:** P2

### Summary

Analysis of deployment and packaging options for phenoinfrakit crates.

### Deployment Options

| Option | Use Case | Assessment |
|--------|----------|------------|
| Binary | Standalone service | ✅ Recommended |
| Library | Embedded in app | ✅ Supported |
| Docker | Containerized | 🔲 Add Dockerfile |
| Lambda | Serverless | 🔲 Consider |
| WASM | Browser/Edge | 🔲 Future |

### Recommended: Binary + Docker

```dockerfile
# evidence-ledger.Dockerfile
FROM rust:1.85-slim as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/evidence-ledger ./crates/evidence-ledger
RUN cargo build --release -p evidence-ledger

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/evidence-ledger /usr/local/bin/
ENTRYPOINT ["evidence-ledger"]
```

### Binary Distribution

```toml
# Cargo.toml
[[bin]]
name = "evidence-ledger"
path = "src/main.rs"
required-features = ["server"]
```

---

## 2026-03-29 - API Versioning & Evolution

**Project:** phenotype-infrakit
**Category:** architecture
**Status:** completed
**Priority:** P2

### Summary

Analysis of API versioning strategies for phenoinfrakit services.

### Versioning Strategies

| Strategy | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| URL path (`/v1/`) | Simple | URL pollution | ✅ Use for HTTP |
| Header (`Accept: v2`) | Clean URLs | Complex | For gRPC |
| Query param | Simple | Caching issues | ❌ Avoid |

### Recommended: Hybrid

```rust
// HTTP: URL path versioning
#[axum::routing::get("/v1/evidence/chain")]
async fn get_chain_v1() {}

#[axum::routing::get("/v2/evidence/chain")]
async fn get_chain_v2() {}

// gRPC: Package versioning
// package evidence.v2;
```

### Breaking Change Policy

1. Add new fields (backward compatible)
2. Add new endpoints (backward compatible)
3. Remove fields (deprecated first)
4. Remove endpoints (after 2 minor versions)

---

## 2026-03-29 - Security Architecture

**Project:** phenotype-infrakit
**Category:** architecture
**Status:** completed
**Priority:** P1

### Summary

Security architecture analysis for phenoinfrakit.

### Current State

| Area | Status | Notes |
|------|--------|-------|
| Input validation | ⚠️ Basic | serde only |
| Secret management | ❌ None | No keychain |
| Encryption | ⚠️ Basic | SHA-256 only |
| Access control | ❌ None | No RBAC |
| Audit logging | ✅ Evidence ledger | Built-in |

### Missing Security Features

| Feature | Priority | Implementation |
|---------|----------|----------------|
| Input validation | 🟠 HIGH | Add validator crate |
| Secret storage | 🟠 HIGH | Use OS keychain |
| TLS | 🟠 HIGH | Add rustls |
| RBAC | 🟡 MEDIUM | Add casbin |
| Rate limiting | 🟡 MEDIUM | Add tower-limit |

### Recommended: Security Stack

```toml
# Add for security
validator = "0.16"
keyring = "3.0"
rustls = "0.23"
casbin = "3.0"
tower = { version = "0.5", features = ["limit"] }
```

---

_Last updated: 2026-03-29_

---

## 2026-03-29 - Round 13: Edge-First Deployment Architecture

**Project:** [cross-repo]
**Category:** architecture
**Status:** proposed
**Priority:** P2

### Summary
Architecture for deploying Phenotype agents to edge locations (CDN nodes, local branch offices) to ensure low-latency response for high-frequency user interactions.

### Edge Node Components
1. **Lightweight Runtime:** WASM or Firecracker microVMs.
2. **Local State:** SQLite or Sled for transient data.
3. **Upstream Sync:** NATS JetStream for asynchronous state propagation to the "Home" region.

### Connectivity Topography
- **Edge-to-Cloud:** Persistent gRPC stream for real-time control.
- **Edge-to-Edge:** Peer-to-peer gossip (future) for local discovery.

---

## 2026-03-29 - Round 13: Collaborative State Architecture (CRDT)

**Project:** [cross-repo]
**Category:** architecture
**Status:** proposed
**Priority:** P3

### Summary
Architecture for multi-agent and multi-user collaborative editing of shared state (e.g., project boards, policy docs) using Conflict-free Replicated Data Types (CRDTs).

### Implementation Layers
1. **Model Layer:** `Automerge` or `Yjs` structures.
2. **Persistence Layer:** Storing CRDT change logs in the `evidence-ledger`.
3. **Network Layer:** WebSockets via the `phenotype-gateway` for real-time update broadcasts.

### Key Benefits
- **No Merge Conflicts:** Automatic deterministic merging of state.
- **Offline Support:** Agents can continue working during network outages and sync later.

_Last updated: 2026-03-29 (Round 13)_
