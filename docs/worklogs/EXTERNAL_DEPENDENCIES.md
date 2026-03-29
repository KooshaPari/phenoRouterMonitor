# External Dependencies Analysis

> Fork/Wrap/Adopt opportunities for external packages and repositories.

---

## Blackbox vs Whitebox Analysis

### Terminology

| Mode | Description | When to Use |
|------|-------------|-------------|
| **Blackbox** | Use as-is, no modifications | Stable, well-maintained deps |
| **Whitebox** | Fork and customize | Need modifications, better devs available |
| **Wrap** | Create adapter/shim around library | Want to isolate from changes |
| **Fork** | Full control, periodic sync | Heavy customization, internal release cycle |

---

## GitHub Starred Repos (2026)

### High Priority Fork/Wrap Candidates

#### 1. `Data-Wise/craft` ⭐ 1

| Property | Value |
|----------|-------|
| Type | Claude Code Plugin |
| Stars | 1 |
| LOC | ~500 (86 commands, 8 agents, 21 skills) |
| Recommendation | **FORK** |
| Why | Full-stack dev toolkit with Claude Code integration |
| Benefit | 500+ LOC savings, proven patterns |

**Use Case:** Extend `thegent` agent capabilities with craft's workflow patterns.

#### 2. `newrelic/*` (multiple repos) ⭐ 400+

| Property | Value |
|----------|-------|
| Type | Observability tooling |
| Stars | 400+ total |
| Repos | `newrelic-cli`, `newrelic-client-go`, `tutone` |
| Recommendation | **WRAP** |
| Why | Observability as code, GraphQL codegen |
| Benefit | 200+ LOC savings |

**Use Case:** Wrap `newrelic-client-go` for Go observability integration.

#### 3. `michen00/invisible-squiggles` ⭐ 3

| Property | Value |
|----------|-------|
| Type | VSCode Extension |
| Stars | 3 |
| Purpose | Distraction-free linter diagnostics |
| Recommendation | **WRAP** |
| Why | Clean UX patterns for diagnostics |

---

## Rust Crates Analysis

### Already Wrapped/Integrated

| Crate | Usage | Mode |
|-------|-------|------|
| `clap` | CLI parsing | Blackbox |
| `tokio` | Async runtime | Blackbox |
| `serde` | Serialization | Blackbox |
| `tracing` | Logging | Blackbox |
| `sqlx` | Database | Blackbox |

### Fork Candidates

| Crate | Why Fork | Effort | Alternative |
|-------|---------|--------|-------------|
| `health_check` | Add async_trait support | Low | Create `agileplus-health` |
| `config-rs` | Custom provenance tracking | Low | Use `libs/config-core` |
| `eventually` | ES Aggregate/Repository traits | Medium | Wrap with custom types |

### Wrap Candidates

| Crate | Wrap For | LOC Savings |
|-------|----------|------------|
| `casbin` | RBAC/ABAC policy | 300+ |
| `temporal-sdk` | Long-running workflows | 500+ |
| `miette` | Pretty diagnostic errors | 100+ |

### Adopt Candidates

| Crate | Why Adopt | Weekly Downloads |
|-------|-----------|------------------|
| `figment` | Multi-source config + provenance | ~300 |
| `indicatif` | Progress bars, spinners | ~100k |
| `command-group` | Signal propagation | Built-in |

---

## JavaScript/npm Packages

### Already Used

| Package | Usage | Mode |
|--------|-------|------|
| `react` | UI framework | Blackbox |
| `zod` | Schema validation | Blackbox |
| `@tanstack/react-query` | Server state | Blackbox |

### Fork Candidates

| Package | Why Fork | Alternative |
|---------|---------|-------------|
| `@temporalio/client` | Workflow orchestration | Wrap with Phenotype types |
| `xstate` | State machines | Wrap with custom patterns |

### Wrap Candidates

| Package | Wrap For | LOC Savings |
|---------|----------|------------|
| `ajv` | JSON Schema validation | 100+ |
| `casbin` | Cross-runtime policy | 200+ |

---

## Python Packages

### Already Used

| Package | Usage | Mode |
|--------|-------|------|
| `pydantic` | Data validation | Blackbox |
| `ruff` | Linting | Blackbox |

### Wrap Candidates

| Package | Wrap For | LOC Savings |
|---------|----------|------------|
| `eventsourcing` | ES patterns | 300+ |
| `temporalio` | Workflow orchestration | 400+ |
| `transitions` | State machine | 100+ |

---

## Internal Fork Candidates (Phenotype)

### From DUPLICATION_AUDIT.md

| Pattern | Current | Recommendation | LOC Savings |
|---------|---------|----------------|-------------|
| Health checks | Manual impl x3 | Fork `health_check` + async_trait | 80+ |
| Error types | 12 types | Create `agileplus-error-core` | 150+ |
| Config loading | TOML/YAML manual | Integrate `libs/config-core` | 200+ |

### Unused Libraries Ready for Migration

| Library | Purpose | Edition | Action |
|---------|---------|---------|--------|
| `hexagonal-rs` | Repository traits | 2021 | Migrate to 2024 |
| `config-core` | Config loading | 2021 | Migrate to 2024 |
| `phenotype-port-interfaces` | Port traits | 2021 | Migrate to 2024 |

---

## Decision Matrix

### When to Fork vs Wrap vs Adopt

```
┌─────────────────────────────────────────────────────────────────┐
│                    Dependency Decision Tree                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Need modifications? ──NO──▶ Blackbox (use as-is)              │
│         │                                                       │
│        YES                                                     │
│         │                                                       │
│  Better devs available? ──NO──▶ Wrap (create adapter)          │
│         │                                                       │
│        YES                                                     │
│         │                                                       │
│  Need full control? ──YES──▶ Fork (periodic sync)             │
│         │                                                       │
│        NO                                                      │
│         │                                                       │
│  Need fast iteration? ──YES──▶ Fork (tight sync)              │
│         │                                                       │
│        NO                                                      │
│         │                                                       │
│  Long-term maintenance? ──YES──▶ Fork (formal sync)            │
│         │                                                       │
│        NO                                                      │
│         │                                                       │
│  ▶ Wrap (lightweight adapter)                                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## LOC Reduction Opportunities

### Total Potential LOC Savings

| Category | Current | Target | Savings |
|----------|---------|--------|---------|
| Fork health_check | 80 | 0 | **80** |
| Create error-core | 150 | 0 | **150** |
| Integrate config-core | 200 | 0 | **200** |
| Wrap temporal-sdk | 500 | 0 | **500** |
| Wrap casbin | 300 | 0 | **300** |
| Wrap eventsourcing | 300 | 0 | **300** |
| **TOTAL** | **1,530** | **0** | **1,530** |

---

_Last updated: 2026-03-29_
