# ADR Registry — Architecture Decision Index

**Version:** 1.0
**Status:** Active
**Updated:** 2026-04-01
**Branch:** `specs/main`

---

## Overview

This registry is the authoritative index of all Architecture Decision Records (ADRs) across the Phenotype polyrepo ecosystem. Each ADR documents a significant architectural decision with its context, decision, and consequences.

---

## Master ADR Index

### By Repository & Status

#### phenotype-infrakit (8 ADRs)

| ADR | Title | Status | Date | Impact | Dependencies |
|-----|-------|--------|------|--------|--------------|
| ADR-001 | Rust Workspace Monorepo with 22 Crates | ✅ Accepted | 2026-03-25 | High | Foundation |
| ADR-002 | Hexagonal Architecture with Port/Adapter Pattern | ✅ Accepted | 2026-03-25 | High | ADR-001 |
| ADR-003 | SQLite as Local-First Storage | ✅ Accepted | 2026-03-25 | High | ADR-002 |
| ADR-004 | SHA-256 Hash-Chained Immutable Audit Log | ✅ Accepted | 2026-03-25 | Medium | ADR-003 |
| ADR-005 | gRPC Service Layer with Tonic + Protobuf | ✅ Accepted | 2026-03-25 | High | ADR-002 |
| ADR-006 | Event Sourcing Pattern for State Reconstruction | ✅ Accepted | 2026-03-26 | Medium | ADR-004 |
| ADR-007 | Trait-Based Plugin Registry Pattern | ✅ Accepted | 2026-03-26 | Medium | ADR-002 |
| ADR-008 | Zero-Copy Serialization with serde + bincode | ✅ Accepted | 2026-03-27 | Medium | Foundation |

**Health:** ✅ 100% (all accepted, no pending)

**Critical Decisions:** ADR-001, ADR-002, ADR-003 form foundation for all other crates

#### AgilePlus (5 ADRs)

| ADR | Title | Status | Impact |
|-----|-------|--------|--------|
| ADR-001 | Rust Workspace Monorepo (22 crates) | ✅ Accepted | High |
| ADR-002 | Hexagonal Architecture + Ports | ✅ Accepted | High |
| ADR-003 | SQLite Local Storage | ✅ Accepted | High |
| ADR-004 | Audit Trail + Event Store | ✅ Accepted | Medium |
| ADR-005 | gRPC Services Layer | ✅ Accepted | Medium |

**Health:** ✅ 100% (all accepted)

**Note:** ADRs largely extend phenotype-infrakit decisions with AgilePlus-specific adaptations

#### platforms/thegent (8 ADRs)

| ADR | Title | Status | Impact |
|-----|-------|--------|--------|
| ADR-001 | Agent Execution Platform Architecture | ✅ Accepted | High |
| ADR-002 | Multi-Language MCP SDKs (Go, Python, TS) | ✅ Accepted | High |
| ADR-003 | Distributed Tracing with Jaeger | ✅ Accepted | Medium |
| ADR-004 | Hotload Capability via WASM/Plugin | ✅ Accepted | Medium |
| ADR-005 | Circuit Breaker Pattern for Resilience | ✅ Accepted | Medium |
| ADR-006 | Resource Isolation (cgroups, namespaces) | ✅ Accepted | High |
| ADR-007 | Observability Telemetry (logs, metrics, traces) | ✅ Accepted | High |
| ADR-008 | Chaos Testing Framework Integration | ✅ Accepted | Medium |

**Health:** ✅ 100% (all accepted)

**Critical Decisions:** ADR-001, ADR-002, ADR-006 define platform core

#### heliosCLI (4 ADRs)

| ADR | Title | Status | Impact |
|-----|-------|--------|--------|
| ADR-001 | CLI Agent Harness Design | 🔧 Draft | High |
| ADR-002 | Sandboxing Strategy (containers, WASM) | 🔧 Draft | High |
| ADR-003 | Plugin Loading Mechanism | ✅ Accepted | Medium |
| ADR-004 | Hotload Agents Runtime | 🔧 Draft | Medium |

**Health:** ⚠️ 50% (2 accepted, 2 draft)

**Blockers:** ADR-001 and ADR-002 must be finalized for Phase 1 completion

---

## ADR Dependency Graph

```
phenotype-infrakit (Foundation)
  ├─ ADR-001: Workspace structure
  ├─ ADR-002: Architecture pattern
  ├─ ADR-003: Storage layer
  ├─ ADR-004: Audit log
  ├─ ADR-005: Service layer
  ├─ ADR-006: Event sourcing
  ├─ ADR-007: Plugins
  └─ ADR-008: Serialization

AgilePlus (extends phenotype-infrakit)
  ├─ ADR-001 → extends phenotype-infrakit ADR-001
  ├─ ADR-002 → extends phenotype-infrakit ADR-002
  └─ ...

platforms/thegent (builds on phenotype-infrakit + AgilePlus)
  ├─ ADR-001: Agent platform (depends on phenotype-infrakit ADR-002)
  ├─ ADR-002: MCP SDKs (depends on phenotype-infrakit ADR-005)
  └─ ...

heliosCLI (depends on all three)
  ├─ ADR-001: Harness (depends on thegent ADR-001)
  └─ ADR-002: Sandboxing (depends on thegent ADR-006)
```

---

## Key ADR Summaries

### ADR-INFRA-001: Rust Workspace Monorepo

**Context:** AgilePlus requires distinct subsystems (domain, CLI, API, gRPC, storage, etc.) with independent versions and test isolation.

**Decision:** Use Cargo workspace with 22 member crates, each scoped to one architectural concern.

**Consequences:**
- ✅ Independent crate compilation and caching
- ✅ Enforced module boundaries via Rust visibility
- ✅ Crate-level feature flags
- ⚠️ Longer cold builds
- ⚠️ Contributors must understand crate graph

**Alternatives Considered:**
- Single monolith (rejected: circular deps)
- Polyrepo per subsystem (rejected: coordination overhead)
- Feature gates in one crate (rejected: no compilation isolation)

### ADR-INFRA-002: Hexagonal Architecture

**Context:** AgilePlus must support swappable backends (SQLite → Postgres, git → GitHub API, etc.) without tight coupling.

**Decision:** Apply hexagonal pattern with ports (traits) and adapters (implementations). Business logic depends only on traits, never on concrete backends.

**Consequences:**
- ✅ Domain testable with in-memory stubs
- ✅ Adapters swappable at runtime
- ✅ New backends require only new adapter crate
- ⚠️ Adds indirection and code volume

### ADR-INFRA-003: SQLite Local-First Storage

**Context:** Solo developers and AI agents need friction-free local operation without requiring Postgres/MySQL.

**Decision:** Use SQLite (bundled) as sole persistence layer. External sync is optional and explicit via content-hash mapping.

**Consequences:**
- ✅ Zero-dependency installation
- ✅ Full offline operation
- ✅ No connection management
- ⚠️ Serialized writes (single writer)
- ⚠️ Sync conflicts require explicit resolution

### ADR-INFRA-004: SHA-256 Hash-Chained Audit Log

**Context:** AgilePlus governance contracts require cryptographic proof of integrity—no tamper detection without hashing.

**Decision:** Every mutation produces Event + AuditEntry, both forming independent SHA-256 hash chains.

**Consequences:**
- ✅ Tamper detection for any entry
- ✅ Full event-sourcing capability
- ✅ Fast append-only writes
- ⚠️ O(n) verification
- ⚠️ Storage grows monotonically

---

## ADR Approval Process

### States

| State | Meaning | Next State |
|-------|---------|-----------|
| Draft | Under review, not yet decided | Review → Accepted/Rejected |
| Review | Submitted for team approval | Accepted/Rejected |
| Accepted | Team agreed, implemented | (no further changes) |
| Superseded | Replaced by newer ADR | (reference successor) |
| Deprecated | No longer used | (mark with reason) |

### Approval Workflow

1. **Propose:** Draft ADR in new branch `specs/agent-<name>-adr-<number>`
2. **Discuss:** Team reviews context, decision, consequences
3. **Decide:** Team votes to accept or request changes
4. **Approve:** Merge to specs/main with "Accepted" status
5. **Implement:** Code follows ADR recommendations

### Review Criteria

**Accepted ADRs must have:**
- ✅ Clear context (why this decision?)
- ✅ Explicit decision (what are we choosing?)
- ✅ Consequences (what are the tradeoffs?)
- ✅ Alternatives (what did we reject and why?)
- ✅ Team consensus (no major objections)

---

## Cross-Repository ADR Inheritance

### phenotype-infrakit (Foundation Layer)

All ADRs in phenotype-infrakit cascade down to dependent repos.

**Inheritance Rules:**
1. Dependent repos MUST follow phenotype-infrakit ADRs
2. Dependent repos MAY extend/refine phenotype-infrakit ADRs
3. Conflicts between ADRs must be escalated to Platform Architect

### AgilePlus (Application Layer)

Extends phenotype-infrakit ADRs with AgilePlus-specific decisions.

**Refinements:**
- ADR-INFRA-001 (Workspace) → ADR-AGILE-001 (22 crate structure + domain logic)
- ADR-INFRA-003 (SQLite) → ADR-AGILE-003 (AgilePlus schema + migrations)

### platforms/thegent (Platform Layer)

Extends both phenotype-infrakit and AgilePlus.

**Dependencies:**
- ADR-INFRA-002 (Hexagonal) → ADR-THEGENT-001 (Agent platform ports)
- ADR-INFRA-005 (gRPC) → ADR-THEGENT-002 (MCP SDKs)
- ADR-AGILE-005 (Services) → ADR-THEGENT-005 (Circuit breakers)

### heliosCLI (Tool Layer)

Depends on all three.

**Dependencies:**
- ADR-THEGENT-001 (Agent platform) → ADR-HELIOS-001 (CLI harness)
- ADR-THEGENT-006 (Isolation) → ADR-HELIOS-002 (Sandboxing)

---

## Decision History

### Accepted Decisions (All Repos)

| Total | Accepted | Draft | Review | Deprecated |
|-------|----------|-------|--------|-----------|
| 25 | 21 (84%) | 4 (16%) | 0 (0%) | 0 (0%) |

**Target:** 100% accepted by 2026-04-11

### Decision Categories

| Category | Count | Examples |
|----------|-------|----------|
| Architecture | 6 | Workspace, hexagonal, ports/adapters |
| Storage | 4 | SQLite, audit log, event sourcing, serialization |
| Services | 3 | gRPC, MCP SDKs, circuit breakers |
| Operations | 5 | Tracing, resource isolation, observability, chaos testing |
| Plugins | 3 | Plugin registry, hotload, agent loading |
| Other | 4 | Sandboxing, etc. |

---

## Maintenance & Updates

### When to Create an ADR

- Architectural decision with repo-wide impact
- Significant technology choice (framework, database, etc.)
- Pattern or convention becoming standard
- Design affecting multiple crates/teams

### When NOT to Create an ADR

- Local implementation details
- Temporary workarounds
- Comments/documentation updates
- Minor refactorings

### ADR Update Process

1. Create branch: `specs/agent-<name>-adr-update-<number>`
2. Update ADR file with `Supersedes: ADR-NNN` reference
3. Mark old ADR as "Superseded"
4. Commit with `Spec-Traces: ADR-NNN`
5. Merge to specs/main

---

## Related Documents

- `docs/reference/SSOT_PHASE1_IMPLEMENTATION_PLAN.md` — SSOT architecture
- `ADR.md` — Detailed ADR files (per-repo)
- `PLAN.md` — Implementation plans (per-repo)
- `FUNCTIONAL_REQUIREMENTS.md` — FRs (per-repo)

---

**Registry Owner:** Architecture Team
**Last Updated:** 2026-04-01
**Next Review:** 2026-04-15
