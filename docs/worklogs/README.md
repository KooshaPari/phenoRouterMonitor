# Phenotype Worklogs (2026)

This directory contains detailed audit and research worklogs for the Phenotype ecosystem, focusing on duplication reduction, library extraction (libification), and modernization.

## Core Worklogs

| Log | Purpose | Last Updated | Status |
|---|---|---|---|
| [RESEARCH.md](./RESEARCH.md) | Ecosystem research, 3rd party repos, modernization targets | 2026-03-31 | Wave 118-120 appended |
| [DEPENDENCIES.md](./DEPENDENCIES.md) | Package audit, fork candidates, security provenance | 2026-03-31 | Wave 131-133 appended |
| [DUPLICATION.md](./DUPLICATION.md) | Code duplication hotspots, patterns, libification plans | 2026-03-31 | Wave 92 & 118 appended |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | System architecture, patterns, port hierarchy | 2026-03-30 | Wave 108-112 appended |
| [QUALITY.md](./QUALITY.md) | Code quality, testing, review automation | 2026-03-30 | Wave 131-135 appended |
| [PERFORMANCE.md](./PERFORMANCE.md) | Performance optimization, serialization, concurrency | 2026-03-30 | Wave 136-139 appended |
| [WORK_LOG.md](./WORK_LOG.md) | Master session history and task execution log | 2026-03-30 | Active |

---

## 2026 Modernization Roadmap Summary

### Phase 1: Critical Infrastructure (P0)
- **`phenotype-error-core`**: Consolidate 15+ error enums (~850 LOC savings)
- **`phenotype-config-core`**: Standardize on `figment` + JSON Schema (~650 LOC savings)
- **`phenotype-port-traits`**: Extract traits from `agileplus-domain/src/ports/` (~1,000 LOC)

### Phase 2: Performance & Quality (P1)
- **Serialization**: Adopt `rkyv` for zero-copy event store (~2x perf)
- **Testing**: Add `proptest` and `cargo-mutants` for comprehensive testing
- **Build**: Enable `sccache` for 10x faster CI builds

### Phase 3: Ecosystem Integration (P2)
- **MCP**: Standardize on `mcp-sdk-rust` + `FastMCP v3.0`
- **LLM Routing**: Adopt `LiteLLM` with `stamina` retry
- **CLI**: Standardize on `clap` (Rust) + `typer` (Python)

---

## File Index

| File | Lines | Category | Priority |
|------|-------|----------|----------|
| `ARCHITECTURE.md` | ~2,050 | ARCHITECTURE | P0 |
| `DEPENDENCIES.md` | ~2,750 | DEPENDENCIES | P0 |
| `DUPLICATION.md` | ~4,050 | DUPLICATION | P0 |
| `RESEARCH.md` | ~1,850 | RESEARCH | P1 |
| `QUALITY.md` | ~760 | QUALITY | P1 |
| `PERFORMANCE.md` | ~380 | PERFORMANCE | P1 |
| `GOVERNANCE.md` | ~400 | GOVERNANCE | P1 |
| `UX_DX.md` | ~900 | UX_DX | P2 |
| `INTEGRATION.md` | ~210 | INTEGRATION | P2 |

**Total: ~14,350 lines** (expanded ~2.2x from initial audit)

---

## Resuming Work

Start from [WORK_LOG.md](./WORK_LOG.md) **Resume pointers (2026-03-31)**, then the **P0** items in [DEPENDENCIES.md](./DEPENDENCIES.md) and **Libification Hotspots** in [DUPLICATION.md](./DUPLICATION.md). Session bundles: [20260330-stacked-pr-sbom](../sessions/20260330-stacked-pr-sbom/00_OVERVIEW.md), [phase READMEs](../sessions/20260329-phase2-error-core/README.md). **`platforms/thegent`:** [PLATFORMS_THEGENT.md](../reference/PLATFORMS_THEGENT.md). **Docs deps:** `npm run deps:reinstall` from `docs/`.

## 2026-03-30 Wave 96 Summary

### Completed Actions
- ✅ `phenotype-http-client-core` crate added to workspace
- ✅ Worktree audit completed (33 worktrees tracked)
- ✅ Stale worktree `.worktrees/phench/` cleaned
- ✅ Workspace compiles cleanly
- ✅ Worklog updated with latest findings

### Pending PR Actions
| PR | Status | Worktrees to Prune After Merge |
|----|--------|-------------------------------|
| #278 | Open | add-tests, cli-errors, fix-clippy, fix-event-sourcing, impl-* |

### Next Priority Actions

1. **Finish `git2` → `gix` in `phenotype-git-core`** (workspace already on **gix 0.81**)
2. **Deprecate / merge `phenotype-errors`** toward **`phenotype-error-core`** (ADR + spec)
3. **Create phenotype-async-traits** for unified async patterns (spec-gated)
4. **Fork or wrap `cqrs-es`** for event sourcing foundation (spec-gated)
5. **P0 items** in `DUPLICATION.md` (Wave 92 & 118)
6. **Third-party candidates** in `DEPENDENCIES.md` (Wave 131-133)
7. **Architecture** in `ARCHITECTURE.md` (Wave 108-112)
8. **Quality** in `QUALITY.md` (Wave 131-135)
9. **Performance** in `PERFORMANCE.md` (Wave 136-139)
10. **Research** in `RESEARCH.md` (Wave 118-120)

---

## Key Findings Summary (2026-03-31)

### LOC Reduction Targets
| Area | Current | Target | Savings |
|------|---------|--------|---------|
| Error handling | 15+ error enums | 1 canonical | ~850 LOC |
| Config loading | 8 implementations | 1 canonical | ~800 LOC |
| Git operations | 6 implementations | 1 canonical | ~600 LOC |
| Duplicate state machines | 2 crates | 1 canonical | ~726 LOC |
| Serialization | Manual (JSON) | buf/Protobuf | ~250 LOC |
| **Total** | - | - | **~3,226 LOC** |

### 3rd Party Candidates
| Domain | Candidate | Strategy | Status |
|--------|-----------|----------|--------|
| Event Sourcing | `cqrs-es` | WRAP | Identified |
| Policy Engine | `casbin-rs` / `cedar` | WRAP | Identified |
| Git Ops | `gix` (gitoxide) | ADOPT | P0 (RUSTSEC) |
| Serialization | `rkyv` | ADOPT | Proposed |
| Retry Logic | `backon` / `stamina` | WRAP | Proposed |
| Validation | `nutype` | ADOPT | Proposed |

### Inactive Folder Audit
| Category | Count | Action |
|----------|-------|--------|
| Worktrees to delete | 5+ | After merge review |
| Stashed changes | 10 | Apply or drop |
| Nested duplicate crates | 4 | Remove nested |

### Next Priority Actions
1. **IMMEDIATE**: Migrate `git2` → `gix` (RUSTSEC-2025-0140)
2. **HIGH**: Remove nested duplicate state machine crates
3. **HIGH**: Deprecate `phenotype-errors`, promote `phenotype-error-core`
4. **MEDIUM**: Fork `cqrs-es` for event sourcing foundation
5. **MEDIUM**: Create `phenotype-async-traits` crate

---

_Last updated: 2026-03-31 (Wave 118-134)_
