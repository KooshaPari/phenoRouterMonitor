# AgilePlus Workspace Audit Index
**Generated:** 2026-03-30
**Audit Files Location:** `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/docs/audits/`

---

## Quick Navigation

| Document | Size | Read Time | Best For |
|----------|------|-----------|----------|
| **README.md** | 152 lines | 2 min | Overview & navigation |
| **IMMEDIATE_ACTIONS_CHECKLIST.md** | 271 lines | 10 min | Task list, check boxes |
| **AGILEPLUS_WORKSPACE_DEEP_AUDIT_2026-03-30.md** | 364 lines | 15 min | Full technical context |
| **BUILD_FIX_ROADMAP_2026-03-30.md** | 420 lines | 20 min | Implementation details |

---

## Summary

**AgilePlus** is a **standalone, 71 KLOC Rust workspace** with **23 crates (7 active, 16 disabled)** in Phase 2.5 development (plugin system integration). The workspace is currently **unmergeable due to a critical 1-line build error** in plugin-registry exports. **5 files exceed 600 LOC** requiring decomposition, and **28 dead code suppressions** indicate incomplete refactors. The repository should remain **separate from phenotype-infrakit** with independent governance.

---

## Critical Issues (Blocking)

### Issue 1: plugin-registry Missing Exports
- **Error:** `E0432 unresolved imports plugin_registry::{PluginConfig, PluginMetadata}`
- **File:** `libs/plugin-registry/src/lib.rs` line 39
- **Fix:** Add 2 words: `pub use plugin_trait::{Plugin, PluginConfig, PluginMetadata};`
- **Effort:** <1 minute, 1 tool call
- **Unblocks:** 4 crates (plugin-git, plugin-cli, plugin-sample, plugin-grpc)

### Issue 2: 22 Disabled Crates Unclear Status
- **Problem:** Crates marked "TODO: missing src/lib.rs" but files exist
- **Options:** 
  1. Quick clarification (update comments, create WORKSPACE_STATUS.md) — **RECOMMENDED**
  2. Re-enable all crates now
  3. Archive to .archive/ directory
- **Effort:** 15-30 minutes

---

## High Priority (This Week)

1. **Decompose routes.rs** (2,640 → 250 LOC)
   - 121 async handlers in single file
   - Target: 6 focused modules
   - Effort: 8-10 tool calls, 25-30 min

2. **Decompose sqlite/lib.rs** (1,582 → 250 LOC)
   - 125 functions in single file
   - Target: 5 focused modules
   - Effort: 10-12 tool calls, 40-50 min

3. **Audit dead code suppressions**
   - 28 total (mostly #[allow(dead_code)])
   - Action: Remove or justify each
   - Effort: 2-3 tool calls, 15-20 min

---

## Medium Priority (Next 2 Weeks)

1. **CLI command handler refactoring**
   - 4 files >600 LOC (validate.rs, retrospective.rs, plan.rs, implement.rs)
   - Target: Extract validation/formatting logic into traits
   - Effort: 6-8 tool calls, 30-40 min
   - Reduction: ~920 LOC

2. **gRPC/P2P consolidation**
   - 3 files (server/mod.rs 595 LOC, git_merge.rs 613 LOC, import.rs 525 LOC)
   - Target: Extract common patterns into service modules
   - Effort: 8-10 tool calls, 40-50 min
   - Reduction: ~700 LOC

3. **Re-enable disabled crates**
   - Gradual batches (5-6 crates per batch)
   - Verify build + tests after each batch
   - Effort: 8-12 tool calls per batch

---

## Repository Snapshot

| Metric | Value |
|--------|-------|
| **Remote** | `git@github.com:KooshaPari/AgilePlus.git` |
| **Branch** | `main` (ahead of origin by 3) |
| **Latest Commit** | c150756 (phase2.5 integration tests) |
| **Workspace Size** | 563 MB |
| **Total LOC** | 71,017 |
| **Crates** | 23 (7 active, 16 disabled) |
| **Build Status** | ❌ BROKEN (E0432) |
| **Phase** | 2.5 (plugin system) |

---

## Crate Inventory

### Active (7 crates, compilable)
- `libs/nexus` — Event bus
- `libs/plugin-registry` — ⚠️ BUILD ERROR
- `libs/plugin-sample` — ⚠️ BUILD ERROR
- `libs/plugin-cli` — ⚠️ BUILD ERROR
- `libs/plugin-git` — ⚠️ BUILD ERROR
- `libs/plugin-grpc` — Plugin gRPC interface
- `libs/plugin-integration` — Plugin composition

### Disabled (16 crates, unclear status)
- `crates/agileplus-cli` (8,987 LOC) — CLI framework
- `crates/agileplus-sqlite` (6,124 LOC) — Database adapter
- `crates/agileplus-dashboard` (5,038 LOC) — Web dashboard
- `crates/agileplus-subcmds` (4,386 LOC) — Subcommands
- `crates/agileplus-domain` (4,265 LOC) — Domain model
- `crates/agileplus-p2p` (3,943 LOC) — P2P sync
- `crates/agileplus-plane` (3,855 LOC) — Plane integration
- `crates/agileplus-api` (3,121 LOC) — REST API
- `crates/agileplus-git` (2,556 LOC) — Git operations
- `crates/agileplus-grpc` (1,956 LOC) — gRPC server
- `crates/agileplus-telemetry` (1,837 LOC) — Observability
- `crates/agileplus-graph` (1,124 LOC) — Graph queries
- `crates/agileplus-fixtures` (999 LOC) — Test fixtures
- `crates/agileplus-sync` (832 LOC) — Synchronization
- `crates/agileplus-events` (815 LOC) — Event system
- `crates/agileplus-nats` (781 LOC) — NATS bus
- `crates/agileplus-import` (755 LOC) — Data import
- `crates/agileplus-triage` (731 LOC) — Triage logic
- `crates/agileplus-cache` (460 LOC) — Caching layer
- `crates/agileplus-github` (458 LOC) — GitHub integration
- `crates/agileplus-benchmarks` (245 LOC) — Benchmarks
- `crates/agileplus-integration-tests` (239 LOC) — Tests
- `crates/agileplus-contract-tests` (11 LOC) — Tests

---

## Monolithic Code Concentration (Priority Ranked)

| File | LOC | Functions | Priority | Target |
|------|-----|-----------|----------|--------|
| routes.rs | 2,640 | 121 | CRITICAL | 6 modules (~250 each) |
| sqlite/lib.rs | 1,582 | 125 | HIGH | 5 modules (~250 each) |
| device.rs | 701 | 12 | HIGH | 2-3 modules |
| validate.rs | 674 | 15 | HIGH | Trait-based split |
| materialize.rs | 633 | 8 | HIGH | Extract logic |
| retrospective.rs | 630 | 19 | HIGH | Trait-based split |
| git_merge.rs | 613 | 14 | MEDIUM | Merge strategy traits |
| server/mod.rs | 595 | 8 | MEDIUM | Service modules |
| dogfood.rs | 588 | – | MEDIUM | Test fixture cleanup |
| plan.rs | 553 | 16 | HIGH | Trait-based split |

---

## Code Quality Metrics

| Metric | Count | Impact | Action |
|--------|-------|--------|--------|
| `#[allow(dead_code)]` | 28 | ~800-1,200 LOC debt | Audit & remove |
| `#[allow(unused_imports)]` | 4 | Minor | Auto-fix |
| `#[allow(clippy::type_complexity)]` | 2 | Generic bloat | Extract type aliases |
| `#[allow(clippy::too_many_arguments)]` | 2 | Bad function sig | Parameter structs |

---

## Comparison: AgilePlus vs phenotype-infrakit

| Dimension | AgilePlus | phenotype-infrakit | Observation |
|-----------|-----------|-------------------|-------------|
| Type | Standalone | Shared workspace | Different repos |
| Maturity | Phase 2.5 (active) | v0.2.0 (stable) | Different stages |
| Architecture | Plugin-first | Generic crates | Different patterns |
| LOC | 71 KLOC | 10 KLOC | AgilePlus is 7.2x |
| Crates | 23 (7 active) | 8 | AgilePlus more granular |
| Build | BROKEN | PASSING | AgilePlus needs work |
| Tests | BLOCKED | PASSING | Cannot run yet |

**Recommendation:** Keep AgilePlus as separate repository with independent governance, CI/CD, and release cycle.

---

## Refactoring Timeline & Effort

| Phase | Week | Tasks | Effort | Deliverable |
|-------|------|-------|--------|-------------|
| **FIX** | ASAP | Fix plugin-registry | <1 min | Build passes |
| **CLARIFY** | W0 | Disabled crates status | 15 min | WORKSPACE_STATUS.md |
| **REFACTOR-1** | W1 | routes.rs + sqlite | 8-10+10-12 calls | 2 modules split |
| **REFACTOR-2** | W2 | CLI handlers + gRPC | 6-8+8-10 calls | 2 modules split |
| **REFACTOR-3** | W3 | Dead code + re-enable | 2-3+8-12 calls | 28 suppressions ↓ |
| **VERIFY** | W4 | Full test suite | Varies | Phase 2.5 complete |

**Total Effort:** ~35-45 tool calls, 8-12 hours wall clock (parallel execution possible)

**Target Date:** 2026-04-06 (Phase 2.5 complete, v0.3.0 release ready)

---

## Next Steps (In Order)

### Today (2026-03-30)
1. Read `IMMEDIATE_ACTIONS_CHECKLIST.md` (10 min)
2. Fix plugin-registry exports (1 min)
3. Clarify disabled crates status (15 min)
4. Verify build passes: `cargo build --workspace`
5. Commit changes

### This Week (W1)
1. Decompose routes.rs (25-30 min)
2. Decompose sqlite/lib.rs (40-50 min)
3. Audit dead code (15-20 min)
4. Run full test suite

### Next Week (W2-3)
1. Refactor CLI handlers (30-40 min)
2. Consolidate gRPC/P2P (40-50 min)
3. Re-enable crates batch 1
4. Verify tests pass

### By 2026-04-06
- Phase 2.5 complete
- All tests passing
- v0.3.0 release ready

---

## File Locations

All audit documents:
```
/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/docs/audits/
├── README.md
├── AGILEPLUS_WORKSPACE_DEEP_AUDIT_2026-03-30.md
├── BUILD_FIX_ROADMAP_2026-03-30.md
├── IMMEDIATE_ACTIONS_CHECKLIST.md
└── (this file: AUDIT_INDEX.md is in root)
```

---

## Document Cross-References

### README.md
→ Use for: Quick overview, navigation, summary metrics
→ Links to: All other docs

### IMMEDIATE_ACTIONS_CHECKLIST.md
→ Use for: Task list, checkboxes, day-by-day plan
→ Links to: BUILD_FIX_ROADMAP for details

### AGILEPLUS_WORKSPACE_DEEP_AUDIT_2026-03-30.md
→ Use for: Technical context, architecture, full findings
→ Sections: Repository, crates, build, code quality, comparison, recommendations

### BUILD_FIX_ROADMAP_2026-03-30.md
→ Use for: Implementation details, effort estimates, timeline
→ Parts: Critical fix, options, phased decomposition, testing, git hygiene

---

## Questions?

1. **What needs fixing first?** → See `IMMEDIATE_ACTIONS_CHECKLIST.md` (BLOCKING section)
2. **How long will decomposition take?** → See `BUILD_FIX_ROADMAP_2026-03-30.md` (Part 5: Timeline)
3. **Why is the build broken?** → See `AGILEPLUS_WORKSPACE_DEEP_AUDIT_2026-03-30.md` (Build Status & Errors section)
4. **How do I implement each fix?** → See `BUILD_FIX_ROADMAP_2026-03-30.md` (Parts 1-4 with step-by-step instructions)
5. **What about the disabled crates?** → See `BUILD_FIX_ROADMAP_2026-03-30.md` (Part 2: Disabled Crates Analysis with 3 options)

---

**Audit Date:** 2026-03-30
**Next Review:** 2026-04-06 (Phase 2.5 completion checkpoint)
**Owner:** Implementation team
**Status:** ACTIONABLE (Ready for immediate work)
