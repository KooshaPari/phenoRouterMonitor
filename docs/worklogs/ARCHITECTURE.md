# Architecture Worklogs

**Category:** ARCHITECTURE | **Updated:** 2026-03-29

---

## 2026-03-29 - Port/Trait Architecture Split Analysis

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

---

## 2026-03-29 - ARCHITECTURE DECOMPOSITION (Non-Heliso)

**Project:** [cross-repo]
**Category:** architecture
**Status:** completed
**Priority:** P0

### Architecture Overview

```
phenotype/
├── crates/                    # Core libraries (73,444 LOC)
│   ├── phenotype-contracts/  # Port/trait definitions
│   ├── phenotype-event-sourcing/
│   ├── phenotype-policy-engine/
│   └── ...
├── libs/                      # Shared utilities (1,470 LOC)
│   ├── cli-framework/
│   ├── logger/
│   └── ...
├── repos/worktrees/
│   ├── AgilePlus/            # Main application (80,191 LOC)
│   ├── consolidate-libraries/ # ORPHANED - to be deleted
│   └── expand-test-coverage/ # ORPHANED - to be deleted
└── platforms/
    ├── thegent/              # Agent runtime
    ├── heliosCLI/            # CLI tool
    └── phenotype-infrakit/   # Infrastructure kit
```

---

### 1. Crate Architecture Issues

#### Problem: Monolithic Crates

| Crate | LOC | Issue | Solution |
|-------|-----|-------|----------|
| `agileplus-api` | 6,739 | All-in-one | Split by route |
| `agileplus-cli` | 8,884 | All-in-one | Split by command |
| `agileplus-dashboard` | 5,669 | Mixed concerns | Separate UI/backend |

#### Solution: Layered Architecture

```
phenotype-api/
├── phenotype-api-core/       # 2,000 LOC - Shared logic
│   ├── src/
│   │   ├── error.rs         # Error types
│   │   ├── config.rs        # Configuration
│   │   └── auth.rs          # Auth middleware
│   └── Cargo.toml
├── phenotype-api-routes/     # 2,500 LOC - Route handlers
│   ├── src/
│   │   ├── projects.rs      # Project routes
│   │   ├── tasks.rs         # Task routes
│   │   └── users.rs         # User routes
│   └── Cargo.toml
├── phenotype-api-middleware/  # 1,000 LOC - Middleware
│   └── src/
│       ├── tracing.rs
│       ├── rate_limit.rs
│       └── cors.rs
└── phenotype-api-models/     # 1,239 LOC - DTOs
    └── src/
        ├── project.rs
        └── task.rs
```

---

### 2. Port/Trait Consolidation

#### Current: Scattered Ports

| Location | Port Trait | Status |
|----------|-----------|--------|
| `phenotype-contracts` | `Repository` | ✅ Defined |
| `phenotype-contracts` | `EventStore` | ✅ Defined |
| `agileplus-domain` | `MetricsHook` | ❌ Duplicate |
| `agileplus-domain` | `Logger` | ❌ Duplicate |
| `agileplus-domain` | `ConfigPort` | ❌ Duplicate |

#### Solution: Unified Port Traits

```rust
// phenotype-contracts/src/ports.rs

// === Storage Ports ===
pub trait Repository<E: Entity>: Send + Sync {
    async fn save(&self, entity: E) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: &E::Id) -> Result<Option<E>, RepositoryError>;
    async fn delete(&self, id: &E::Id) -> Result<(), RepositoryError>;
}

pub trait EventStore: Send + Sync {
    async fn append(&self, events: Vec<Event>) -> Result<(), EventStoreError>;
    async fn get_events(&self, aggregate_id: &str) -> Result<Vec<Event>, EventStoreError>;
}

// === Observability Ports ===
pub trait MetricsPort: Send + Sync {
    fn increment(&self, name: &str, value: f64);
    fn gauge(&self, name: &str, value: f64);
    fn histogram(&self, name: &str, value: f64);
}

pub trait TracingPort: Send + Sync {
    fn span(&self, name: &str) -> Span;
    fn event(&self, name: &str, attrs: SpanAttributes);
}

// === Configuration Ports ===
pub trait ConfigPort: Send + Sync {
    fn get(&self, key: &str) -> Result<ConfigValue, ConfigError>;
    fn get_optional(&self, key: &str) -> Result<Option<ConfigValue>, ConfigError>;
}
```

---

### 3. Event Sourcing Architecture

#### Current: Simple Implementation

```
phenotype-event-sourcing/
├── src/
│   ├── lib.rs
│   ├── event.rs
│   ├── aggregate.rs
│   └── hash.rs
└── tests/
    └── basic.rs
```

#### Proposed: Production-Ready Architecture

```
phenotype-event-sourcing/
├── phenotype-event-sourcing-core/      # Core abstractions
│   ├── src/
│   │   ├── event.rs                   # Event traits
│   │   ├── aggregate.rs               # Aggregate root
│   │   ├── snapshot.rs               # Snapshotting
│   │   └── projector.rs              # Projection traits
│   └── Cargo.toml
├── phenotype-event-sourcing-hash/       # Hash chain
│   ├── src/
│   │   ├── blake3_chain.rs           # blake3 implementation
│   │   ├── sha256_chain.rs           # SHA-256 legacy
│   │   └── chain_builder.rs          # Builder pattern
│   └── Cargo.toml
├── phenotype-event-sourcing-store/     # Event storage
│   ├── phenotype-event-sourcing-sqlite/ # SQLite adapter
│   ├── phenotype-event-sourcing-sqlx/   # SQLx adapter
│   └── phenotype-event-sourcing-memory/ # In-memory (tests)
├── phenotype-event-sourcing-derive/     # Macros
│   └── src/
│       ├── aggregate_derive.rs
│       └── event_derive.rs
└── Cargo.toml (workspace)
```

---

### 4. Policy Engine Architecture

#### Current: Regex-Based

```rust
pub struct PolicyEngine {
    rules: Vec<PolicyRule>,
}

impl PolicyEngine {
    pub fn evaluate(&self, ctx: &Context) -> Result<bool> {
        for rule in &self.rules {
            if rule.regex.is_match(&ctx.action)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
```

#### Proposed: RBAC with Casbin

```
phenotype-policy/
├── phenotype-policy-core/              # Core policy logic
│   └── src/
│       ├── engine.rs                   # Policy engine
│       ├── context.rs                  # Evaluation context
│       └── decision.rs                 # Decision types
├── phenotype-policy-casbin/            # Casbin adapter
│   └── src/
│       ├── model.rs                    # Casbin model
│       └── adapter.rs                  # Storage adapter
├── phenotype-policy-rules/             # Built-in rules
│   └── src/
│       ├── rbac.rs                     # RBAC rules
│       └── abac.rs                     # ABAC rules
└── phenotype-policy-cli/               # CLI tools
    └── src/
        └── test_policy.rs
```

---

### 5. CLI Architecture

#### Current: Monolithic

```
agileplus-cli/
├── src/
│   ├── main.rs                        # 2,000 LOC
│   ├── commands/                      # 3,000 LOC
│   ├── config/                        # 1,500 LOC
│   └── utils/                         # 2,384 LOC
└── Cargo.toml
```

#### Proposed: Modular

```
phenotype-cli/
├── phenotype-cli-core/                 # 2,000 LOC - Core
│   └── src/
│       ├── app.rs                     # CLI app
│       ├── output.rs                  # Output formatting
│       └── errors.rs                  # CLI errors
├── phenotype-cli-commands/            # 2,500 LOC - Commands
│   └── src/
│       ├── project.rs                 # Project subcommands
│       ├── task.rs                    # Task subcommands
│       └── user.rs                    # User subcommands
├── phenotype-cli-config/               # 1,000 LOC - Config
│   └── src/
│       ├── loader.rs                  # Config loading
│       └── validation.rs              # Config validation
├── phenotype-cli-display/              # 1,000 LOC - Display
│   └── src/
│       ├── table.rs                  # Table formatting
│       └── tree.rs                   # Tree view
└── phenotype-cli-main/                # 884 LOC - Entry
    └── src/
        └── main.rs
```

---

### 6. Dependency Inversion

#### Current: Direct Dependencies

```
agileplus-api → agileplus-sqlite → rusqlite
agileplus-api → agileplus-git → git2
agileplus-api → agileplus-cache → moka
```

#### Proposed: Ports/Adapters

```
agileplus-api → phenotype-contracts (ports)
                          ↓
          ┌──────────────┼──────────────┐
          ↓              ↓              ↓
phenotype-sqlite  phenotype-git   phenotype-moka
     ↓                 ↓               ↓
   rusqlite           git2           moka
```

---

### 7. Workspace Structure

```toml
# phenotype/Cargo.toml
[workspace]
members = [
    # Core
    "crates/phenotype-contracts",
    "crates/phenotype-errors",
    "crates/phenotype-config",
    
    # Domain
    "crates/phenotype-event-sourcing",
    "crates/phenotype-policy",
    "crates/phenotype-state-machine",
    
    # Infrastructure
    "crates/phenotype-sqlite",
    "crates/phenotype-git",
    "crates/phenotype-cache",
    "crates/phenotype-telemetry",
    
    # CLI
    "crates/phenotype-cli-core",
    "crates/phenotype-cli-commands",
    "crates/phenotype-cli-display",
    
    # API
    "crates/phenotype-api-core",
    "crates/phenotype-api-routes",
    "crates/phenotype-api-models",
    
    # Apps
    "apps/phenotype-server",
    "apps/phenotype-client",
]
resolver = "2"
```

---

_Last updated: 2026-03-29_
