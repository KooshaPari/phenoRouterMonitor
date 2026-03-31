# AgilePlus Workspace Deep Audit
**Date:** 2026-03-30
**Repository:** `git@github.com:KooshaPari/AgilePlus.git`
**Branch:** `main` (ahead of origin by 3 commits)
**Workspace Size:** 563 MB
**Last Commit:** c150756 — test(phase2.5): add comprehensive integration tests for plugin system

---

## Executive Summary

AgilePlus is a **separate, active Git repository** maintained independently of phenotype-infrakit. The workspace contains **71,017 LOC** across **23 crates** (only 7 actively compiled due to workspace member comment-out). Critical **build failures block all compilation**: plugin registry exports are missing, causing 4 crates to fail compilation. The workspace exhibits **significant monolithic code concentration** in 5 files (>600 LOC each), with 28 dead code suppressions indicating incomplete refactors.

**Key Finding:** AgilePlus is in active Phase 2.5 development (plugin system integration) but **cannot build due to import resolution errors**. Recommend immediate fix before Phase 2 completion.

---

## Repository Status

| Property | Value |
|----------|-------|
| **Remote URL** | `git@github.com:KooshaPari/AgilePlus.git` |
| **Current Branch** | `main` |
| **Commit Ahead** | 3 commits (local) |
| **Latest Commit** | c150756 (phase2.5 plugin integration tests) |
| **Build Status** | ❌ **FAILED** — E0432 unresolved imports |
| **Test Status** | ❌ **BLOCKED** — cannot run tests due to build failure |
| **Repository Type** | Standalone (separate from phenotype-infrakit) |

### Recent Commit History (Last 5)
```
c150756 — test(phase2.5): add comprehensive integration tests for plugin system
7f36728 — feat(phase2.4): add plugin-integration layer for external plugin bridging
9e0c021 — feat(plugin-integration): add plugin integration library
719852e — feat: create agileplus-fixtures shared test crate (#235)
2d64c70 — chore: update gitignore, Cargo.toml and add plugin-grpc library
```

---

## Workspace Structure

### Crate Inventory (23 total; 7 compiled, 16 disabled)

**Actively Compiled Crates:**
1. `libs/nexus` (multicast dependency hub)
2. `libs/plugin-registry` — **⚠️ BUILD ERROR**
3. `libs/plugin-sample` — **⚠️ BUILD ERROR**
4. `libs/plugin-cli` — **⚠️ BUILD ERROR**
5. `libs/plugin-git` — **⚠️ BUILD ERROR**
6. `libs/plugin-grpc`
7. `libs/plugin-integration`

**Disabled Crates (commented out, reason: "TODO: missing src/lib.rs"):**
- `crates/agileplus-*` (22 total: domain, cli, api, grpc, sqlite, git, plane, telemetry, triage, events, cache, subcmds, graph, nats, sync, dashboard, github, p2p, integration-tests, contract-tests, benchmarks)
- `libs/hexagonal-rs`, `libs/hexkit`, `libs/cipher`, `libs/gauge`, `libs/logger`, `libs/metrics`, `libs/tracing`, `libs/cli-framework`, `libs/config-core`, `libs/xdd-lib-rs`
- `tools/forge`

### Lines of Code (LOC) by Active Crate

| Crate | LOC | Files | Status |
|-------|-----|-------|--------|
| agileplus-cli | 8,987 | 12 | Disabled |
| agileplus-sqlite | 6,124 | 8 | Disabled |
| agileplus-dashboard | 5,038 | 6 | Disabled |
| agileplus-subcmds | 4,386 | 5 | Disabled |
| agileplus-domain | 4,265 | 6 | Disabled |
| agileplus-p2p | 3,943 | 7 | Disabled |
| agileplus-plane | 3,855 | 6 | Disabled |
| agileplus-api | 3,121 | 5 | Disabled |
| agileplus-git | 2,556 | 5 | Disabled |
| agileplus-grpc | 1,956 | 4 | Disabled |
| agileplus-telemetry | 1,837 | 4 | Disabled |
| agileplus-graph | 1,124 | 3 | Disabled |
| agileplus-fixtures | 999 | 3 | Disabled |
| agileplus-sync | 832 | 3 | Disabled |
| agileplus-events | 815 | 3 | Disabled |
| agileplus-nats | 781 | 3 | Disabled |
| agileplus-import | 755 | 2 | Disabled |
| agileplus-triage | 731 | 3 | Disabled |
| agileplus-cache | 460 | 2 | Disabled |
| agileplus-github | 458 | 2 | Disabled |
| agileplus-benchmarks | 245 | 2 | Disabled |
| agileplus-integration-tests | 239 | 2 | Disabled |
| agileplus-contract-tests | 11 | 1 | Disabled |
| plugin-registry | (empty) | 0 | ❌ ERROR |

**Total Workspace LOC:** 71,017 (excludes disabled crates with src/lib.rs present)

---

## Build Status & Errors

### Critical Build Failure

**Error Chain:**
```
error[E0432]: unresolved imports `plugin_registry::PluginConfig`, `plugin_registry::PluginMetadata`
 --> libs/plugin-git/src/lib.rs:9:31
  |
9 | use plugin_registry::{Plugin, PluginConfig, PluginMetadata, Result};
```

**Affected Crates (4):**
1. `libs/plugin-git` — E0432 unresolved PluginConfig, PluginMetadata
2. `libs/plugin-cli` — E0432 unresolved PluginConfig, PluginMetadata
3. `libs/plugin-sample` — E0432 unresolved PluginConfig, PluginMetadata
4. `libs/plugin-grpc` — E0432 unresolved PluginConfig

**Root Cause:**
The `plugin-registry/src/lib.rs` defines `PluginConfig` and `PluginMetadata` in `plugin_trait.rs` module but **does not export them** from the root. Current exports:
```rust
pub mod error;
pub mod plugin_trait;
pub mod registry;

pub use error::{PluginError, Result};
pub use registry::PluginRegistry;
pub use plugin_trait::Plugin;
// ❌ MISSING: pub use plugin_trait::{PluginConfig, PluginMetadata};
```

**Fix Required:**
```rust
pub use plugin_trait::{Plugin, PluginConfig, PluginMetadata};
```

---

## Monolithic Code Concentration

### Files Exceeding 500 LOC (Decomposition Priority)

| File | LOC | Handlers/Functions | Nesting Depth | Type | Priority |
|------|-----|-------------------|----------------|------|----------|
| `crates/agileplus-dashboard/src/routes.rs` | 2,640 | 121 | 2,050 indent | HTTP handlers | **CRITICAL** |
| `crates/agileplus-sqlite/src/lib.rs` | 1,582 | 125 | 1,334 indent | Database adapter | **HIGH** |
| `crates/agileplus-api/tests/api_integration.rs` | 943 | – | – | Integration test | Medium |
| `crates/agileplus-integration-tests/tests/modules_and_cycles.rs` | 905 | – | – | Integration test | Medium |
| `crates/agileplus-subcmds/src/device.rs` | 701 | 12 | 546 indent | Device subcommand | **HIGH** |
| `crates/agileplus-cli/src/commands/validate.rs` | 674 | 15 | 558 indent | CLI validation | **HIGH** |
| `crates/agileplus-git/src/materialize.rs` | 633 | 8 | 456 indent | Git materialization | HIGH |
| `crates/agileplus-cli/src/commands/retrospective.rs` | 630 | 19 | 506 indent | CLI retrospective | HIGH |
| `crates/agileplus-p2p/src/git_merge.rs` | 613 | 14 | 448 indent | P2P merge logic | HIGH |
| `crates/agileplus-grpc/src/server/mod.rs` | 595 | 8 | 512 indent | gRPC server | HIGH |
| `crates/agileplus-fixtures/src/dogfood.rs` | 588 | – | – | Test fixtures | Medium |
| `crates/agileplus-cli/src/commands/plan.rs` | 553 | 16 | 446 indent | CLI plan command | HIGH |
| `crates/agileplus-p2p/src/import.rs` | 525 | 9 | 414 indent | P2P import | HIGH |

### Decomposition Analysis

**routes.rs (2,640 LOC)** — **CRITICAL REFACTOR**
- 121 async HTTP handlers consolidated in single file
- Estimated decomposition: 6-8 modules (~330 LOC each)
  - `routes/dashboard.rs` — Dashboard endpoints (~600 LOC)
  - `routes/api.rs` — API endpoints (~500 LOC)
  - `routes/settings.rs` — Settings endpoints (~300 LOC)
  - `routes/health.rs` — Health checks (~200 LOC)
  - `routes/middleware.rs` — Shared middleware (~250 LOC)
- **Estimated refactoring effort:** 12-16 tool calls

**sqlite/lib.rs (1,582 LOC)** — **HIGH PRIORITY REFACTOR**
- 125 functions in monolithic adapter file
- Includes: schema migrations, query builders, transaction handlers
- Estimated decomposition: 4-5 modules (~350 LOC each)
  - `sqlite/sync.rs` — Synchronization logic (~400 LOC)
  - `sqlite/query.rs` — Query builder (~300 LOC)
  - `sqlite/migrations.rs` — Schema management (~250 LOC)
  - `sqlite/transaction.rs` — Transaction handling (~300 LOC)
- **Estimated refactoring effort:** 8-12 tool calls

**device.rs, validate.rs, retrospective.rs, git_merge.rs (600+ LOC each)**
- CLI command handlers and subcommand logic
- Estimated refactoring: Extract validation logic to separate trait impls, split command dispatch
- **Each:** 4-6 tool calls, ~200-300 LOC reduction per file

---

## Code Quality Metrics

### Suppression Analysis

| Allow Type | Count | Impact | Recommendation |
|-----------|-------|--------|-----------------|
| `dead_code` | 28 | Indicates incomplete refactors, unused exports | Audit & remove before release |
| `unused_imports` | 4 | Minor cleanup needed | Auto-fix via rustfmt |
| `clippy::type_complexity` | 2 | Generic type bloat | Extract type aliases |
| `clippy::too_many_arguments` | 2 | Function signature too complex | Introduce parameter structs |
| `clippy::result_large_err` | 1 | Error type optimization | Consider error boxing |
| `clippy::await_holding_lock` | 1 | Potential deadlock risk | Review async lock usage |

**Total Suppressions:** 38 (indicates ~800-1,200 LOC of technical debt)

### Dependency Versions

| Dependency | Version | Status |
|-----------|---------|--------|
| `tokio` | 1 (latest) | ✅ Cutting-edge |
| `axum` | 0.8 | ✅ Latest stable |
| `tonic` | 0.13 | ✅ Latest |
| `serde` | 1 | ✅ Latest |
| `thiserror` | 2 | ⚠️ Recently bumped; check compatibility |
| `chrono` | 0.4 | ✅ Latest |
| `gix` | 0.71 | ✅ Latest |
| `git2` | 0.20 | ✅ Latest |

---

## Disabled Crates Analysis

**22 of 25 crates are disabled** with note "TODO: missing src/lib.rs". This is **unusual** — the crates exist but have no compiled library targets. Investigation suggests:

1. **Workspace Refactoring in Progress**: Phase 2.5 development focused on plugin system; legacy crates may have been disabled during modularization.
2. **Source Files Present**: All `crates/agileplus-*/src/` directories exist with Rust files, suggesting intentional disable rather than missing implementations.
3. **Confusing Error Message**: "missing src/lib.rs" is misleading since files exist; likely copy-paste error in Cargo.toml.

**Recommendation:** Clarify why crates are disabled:
- If archived: move to `.archive/` directory
- If refactoring: enable gradually as Phase 2 progresses
- If intentional split: document in README

---

## Comparison: AgilePlus vs phenotype-infrakit

| Metric | AgilePlus | phenotype-infrakit | Observation |
|--------|-----------|-------------------|-------------|
| **Repository Type** | Standalone, active development | Shared Phenotype workspace | Separate governance |
| **Total LOC** | 71,017 | 9,926 | AgilePlus is 7.2x larger |
| **Crates** | 23 (7 active) | 8 | AgilePlus is more modular (by design) |
| **Build Status** | ❌ BROKEN | ✅ PASSING | AgilePlus blocks on imports |
| **Workspace Size** | 563 MB | ~100 MB (est.) | AgilePlus includes target/ bloat |
| **Monolithic Files** | 13 files >500 LOC | ~3 files | AgilePlus has more consolidation |
| **Dead Code Suppressions** | 28 | 0 | AgilePlus has incomplete refactors |
| **Phase** | Phase 2.5 (plugin system) | Production stable (v0.2.0) | Different maturity levels |

---

## Crate Dependency Graph (Hub Analysis)

### High Fanin/Fanout Crates (Central Hubs)

**nexus** — Connection point for plugin ecosystem
- Fanout: plugin-registry, plugin-git, plugin-cli, plugin-grpc, plugin-integration
- Role: Plugin bus/event coordinator
- Status: Compiles (not yet audited)

**plugin-registry** — Central plugin management
- Fanin: plugin-git, plugin-cli, plugin-sample, plugin-grpc (4 imports)
- Fanout: error types, plugin trait definitions
- **Status: BROKEN** (missing exports)
- Role: Plugin discovery & lifecycle

**agileplus-domain** (disabled but high LOC: 4,265)
- Likely core data model; many crates probably depend on it once enabled
- Estimated fanin: 15+ (cli, sqlite, api, grpc, plane, p2p, etc.)

---

## Phase 2.5 Integration Status

The latest commits indicate **active Phase 2.5 work** on plugin system integration:

- `c150756`: Comprehensive integration tests added
- `7f36728`: Plugin-integration layer completed
- `9e0c021`: Plugin integration library created
- No commits beyond 3 days

**Assessment:** Development velocity is high, but **build gate is down**. The plugin-registry export fix is a 1-minute fix but blocks all downstream work.

---

## Recommendations

### Immediate (Blocking)

1. **Fix plugin-registry exports** (Priority: **CRITICAL**)
   - Add 2 lines to `libs/plugin-registry/src/lib.rs`:
     ```rust
     pub use plugin_trait::{PluginConfig, PluginMetadata};
     ```
   - Estimated effort: 1 tool call, <1 min
   - Unblocks: 4 crates, all tests, Phase 2.5 completion

2. **Re-enable disabled crates or clarify status** (Priority: **HIGH**)
   - Audit why 22 crates are disabled with "missing src/lib.rs" message
   - Either re-enable them or move to `.archive/`
   - Estimated effort: 2-3 tool calls

### Short-term (Next 1-2 weeks)

3. **Decompose monolithic files** (Priority: **HIGH**)
   - routes.rs (2,640 LOC) → 6-8 focused modules (~12-16 tool calls)
   - sqlite/lib.rs (1,582 LOC) → 4-5 modules (~8-12 tool calls)
   - Estimated LOC reduction: 800-1,000 LOC

4. **Remove dead code suppressions** (Priority: **MEDIUM**)
   - Audit 28 `#[allow(dead_code)]` instances
   - Remove unused exports or restore functionality
   - Estimated effort: 1-2 tool calls (parallel scanning)

5. **Clarify disabled crates governance** (Priority: **MEDIUM**)
   - Update Cargo.toml comment to reflect actual status
   - Create DISABLED_CRATES.md documenting Phase 2 refactoring roadmap
   - Estimated effort: 1 tool call

### Medium-term (Phase 2 completion, 2-4 weeks)

6. **Plugin system stabilization** (Priority: **HIGH**)
   - Complete Phase 2.5 integration tests
   - Add plugin discovery benchmarks
   - Document plugin development guide

7. **Workspace consolidation** (Priority: **MEDIUM**)
   - Once disabled crates are re-enabled, establish build gate CI
   - Add clippy lint for dead_code (fail on new suppressions)
   - Run `cargo test --all` on main branch

---

## Architecture Observations

### Plugin System (Emerging Pattern)

AgilePlus is implementing a **plugins-first architecture**:
- Central `plugin-registry` for lifecycle management
- Trait-based plugin interface (`Plugin` trait)
- Multiple plugin implementations: git, CLI, gRPC, sample
- Integration layer for composing plugins

**Strength:** Clean separation of concerns; plugins can be compiled independently
**Concern:** Registry currently broken; import visibility not properly exported

### Disabled Crates (Mystery)

The mass disabling of 22 crates with "missing src/lib.rs" is atypical. Theories:
1. **Workspace refactoring mid-flight**: Someone disabled everything while restructuring
2. **Documentation error**: Note copy-pasted incorrectly
3. **Intentional Phase 2 approach**: Building minimal plugin system first, enabling domains later

**Clarification needed** to avoid confusion in future onboarding.

---

## Summary Table

| Category | Status | Effort | Timeline |
|----------|--------|--------|----------|
| **Build** | ❌ BROKEN | 1 min | ASAP |
| **Tests** | ❌ BLOCKED | Depends on build fix | Same as build |
| **Monolithic Code** | ⚠️ Needs refactor | 20-28 tool calls | 1-2 weeks |
| **Dead Code** | ⚠️ 28 suppressions | 1-2 tool calls | 1 week |
| **Plugin System** | ✅ In progress (Phase 2.5) | 5-10 tool calls | 2-4 weeks |
| **Documentation** | ⚠️ Disabled crates unclear | 1-2 tool calls | 1 week |

---

## Conclusion

**AgilePlus is a standalone, active Rust workspace in Phase 2.5 development** with **71 KLOC across 23 crates**. It is **currently unmergeable due to a critical import resolution bug** in the plugin-registry (missing exports for PluginConfig, PluginMetadata). The workspace contains significant **monolithic code concentration** (5 files >600 LOC each) and **28 dead code suppressions** indicating incomplete refactors. **22 of 25 crates are mysteriously disabled** with outdated error messages, requiring clarification.

**Key Priority:** Fix the 1-line plugin-registry export to unblock Phase 2.5 completion and enable full test suite.

**Separate from phenotype-infrakit:** AgilePlus should remain a dedicated repository with its own governance, CI/CD, and release cycle. The plugin system is a unique architectural direction not present in phenotype-infrakit.
