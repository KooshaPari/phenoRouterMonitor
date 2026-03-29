# Agent Collaboration Context

> Template for assigning work to individual agents or agent teams.

---

## Canonical Documentation

| Document | Purpose | Location |
|----------|---------|----------|
| **Main Audit Framework** | Full audit protocol | `AUDIT_FRAMEWORK.md` |
| **Plan Log** | Strategic roadmap | `PLAN_LOG.md` |
| **Research Log** | Initial findings | `RESEARCH_LOG.md` |
| **Audit Log** | Real-time tracking | `AUDIT_LOG.md` |
| **Agent Registry** | Claim tracking | `AGENT_REGISTRY.md` |

---

## Agent Assignment Template

Copy and customize for each agent:

```markdown
## Agent Assignment: [AGENT-XX]

### Your Role
- **Category**: [Duplication | Packages | Decomposition | Dead Code | API | Test]
- **Focus**: [specific pkg X, repo Y, pattern category Z]
- **Priority**: 🔴 CRITICAL | 🟡 HIGH | 🟠 MEDIUM | 🟢 LOW

### Working Context
- **Root Directory**: [path]
- **Target Files**: [specific files to audit]
- **Avoid Files**: [files claimed by other agents]
- **Report To**: [audit category file]

### Objectives
1. [ ] Audit [specific area]
2. [ ] Document findings in [specific format]
3. [ ] Add to CODEBASE_ATLAS.md if new discovery
4. [ ] Mark action items with checkbox format

### Coordination
- **Claim your scope**: Add entry to Agent Registry
- **Report progress**: Every 30 minutes
- **Escalate**: Blockers to [coordinator]
```

---

## Phase 1: Duplication (8 Agents)

### AGENT-01: Error Types (api, domain)
**Scope**: `crates/agileplus-api/src/error.rs`, `crates/agileplus-domain/src/error.rs`
**Tasks**: Document all error variants, identify duplicates

### AGENT-02: Error Types (sync, events, graph)
**Scope**: `crates/agileplus-sync/src/error.rs`, `crates/agileplus-events/src/error.rs`
**Tasks**: Document SyncError, EventError, GraphError

### AGENT-03: Config Loading
**Scope**: `crates/agileplus-domain/src/config/`, `crates/agileplus-telemetry/src/config.rs`
**Tasks**: Document config patterns, audit libs/config-core

### AGENT-04: Async Traits
**Scope**: `crates/agileplus-nats/src/bus.rs`, `libs/hexagonal-rs/`
**Tasks**: Map existing traits to hexagonal-rs patterns

### AGENT-05: Store Patterns (event, graph)
**Scope**: `crates/agileplus-events/src/store.rs`, `crates/agileplus-graph/src/store.rs`
**Tasks**: Document InMemory implementations

### AGENT-06: Store Patterns (sync, cache)
**Scope**: `crates/agileplus-sync/src/store.rs`, `crates/agileplus-cache/src/store.rs`
**Tasks**: Document InMemory implementations

### AGENT-07: HTTP Clients
**Scope**: `crates/agileplus-plane/src/client/`, `crates/agileplus-github/src/client.rs`
**Tasks**: Document reqwest patterns, recommend shared lib

### AGENT-08: Startup Boilerplate
**Scope**: `crates/agileplus-cli/src/main.rs`, `crates/agileplus-agent-service/src/main.rs`
**Tasks**: Document initialization patterns

---

## Phase 2: Library (4 Agents)

### LIB-01: hexagonal-rs Deep Dive
**Scope**: `libs/hexagonal-rs/`
**Tasks**: Document patterns, create integration plan

### LIB-02: config-core Audit
**Scope**: `libs/config-core/`
**Tasks**: Audit unused library, create activation strategy

### LIB-03: Remaining Libraries
**Scope**: `cipher`, `gauge`, `nexus`, `xdd-lib-rs`, `hexkit`
**Tasks**: Categorize: ACTIVATE | ARCHIVE | DELETE

### DEAD-01: Dead Code
**Scope**: All unused code
**Tasks**: Create dead code inventory

---

## Phase 3: Decomposition (6 Agents)

### DEC-01: Hexagonal Boundaries
**Scope**: `crates/agileplus-domain/src/`
**Tasks**: Audit port definitions, document violations

### DEC-02: Adapter Violations
**Scope**: All adapter crates
**Tasks**: Map adapter dependencies, identify violations

### DEC-03: God Modules
**Scope**: Files >300 LOC
**Tasks**: Identify large files, recommend splits

### DEC-04: Cross-Cutting Concerns
**Scope**: Logging, config, errors
**Tasks**: Map distribution, recommend centralization

### DEC-05: Circular Dependencies
**Scope**: All crates
**Tasks**: Check for cycles, recommend solutions

### DEC-06: Module Naming
**Scope**: All crate directories
**Tasks**: Audit naming, recommend consistency

---

## Phase 4: Packages (5 Agents)

### PKG-01: Core Dependencies
**Scope**: tokio, serde, tracing
**Tasks**: Audit feature flags, recommend optimizations

### PKG-02: Web/HTTP
**Scope**: reqwest, axum, warp
**Tasks**: Document HTTP patterns, warp migration

### PKG-03: Data/Storage
**Scope**: sqlx, redis, rocksdb
**Tasks**: Audit connection patterns, security

### PKG-04: Observability
**Scope**: opentelemetry, metrics
**Tasks**: Audit OTel integration, recommend standards

### PKG-05: Utilities
**Scope**: anyhow, thiserror, uuid, chrono
**Tasks**: Document patterns, identify forks

---

## Phase 5: API & Tests (7 Agents)

### API-01 to API-04: Per-Crate API Documentation
**Scope**: Various crates
**Tasks**: Document public APIs, add missing docs

### T-01 to T-03: Test Coverage
**Scope**: Various crates
**Tasks**: Run coverage, identify gaps

---

## Coordination Protocol

### Before Starting
1. Claim your scope in AGENT_REGISTRY.md
2. Read relevant worklog in `worklogs/`
3. Follow output format in AUDIT_FRAMEWORK.md

### While Working
1. Use finding templates exactly
2. Cite with `filepath:line` format
3. Update CODEBASE_ATLAS.md with discoveries
4. Report progress every 30 minutes

### After Completing
1. Verify quality gates
2. Submit findings to category file
3. Notify completion

---

## Priority Definitions

| Priority | Meaning | Response Time |
|----------|---------|---------------|
| 🔴 CRITICAL | Blocking, security | 24h |
| 🟡 HIGH | Significant issue | 1 week |
| 🟠 MEDIUM | Notable issue | 1 month |
| 🟢 LOW | Nice to fix | Backlog |

---

_Last updated: 2026-03-29_
