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
### Next Steps:

- [ ] Create ADRs for key architectural decisions
- [ ] Evaluate fork candidates for shared libraries
- [ ] Add progress feedback with indicatif

---

## 2026-03-29 - CLI Architecture Patterns (AgilePlus)

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P2

### Summary

Reviewed CLI architecture patterns in `crates/agileplus-cli` for consistency with industry standards.

### Command Structure

```
agileplus-cli/src/
├── commands/
│   ├── mod.rs           # Command registry
│   ├── specify.rs       # Feature specification
│   ├── plan.rs          # Planning commands
│   ├── implement.rs     # Implementation commands
│   ├── validate.rs      # Validation commands
│   └── ship.rs          # Ship commands
└── main.rs              # Entry point
```

### Pattern Analysis

| Pattern | Current | Recommendation |
|---------|---------|----------------|
| CLI framework | clap | ✅ Optimal |
| Subcommands | Nested | ✅ Good |
| Help text | Inline | Add --help examples |
| Error output | Standard | Use colored error |

### Recommendations

1. Add `--json` output flag for scripting
2. Standardize error codes (1=error, 2=validation failed)
3. Add `--verbose` and `--quiet` flags
4. Consider adding shell completions

### Related

- `crates/agileplus-cli/src/commands/`
- `libs/cli-framework/`

---

## 2026-03-29 - Graph Architecture Deep Dive

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P2

### Summary

Deep analysis of graph architecture in `crates/agileplus-graph`.

### Architecture Layers

```
┌─────────────────────────────────────┐
│  Graph Service (HTTP API)            │
├─────────────────────────────────────┤
│  Graph Query Engine (Cypher)        │
├─────────────────────────────────────┤
│  Graph Backend (Neo4j/In-Memory)    │
├─────────────────────────────────────┤
│  Connection Pool (bb8)              │
└─────────────────────────────────────┘
```

### Key Components

| Component | File | Purpose |
|-----------|------|---------|
| GraphService | `src/lib.rs` | HTTP endpoints |
| GraphBackend trait | `src/store.rs` | Backend abstraction |
| Cypher parser | `src/cypher.rs` | Query parsing |
| Bolt protocol | `src/bolt.rs` | Neo4j communication |

### Backend Implementations

| Backend | Location | Status |
|---------|----------|--------|
| InMemoryBackend | `src/store.rs:106` | For testing |
| Neo4jBackend | `src/neo4j.rs` | Production |

### Recommendations

1. Add connection health checks
2. Implement query result caching
3. Add Cypher query validation
4. Consider adding relationship constraints

---

## 2026-03-29 - Event Sourcing Architecture Review

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P1

### Summary

Review of event sourcing patterns in `crates/agileplus-events`.

### Event Sourcing Pattern

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Command    │────▶│    Event     │────▶│   State      │
│   Handler    │     │   Store      │     │   Aggregate  │
└──────────────┘     └──────────────┘     └──────────────┘
```

### Key Traits

| Trait | Location | Purpose |
|-------|----------|---------|
| EventStore | `src/store.rs:21` | Store/retrieve events |
| SnapshotStore | `src/snapshot.rs` | State snapshots |
| EventHandler | `src/handler.rs` | Event processing |

### Snapshot Strategy

| Strategy | When | Benefit |
|----------|------|---------|
| Time-based | Every N events | Predictable storage |
| Size-based | Every N bytes | Consistent growth |
| Delta | Every M versions | Fast replay |

### Recommendations

1. Implement snapshot compression
2. Add event versioning
3. Create event upcasting pattern
4. Consider event projections

---

## 2026-03-29 - Cache Architecture Analysis

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P2

### Summary

Analysis of caching architecture in `crates/agileplus-cache`.

### Cache Layers

```
┌─────────────────────────────────────┐
│  Application Layer                  │
├─────────────────────────────────────┤
│  CacheStore Trait                   │
├─────────────────────────────────────┤
│  Redis Connection Pool (bb8)        │
├─────────────────────────────────────┤
│  Redis Server                       │
└─────────────────────────────────────┘
```

### Key Components

| Component | File | Purpose |
|-----------|------|---------|
| CacheStore | `src/store.rs` | Cache trait |
| RedisPool | `src/pool.rs` | Connection pool |
| CacheConfig | `src/config.rs` | Configuration |

### TTL Strategy

| Key Type | TTL | Reason |
|----------|-----|--------|
| Feature metadata | 1 hour | Mutable |
| User preferences | 24 hours | Stable |
| Session data | 30 min | Security |

### Recommendations

1. Add cache warming on startup
2. Implement cache invalidation webhooks
3. Add cache metrics (hit/miss ratio)
4. Consider tiered caching (L1/L2)

---

## 2026-03-29 - gRPC Architecture Review

**Project:** [AgilePlus]
****Category:** architecture
**Status:** completed
**Priority:** P2

### Summary

Review of gRPC architecture in `crates/agileplus-grpc`.

### Service Definitions

```
proto/
├── agileplus.proto     # Main service definitions
├── events.proto        # Event bus definitions
└── agent.proto         # Agent dispatch definitions
```

### Service Inventory

| Service | Port | Purpose |
|---------|------|---------|
| AgilePlusService | 50051 | Main API |
| EventBusService | 50052 | Event streaming |
| AgentService | 50053 | Agent dispatch |

### Interceptors

| Interceptor | Purpose |
|-------------|---------|
| AuthInterceptor | JWT validation |
| LoggingInterceptor | Request logging |
| MetricsInterceptor | Prometheus metrics |
| RateLimitInterceptor | Rate limiting |

### Recommendations

1. Add circuit breaker pattern
2. Implement client-side load balancing
3. Add health checking endpoints
4. Consider gRPC-web for browser clients

---

## 2026-03-29 - NATS Architecture Deep Dive

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P1

### Summary

Deep analysis of NATS/JetStream architecture in `crates/agileplus-nats`.

### Architecture

```
┌─────────────────────────────────────┐
│  Publishers                         │
│  (Feature CRUD, Agent events)       │
└──────────────┬──────────────────────┘
               │ NATS
┌──────────────▼──────────────────────┐
│  JetStream (Durable streams)         │
│  - FeatureEvents (subjects.*)         │
│  - AgentEvents (agents.*)             │
│  - AuditEvents (audit.*)             │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│  Subscribers                         │
│  (Event handlers, Audit log)          │
└─────────────────────────────────────┘
```

### Key Components

| Component | File | Purpose |
|-----------|------|---------|
| EventBus | `src/bus.rs` | Publisher trait |
| NatsClient | `src/client.rs` | Connection management |
| StreamManager | `src/streams.rs` | Stream configuration |

### Subjects

| Subject | Purpose | Retention |
|---------|---------|-----------|
| `features.*` | Feature events | Durable |
| `agents.*` | Agent events | Durable |
| `audit.*` | Audit trail | Durable |

### Recommendations

1. Implement dead-letter queues
2. Add message schema validation
3. Configure consumer groups properly
4. Add backpressure handling

---

## 2026-03-29 - Telemetry Architecture Review

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P2

### Summary

Review of telemetry architecture in `crates/agileplus-telemetry`.

### Three Pillars

```
┌─────────────────────────────────────────────────────┐
│                    Telemetry                         │
├─────────────────┬─────────────────┬─────────────────┤
│    Tracing      │    Metrics      │     Logs        │
│    (Spans)     │    (Stats)      │   (Events)      │
└─────────────────┴─────────────────┴─────────────────┘
```

### OpenTelemetry Integration

| Signal | Exporter | Destination |
|--------|----------|-------------|
| Traces | OTLP | Jaeger/OTEL Collector |
| Metrics | OTLP | Prometheus/OTEL |
| Logs | OTLP | Loki/OTEL |

### Key Components

| Component | File | Purpose |
|-----------|------|---------|
| TelemetryConfig | `src/config.rs` | Configuration |
| TracingLayer | `src/tracing.rs` | Span management |
| MetricsRegistry | `src/metrics.rs` | Metric collection |

### Recommendations

1. Add sampling strategies
2. Implement trace context propagation
3. Add metric cardinality limits
4. Consider log aggregation tiering

---

## 2026-03-29 - API Gateway Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Summary

Design for API gateway layer in `crates/agileplus-api`.

### Gateway Responsibilities

| Responsibility | Implementation |
|----------------|----------------|
| Authentication | JWT validation |
| Rate limiting | Token bucket |
| Request routing | Path-based |
| Response caching | Cache-Control headers |
| API versioning | URL prefix (/v1/, /v2/) |

### Middleware Stack

```
Request → Auth → RateLimit → Cache → Route → Handler → Response
         ↓         ↓           ↓        ↓
        401      429         HIT     404
```

### Endpoint Groups

| Group | Prefix | Services |
|-------|--------|----------|
| Features | /v1/features | agileplus-api |
| Agents | /v1/agents | agileplus-api |
| Events | /v1/events | agileplus-events |
| Admin | /v1/admin | agileplus-admin |

### Recommendations

1. Add OpenAPI documentation
2. Implement request/response logging
3. Add CORS configuration
4. Consider GraphQL federation later

---

## 2026-03-29 - Database Schema Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P1

### Summary

Analysis of database schema architecture across AgilePlus.

### Schema Overview

```
┌─────────────────────────────────────────────────────┐
│  SQLite (agileplus.db)                              │
├─────────────────────────────────────────────────────┤
│  Features │ Agents │ Workspaces │ Credentials       │
├─────────────────────────────────────────────────────┤
│  Graph (Neo4j)                                       │
├─────────────────────────────────────────────────────┤
│  FeatureGraph │ AgentRelations │ DependencyGraph   │
├─────────────────────────────────────────────────────┤
│  Redis (Cache)                                       │
├─────────────────────────────────────────────────────┤
│  FeatureCache │ SessionCache │ RateLimit           │
└─────────────────────────────────────────────────────┘
```

### SQLite Tables

| Table | Purpose | Indexes |
|-------|---------|---------|
| features | Feature metadata | name, workspace_id |
| agents | Agent registry | type, workspace_id |
| workspaces | Workspace config | owner_id |
| credentials | Secrets (encrypted) | workspace_id |

### Graph Schema

| Node | Properties | Labels |
|------|------------|--------|
| Feature | id, name, status | Feature |
| Agent | id, type, status | Agent |
| Workspace | id, name | Workspace |

### Recommendations

1. Add foreign key constraints
2. Implement soft deletes
3. Add audit columns (created_at, updated_at)
4. Consider partitioning for large tables

---

## 2026-03-29 - Microservices Communication Patterns

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P1

### Communication Matrix

| From \ To | API | Events | gRPC |
|-----------|-----|--------|------|
| CLI | ✅ REST | ❌ | ❌ |
| Dashboard | ✅ REST | ❌ | ❌ |
| MCP Server | ❌ | ✅ | ✅ |
| Agents | ❌ | ✅ | ✅ |
| External | ✅ REST | ❌ | ❌ |

### Sync vs Async

| Interaction | Pattern | Use Case |
|-------------|---------|----------|
| Feature CRUD | Synchronous | REST |
| Event publishing | Asynchronous | NATS |
| Agent dispatch | Synchronous | gRPC |
| Audit logging | Asynchronous | NATS |

### Circuit Breaker

| Service | Threshold | Timeout |
|---------|-----------|---------|
| GitHub API | 5 failures | 30s |
| Neo4j | 3 failures | 10s |
| Redis | 3 failures | 5s |

### Recommendations

1. Add circuit breaker to all external calls
2. Implement retry with exponential backoff
3. Add request timeouts everywhere
4. Consider message queue for heavy operations

---

## 2026-03-29 - Authentication & Authorization Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P0

### Auth Flow

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│  Client  │────▶│   API    │────▶│   Auth   │
│          │◀────│  Gateway  │◀────│  Service │
└──────────┘     └──────────┘     └──────────┘
```

### Token Types

| Token | Lifetime | Storage |
|-------|----------|---------|
| Access JWT | 15 min | Memory |
| Refresh JWT | 7 days | HttpOnly cookie |
| API Key | Non-expiring | Database |

### Permission Model

```
Workspace
  └── Members (role: owner|admin|member|viewer)
        └── Permissions (CRUD on resources)
```

### Key Components

| Component | File | Purpose |
|-----------|------|---------|
| JwtService | `agileplus-auth/` | Token generation |
| PermissionChecker | `agileplus-domain/` | Authorization |
| CredentialStore | `agileplus-domain/` | Secrets management |

### Recommendations

1. Add MFA support
2. Implement SSO (OAuth2/OIDC)
3. Add audit logging for auth events
4. Consider key rotation automation

---

## 2026-03-29 - Storage Abstraction Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P1

### Storage Traits

```
┌─────────────────────────────────────────────────────┐
│  Application Layer                                   │
├─────────────────────────────────────────────────────┤
│  StoragePort (trait)                                 │
│    ├── FeatureStore                                  │
│    ├── AgentStore                                    │
│    ├── WorkspaceStore                                │
│    └── CredentialStore                               │
├─────────────────────────────────────────────────────┤
│  Adapters                                           │
│    ├── SQLiteAdapter                                 │
│    ├── InMemoryAdapter (tests)                       │
│    └── MockAdapter (tests)                           │
└─────────────────────────────────────────────────────┘
```

### Trait Definition

```rust
#[async_trait]
pub trait StoragePort<E: Entity> {
    async fn find(&self, id: &E::Id) -> Result<Option<E>>;
    async fn save(&self, entity: &E) -> Result<()>;
    async fn delete(&self, id: &E::Id) -> Result<()>;
    async fn list(&self, filter: Filter) -> Result<Vec<E>>;
}
```

### Implementation Status

| Trait | SQLite | InMemory | Mock |
|-------|--------|----------|------|
| FeatureStore | ✅ | ✅ | ✅ |
| AgentStore | ✅ | ✅ | ✅ |
| WorkspaceStore | ✅ | ✅ | ✅ |
| CredentialStore | ✅ | ✅ | ❌ |

### Recommendations

1. Add PostgreSQL adapter option
2. Implement optimistic locking
3. Add query builder pattern
4. Consider event sourcing adapter

---

## 2026-03-29 - Testing Architecture Patterns

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P2

### Test Pyramid

```
              ┌─────────────┐
              │   E2E Tests │  (10%)
             ┌┴─────────────┴┐
            ┌┴───────────────┴┐
           ┌┴─────────────────┴┐
           │    Integration     │  (30%)
          ┌┴───────────────────┴┐
         ┌┴─────────────────────┴┐
        ┌┴───────────────────────┴┐
        │       Unit Tests        │  (60%)
       ┌┴─────────────────────────┴┐
```

### Test Infrastructure

| Layer | Framework | Location |
|-------|-----------|----------|
| Unit | #[test], tokio::test | `src/**/*.rs` |
| Integration | testcontainers | `tests/integration/` |
| E2E | trycmd | `tests/e2e/` |

### Test Doubles

| Type | Usage | Example |
|------|-------|---------|
| Mock | Interface verification | MockStore |
| Stub | Fixed responses | StubConfig |
| Fake | In-memory impl | InMemoryStore |
| Spy | Call recording | SpyEventBus |

### Recommendations

1. Increase integration test coverage
2. Add mutation testing (mutant)
3. Implement property-based tests (proptest)
4. Add contract testing (pact)

---

## 2026-03-29 - Deployment Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P1

### Deployment Options

| Option | Use Case | Complexity |
|--------|----------|------------|
| Docker Compose | Local dev | Low |
| Kubernetes | Production | High |
| NixOS | Reproducible | Medium |
| Single binary | Lightweight | Low |

### Container Structure

```
agileplus/
├── api/Dockerfile        # REST API
├── grpc/Dockerfile       # gRPC services
├── mcp/Dockerfile        # MCP server
└── dashboard/Dockerfile  # Web UI
```

### Kubernetes Resources

| Resource | Purpose |
|----------|---------|
| Deployment | Pod management |
| Service | Internal networking |
| Ingress | External access |
| ConfigMap | Configuration |
| Secret | Sensitive data |
| HPA | Auto-scaling |

### Recommendations

1. Add Helm charts
2. Implement rolling deployments
3. Add PodDisruptionBudgets
4. Configure resource limits

---

## 2026-03-29 - Configuration Management Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P1

### Config Hierarchy

```
┌─────────────────────────────────────────┐
│  Environment Variables (highest)         │
├─────────────────────────────────────────┤
│  CLI Arguments                          │
├─────────────────────────────────────────┤
│  User Config (~/.agileplus/config.toml) │
├─────────────────────────────────────────┤
│  Project Config (.agileplus/config.toml)│
├─────────────────────────────────────────┤
│  Defaults (lowest)                      │
└─────────────────────────────────────────┘
```

### Config Sources

| Source | Priority | Example |
|--------|----------|---------|
| ENV | 100 | AGILEPLUS_DATABASE_URL |
| Args | 90 | --port 8080 |
| User | 80 | ~/.agileplus/config.toml |
| Project | 70 | .agileplus/config.toml |
| Defaults | 0 | Built-in values |

### Key Components

| Component | Location | Purpose |
|-----------|----------|---------|
| ConfigLoader | `agileplus-domain/config/` | Multi-source loading |
| EnvPrefix | "AGILEPLUS_" | ENV variable prefix |
| ConfigBuilder | Various | Runtime config building |

### Recommendations

1. Add config schema validation (with JSON Schema)
2. Implement config hot-reload
3. Add config diff tool
4. Consider feature flags framework

---

## 2026-03-29 - Observability Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P1

### Observability Stack

```
┌─────────────────────────────────────────────────────┐
│  AgilePlus Application                               │
├─────────────────────────────────────────────────────┤
│  OpenTelemetry SDK                                   │
├─────────────────────────────────────────────────────┤
│  ┌─────────┬─────────┬─────────┐                    │
│  │ Traces  │ Metrics │  Logs   │                    │
│  └────┬────┴────┬────┴────┬────┘                    │
├───────┼─────────┼─────────┼────────────────────────┤
│       │         │         │                         │
│  ┌────▼────┐┌───▼───┐┌───▼───┐                    │
│  │ Jaeger  ││Prometheus││ Loki │                    │
│  └─────────┘└────────┘└────────┘                    │
└─────────────────────────────────────────────────────┘
```

### Key Signals

| Signal | Collection | Storage | Retention |
|--------|-----------|---------|-----------|
| Traces | OTEL Collector | Jaeger | 30 days |
| Metrics | Prometheus | TSDB | 13 months |
| Logs | Promtail | Loki | 90 days |

### Important Traces

| Trace | Purpose | Key Spans |
|-------|---------|-----------|
| Feature lifecycle | End-to-end flow | create→specify→plan→ship |
| Agent dispatch | Agent execution | spawn→execute→collect |
| Event processing | Eventual consistency | publish→consume→handle |

### Recommendations

1. Add SLO/SLI definitions
2. Implement alerting rules
3. Add dashboard templates
4. Consider continuous profiling

---

## 2026-03-29 - Message Queue Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P1

### Message Flow

```
┌──────────┐     ┌──────────────┐     ┌──────────┐
│ Producer │────▶│ NATS/JetStream│────▶│ Consumer │
└──────────┘     └──────────────┘     └──────────┘
                       │
                       ▼
              ┌──────────────┐
              │   Streams     │
              │ (Durable)    │
              └──────────────┘
```

### Stream Configuration

| Stream | Subjects | Retention | Replicas |
|--------|----------|-----------|----------|
| Features | `features.*` | ByTime | 3 |
| Agents | `agents.*` | ByTime | 3 |
| Audit | `audit.*` | Forever | 5 |

### Consumer Groups

| Consumer | Stream | Filter |
|----------|--------|--------|
| FeatureHandler | Features | `features.feature.*` |
| AgentScheduler | Agents | `agents.dispatch.*` |
| AuditLogger | Audit | `audit.*` |

### Error Handling

| Strategy | Implementation |
|----------|----------------|
| Retry | 3 attempts with backoff |
| DLQ | Dead letter subject |
| Alert | Alert on DLQ messages |

### Recommendations

1. Add message schema registry
2. Implement message replay tool
3. Add consumer lag monitoring
4. Consider Kafka for higher throughput

---

## 2026-03-29 - Security Architecture Review

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P0

### Security Layers

```
┌─────────────────────────────────────────────────────┐
│  Network Layer (Firewall, VPN)                       │
├─────────────────────────────────────────────────────┤
│  TLS (mTLS for services)                             │
├─────────────────────────────────────────────────────┤
│  Authentication (JWT, API Keys)                      │
├─────────────────────────────────────────────────────┤
│  Authorization (RBAC, ABAC)                           │
├─────────────────────────────────────────────────────┤
│  Secrets Management (Vault, AWS Secrets)              │
├─────────────────────────────────────────────────────┤
│  Audit Logging (Immutable trail)                      │
└─────────────────────────────────────────────────────┘
```

### Security Components

| Component | Implementation | Status |
|-----------|---------------|--------|
| AuthN | JWT validation | ✅ |
| AuthZ | Permission checks | ✅ |
| Secrets | Environment/Secret Manager | ⚠️ Partial |
| Audit | Event logging | ✅ |
| Encryption | At-rest, in-transit | ✅ |

### Threat Model

| Threat | Mitigation |
|--------|------------|
| Credential theft | Short-lived JWTs, API key rotation |
| Privilege escalation | Least privilege RBAC |
| Data exfiltration | Network policies, DLP |
| Insider threat | Audit logging, separation of duties |

### Recommendations

1. Add dependency scanning (cargo-audit)
2. Implement SAST (cargo-audit, semgrep)
3. Add DAST in CI pipeline
4. Consider SBOM generation

---

## 2026-03-29 - API Versioning Strategy

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Versioning Approaches

| Approach | Pros | Cons |
|----------|------|------|
| URL Path (/v1/) | Clear, easy routing | Version in every request |
| Header (Accept: v1) | Clean URLs | Complex routing |
| Query (?version=1) | Easy testing | Cluttered URLs |

### Current Decision: URL Path

```
/v1/features/{id}
/v2/features/{id}  (when breaking changes)
```

### Deprecation Policy

| Phase | Duration | Actions |
|-------|----------|---------|
| Announced | - | Add deprecation header |
| Deprecated | 6 months | Log warnings, reject new usage |
| Sunset | - | Remove old version |

### Breaking Changes

| Change | Breaking? | Migration |
|--------|-----------|-----------|
| Add field | No | Client ignores |
| Remove field | Yes | Add field, keep old |
| Rename field | Yes | Add alias |
| Change type | Yes | New field, keep old |

### Recommendations

1. Document breaking change policy
2. Add API changelog
3. Implement version negotiation
4. Consider GraphQL (no versioning)

---

## 2026-03-29 - Multi-Tenancy Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P1

### Tenancy Models

| Model | Isolation | Complexity | Use Case |
|-------|-----------|------------|----------|
| Shared database | Schema | Low | Small scale |
| Shared schema | Row | Medium | Medium scale |
| Separate schema | Schema | High | Compliance |
| Separate database | Database | Very High | Enterprise |

### Current Decision: Shared Schema with Row-Level Isolation

```sql
SELECT * FROM features
WHERE workspace_id = :workspace_id;
```

### Tenant Context

```rust
pub struct TenantContext {
    workspace_id: WorkspaceId,
    user_id: UserId,
    permissions: Vec<Permission>,
}
```

### Data Isolation

| Resource | Isolation Method |
|----------|------------------|
| Features | workspace_id FK |
| Agents | workspace_id FK |
| Workspaces | Tenant root |
| Credentials | Encrypted per workspace |

### Recommendations

1. Add tenant isolation tests
2. Implement tenant quota system
3. Add cross-tenant query prevention
4. Consider tenant-specific backups

---

## 2026-03-29 - Plugin Architecture Design

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Plugin Model

```
┌─────────────────────────────────────────────────────┐
│  Core (agileplus-core)                              │
├─────────────────────────────────────────────────────┤
│  Plugin Host (runtime, API surface)                  │
├─────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐          │
│  │ Plugin A │  │ Plugin B │  │ Plugin C │          │
│  └──────────┘  └──────────┘  └──────────┘          │
└─────────────────────────────────────────────────────┘
```

### Plugin API Surface

| Interface | Purpose |
|-----------|---------|
| FeatureExtension | Add feature types |
| AgentPlugin | Custom agent implementations |
| StorageAdapter | Alternative storage backends |
| AuthProvider | SSO/OAuth integrations |

### Plugin Loading

| Strategy | Pros | Cons |
|----------|------|------|
| Static linking | Fast, simple | Rebuilt for each plugin |
| Dynamic (.so) | Hot reload | Complexity |
| WASM | Sandboxed | Performance overhead |

### Current Plugin Registry

```toml
# Cargo.toml
[dependencies]
agileplus-plugin-git = { git = "...", optional = true }
agileplus-plugin-sqlite = { git = "...", optional = true }
```

### Recommendations

1. Define plugin manifest schema
2. Implement plugin discovery
3. Add plugin sandboxing
4. Consider WASM for third-party plugins

---

## 2026-03-29 - Backup & Disaster Recovery Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P1

### Backup Strategy

| Data Store | Backup Method | Frequency | Retention |
|------------|---------------|-----------|-----------|
| SQLite | File copy + WAL | Hourly | 7 days |
| Neo4j | dump command | Daily | 30 days |
| Redis | RDB + AOF | Every 5 min | 7 days |
| NATS | JetStream snapshots | Hourly | 24 hours |

### Recovery Objectives

| Objective | Target | Current |
|-----------|--------|---------|
| RTO (Recovery Time) | 1 hour | Unknown |
| RPO (Recovery Point) | 5 minutes | Unknown |

### Backup Verification

| Test | Frequency | Alert on Failure |
|------|-----------|------------------|
| Restore to staging | Weekly | Yes |
| Verify integrity | Daily | Yes |
| DR drill | Quarterly | Yes |

### DR Site

| Component | Primary | DR Site |
|-----------|---------|---------|
| Database | Local | Cloud region B |
| Blob storage | Local | Cloud storage |
| Compute | Local | Cloud instances |

### Recommendations

1. Implement automated backup testing
2. Add backup encryption at rest
3. Document recovery procedures
4. Run quarterly DR drills

---

## 2026-03-29 - Feature Flags Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P3

### Flag Types

| Type | Purpose | Example |
|------|---------|---------|
| Release | Gradual rollout | new_feature_v2 |
| Experiment | A/B testing | checkout_flow_variant |
| Operational | Kill switch | disable_payment |
| Permission | Access control | premium_feature |

### Flag Evaluation

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Request    │────▶│   Flag      │────▶│   Return    │
│  (user_id)  │     │  Service    │     │  (enabled?) │
└─────────────┘     └─────────────┘     └─────────────┘
```

### Flag Configuration

```json
{
  "flag": "new_feature_v2",
  "rules": [
    { "percentage": 10, "rollout": "gradual" },
    { "users": ["user_123"], "rollout": "include" },
    { "percentage": 100, "after": "2026-04-01" }
  ]
}
```

### Implementation Status

| Feature | Status | Location |
|---------|--------|----------|
| Flag service | Partial | agileplus-feature-flags |
| SDK | Planned | TBD |
| Dashboard | Not started | TBD |

### Recommendations

1. Choose flag service (LaunchDarkly vs self-hosted)
2. Add flag evaluation caching
3. Implement audit logging for flag changes
4. Add flag dependency support

---

## 2026-03-28 - Dependency Injection Patterns

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P2

### DI Approaches

| Approach | Pros | Cons |
|----------|------|------|
| Manual DI | Simple, explicit | Boilerplate |
| Builder pattern | Fluent, optional deps | Complex |
| Container (ctx) | Automatic | Magic, runtime errors |

### Current Pattern: Context-based DI

```rust
pub struct AppContext {
    pub db: SqlitePool,
    pub graph: GraphBackend,
    pub cache: CacheStore,
    pub nats: NatsClient,
    pub config: AppConfig,
}
```

### Constructor Injection

```rust
impl FeatureService {
    pub fn new(store: FeatureStore, cache: CacheStore) -> Self {
        Self { store, cache }
    }
}
```

### Trait Object Injection

```rust
#[async_trait]
pub trait FeatureRepository: Send + Sync {
    async fn find(&self, id: FeatureId) -> Result<Option<Feature>>;
    async fn save(&self, feature: &Feature) -> Result<()>;
}
```

### Recommendations

1. Consider using `ctx` crate for DI
2. Add dependency validation on startup
3. Implement health checks for dependencies
4. Add circuit breakers for external deps

---

## 2026-03-28 - Error Handling Patterns

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P1

### Error Handling Philosophy

```
┌─────────────────────────────────────────────────────┐
│  User Errors (4xx) - Show user-friendly message      │
├─────────────────────────────────────────────────────┤
│  System Errors (5xx) - Log details, show generic     │
├─────────────────────────────────────────────────────┤
│  Validation Errors - Show field-level feedback        │
└─────────────────────────────────────────────────────┘
```

### Error Response Format

```json
{
  "error": {
    "code": "FEATURE_NOT_FOUND",
    "message": "Feature with ID '123' not found",
    "details": {
      "id": "123",
      "resource": "feature"
    }
  }
}
```

### Error Categories

| Category | HTTP Code | Example |
|----------|-----------|---------|
| NotFound | 404 | Resource doesn't exist |
| Unauthorized | 401 | Invalid/missing token |
| Forbidden | 403 | Insufficient permissions |
| Validation | 400 | Invalid input |
| Conflict | 409 | Duplicate resource |
| Internal | 500 | Unexpected error |

### Logging Strategy

| Error Type | Log Level | Include Details |
|------------|-----------|-----------------|
| User error | WARN | Message only |
| System error | ERROR | Full stack trace |
| Validation | INFO | Field names |
| Auth failure | WARN | User ID, IP |

### Recommendations

1. Standardize error codes across services
2. Add error correlation IDs
3. Implement error boundaries in UI
4. Create error code documentation

---

## 2026-03-28 - Logging Best Practices

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P2

### Structured Logging

```json
{
  "timestamp": "2026-03-29T10:15:30Z",
  "level": "INFO",
  "message": "Feature created",
  "context": {
    "feature_id": "123",
    "workspace_id": "ws_456",
    "user_id": "user_789"
  },
  "trace_id": "abc123",
  "span_id": "def456"
}
```

### Log Levels

| Level | Usage | Example |
|-------|-------|---------|
| TRACE | Debug details | Variable values |
| DEBUG | Developer info | Function entry/exit |
| INFO | Business events | Feature created |
| WARN | Recoverable issues | Retry attempt |
| ERROR | Failures | Database unavailable |

### Sensitive Data Handling

| Data Type | Action | Reason |
|-----------|--------|--------|
| Passwords | Never log | Security |
| API keys | Never log | Security |
| User IDs | OK to log | PII considerations |
| Feature content | Mask | Business sensitive |

### Log Correlation

```rust
let span = tracing::info_span!("create_feature");
span.set_parent(parent_context.trace_id());
```

### Recommendations

1. Add log sampling for high-volume paths
2. Implement log aggregation tiering
3. Add log-based alerting rules
4. Create log retention policies

---

## 2026-03-28 - Rate Limiting Architecture

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P1

### Rate Limit Tiers

| Tier | Requests/min | Burst | Use Case |
|------|--------------|-------|----------|
| Free | 60 | 10 | Development |
| Pro | 600 | 100 | Production |
| Enterprise | 6000 | 1000 | High-volume |

### Rate Limit Algorithm

| Algorithm | Pros | Cons |
|-----------|------|------|
| Token Bucket | Smooth, allows bursts | Complex |
| Leaky Bucket | Fair, constant rate | No bursts |
| Fixed Window | Simple | Boundary spikes |
| Sliding Window | Accurate | Memory intensive |

### Implementation: Token Bucket

```rust
struct RateLimiter {
    capacity: u32,
    tokens: u32,
    refill_rate: f64,  // tokens per second
    last_refill: Instant,
}
```

### HTTP Headers

| Header | Value | Purpose |
|--------|-------|---------|
| X-RateLimit-Limit | 60 | Max requests |
| X-RateLimit-Remaining | 45 | Remaining |
| X-RateLimit-Reset | 1648534500 | Reset timestamp |
| Retry-After | 30 | Seconds to wait |

### Recommendations

1. Add per-user rate limits
2. Implement rate limit by endpoint
3. Add rate limit bypass for internal services
4. Consider distributed rate limiting (Redis)

---

## 2026-03-28 - Circuit Breaker Pattern

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P2

### State Machine

```
       ┌─────────────────────────────────────┐
       │                                      │
       ▼                                      │
  ┌─────────┐     failure      ┌───────────┐  │
  │ CLOSED  │─────────────────▶│  OPEN     │  │
  │(Normal) │                 │ (Failing) │  │
  └─────────┘◀─────────────────┴───────────┘  │
       │                        │            │
       │    success             │ timeout    │
       │    after half-open     │            │
       ▼                        ▼            │
  ┌─────────┐     failure      ┌───────────┐  │
  │ HALF-   │─────────────────▶│  OPEN     │──┘
  │ OPEN    │                 │ (Failing) │
  │(Testing)│◀─────────────────┴───────────┘
  └─────────┘   success
```

### Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| failure_threshold | 5 | Failures to trip |
| success_threshold | 2 | Successes to close |
| timeout | 30s | Time before half-open |
| half_open_requests | 3 | Requests in half-open |

### Implementation

```rust
pub struct CircuitBreaker {
    state: AtomicState,
    failure_count: AtomicU32,
    last_failure: AtomicI64,
    config: CircuitBreakerConfig,
}
```

### Integration Points

| Service | Breaker Config | Fallback |
|---------|---------------|----------|
| GitHub API | 3 failures, 60s | Return cached data |
| Neo4j | 5 failures, 30s | Return error |
| Redis | 3 failures, 10s | Bypass cache |

### Recommendations

1. Add circuit breaker metrics
2. Implement bulkhead pattern
3. Add circuit breaker for all external calls
4. Consider auto-tuning thresholds

---

## 2026-03-28 - Bulkhead Pattern Implementation

**Project:** [AgilePlus]
**Category:** architecture
**Status:** completed
**Priority:** P3

### Bulkhead Concept

```
┌─────────────────────────────────────────────────────┐
│  Without Bulkhead (shared thread pool)               │
├─────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────┐     │
│  │  Shared Pool (10 threads)                   │     │
│  │  [A1][A2][A3][A4][A5][B1][B2][C1][C2][C3]│     │
│  └─────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│  With Bulkhead (isolated pools)                      │
├─────────────────────────────────────────────────────┤
│  ┌───────────┐ ┌───────────┐ ┌───────────┐         │
│  │ Pool A    │ │ Pool B    │ │ Pool C    │         │
│  │ [A1][A2]  │ │ [B1][B2]  │ │ [C1][C2]  │         │
│  └───────────┘ └───────────┘ └───────────┘         │
└─────────────────────────────────────────────────────┘
```

### Pool Configuration

| Pool | Min | Max | Queue | Purpose |
|------|-----|-----|-------|---------|
| API | 4 | 16 | Bounded | User requests |
| Background | 2 | 8 | Unbounded | Async tasks |
| IO | 8 | 32 | Bounded | External calls |

### Implementation

```rust
let api_pool = ThreadPoolBuilder::new()
    .pool_size(4..16)
    .queue_capacity(100)
    .build()?;

let io_pool = ThreadPoolBuilder::new()
    .pool_size(8..32)
    .queue_capacity(1000)
    .build()?;
```

### Recommendations

1. Implement pool isolation for external services
2. Add pool metrics to dashboard
3. Consider semaphore-based bulkheads
4. Test failure isolation

---

## 2026-03-28 - Saga Pattern for Distributed Transactions

**Project:** [AgilePlus]
**Category:** architecture
**Status:** pending
**Priority:** P2

### Saga Pattern

```
┌─────────────────────────────────────────────────────┐
│  Feature Creation Saga                               │
├─────────────────────────────────────────────────────┤
│  Step 1: Create feature record ──────▶ OK           │
│  Step 2: Initialize agent ────────────▶ OK           │
│  Step 3: Create workspace resources───▶ FAIL          │
│  Step 4: Rollback agent (compensating)              │
│  Step 5: Rollback feature record                    │
└─────────────────────────────────────────────────────┘
```

### Saga Steps

| Step | Action | Compensation |
|------|--------|--------------|
| 1 | Create feature | Delete feature |
| 2 | Initialize agent | Deallocate agent |
| 3 | Create resources | Delete resources |
| 4 | Notify stakeholders | Send cancellation |

### Saga Coordinator

```rust
#[async_trait]
pub trait SagaStep<T: SagaData> {
    async fn execute(&self, data: &T) -> Result<T::Output>;
    async fn compensate(&self, data: &T, output: &T::Output) -> Result<()>;
}

pub struct SagaCoordinator {
    steps: Vec<Box<dyn SagaStep>>,
    persistence: SagaStore,
}
```

### Saga Persistence

| State | Description |
|-------|-------------|
| PENDING | Saga not started |
| RUNNING | Step in progress |
| COMPLETED | All steps done |
| COMPENSATING | Rolling back |
| FAILED | Unrecoverable |

### Recommendations

1. Implement saga orchestrator
2. Add saga state persistence
3. Implement retry with backoff
4. Add saga monitoring/alerting
