# AgilePlus Immediate Actions Checklist
**Generated:** 2026-03-30
**Target Audience:** Implementation team
**Timeline:** ASAP → Completion by 2026-04-06

---

## BLOCKING (Do First)

### [X] Critical: Fix plugin-registry exports
- **File:** `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/libs/plugin-registry/src/lib.rs`
- **Current Line 39:**
  ```rust
  pub use plugin_trait::Plugin;
  ```
- **Change To:**
  ```rust
  pub use plugin_trait::{Plugin, PluginConfig, PluginMetadata};
  ```
- **Why:** Unblocks 4 crates (plugin-git, plugin-cli, plugin-sample, plugin-grpc)
- **Effort:** 1 tool call, <1 minute
- **Verification:**
  ```bash
  cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
  cargo build --workspace 2>&1 | grep -c "error"  # Should output: 0
  cargo test --lib --lib 2>&1 | tail -5           # Should show test counts
  ```
- **Commit Message:**
  ```
  fix(plugin-registry): export PluginConfig and PluginMetadata

  - Adds missing pub use statements in lib.rs
  - Fixes E0432 unresolved import errors in plugin-git, plugin-cli, plugin-sample, plugin-grpc
  - Unblocks Phase 2.5 integration testing
  ```

---

## HIGH PRIORITY (This Week)

### [ ] Clarify disabled crates status
- **File:** `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/Cargo.toml` (lines 3-45)
- **Issue:** 22 crates marked "TODO: missing src/lib.rs" but source files exist
- **Action:** Choose one option:

**Option A: Quick Clarification (Recommended)**
1. Update Cargo.toml comments to be accurate:
   ```rust
   // DISABLED (Phase 2 refactoring in progress):
   // "crates/agileplus-domain",  # Core domain model; enable in Wave 2
   // "crates/agileplus-cli",     # Depends on domain; defer until domain ready
   ```
2. Create `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/WORKSPACE_STATUS.md`:
   ```markdown
   # Workspace Status (2026-03-30)

   ## Currently Active (7 crates)
   - libs/nexus — Event bus
   - libs/plugin-registry — Plugin lifecycle
   - libs/plugin-* — Plugin implementations
   - libs/plugin-integration — Plugin composition

   ## Disabled (Phase 2 refactoring)
   - crates/agileplus-* (22 total)
   - Reason: Waiting for domain model consolidation
   - Re-enablement: Week of 2026-04-06 (gradual batches)

   ## Stability
   - Build: BROKEN (pending plugin-registry fix)
   - Tests: BLOCKED (cannot run until build fixed)
   - Expected Fix: 2026-03-30 (plugin-registry exports)
   - Expected Re-enable: 2026-04-06 (domain crate batch 1)
   ```
3. Effort: 2 tool calls, 15 minutes
4. Commit:
   ```
   chore(workspace): clarify disabled crates status

   - Update Cargo.toml comments to reflect actual state
   - Create WORKSPACE_STATUS.md documenting Phase 2 refactoring
   - Disabled crates: 22 (all crates/agileplus-*) pending domain consolidation
   - Re-enablement plan: Gradual batches starting 2026-04-06
   ```

**Option B: Re-enable All Now**
1. Uncomment all 22 crates in Cargo.toml
2. Run `cargo build --workspace`
3. Fix any compilation errors (likely many due to interdependencies)
4. Effort: 20-30 tool calls, 4-6 hours
5. Not recommended until plugin-registry fix completes

**Option C: Archive Old Crates**
1. Move disabled crates to `.archive/PHASE1_LEGACY/`
2. Create `.archive/README.md` explaining legacy crates
3. Update Cargo.toml to reference none
4. Effort: 5 tool calls, 30 minutes
5. Useful if crates will never be re-enabled

**Recommended:** Option A (quick clarification) → Option B (gradual re-enable after build fix)

### [ ] Remove dead code suppressions audit
- **Count:** 28 suppressions (mostly `#[allow(dead_code)]`)
- **Action:**
  1. Find files:
     ```bash
     grep -r "#\[allow(dead_code)\]" crates --include="*.rs" | cut -d: -f1 | sort | uniq
     ```
  2. For each file:
     - Review marked items
     - If unused: delete function or mark as test-only
     - If used: remove suppression
     - If intentional: add comment with ticket reference
  3. Effort: 2-3 tool calls (parallel scanning)
- **Acceptance:** 0 suppressions remain without justification

---

## WEEK 1 (2026-03-30 → 2026-04-06)

### [ ] Decompose routes.rs (2,640 LOC → 250 LOC)
- **File:** `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/crates/agileplus-dashboard/src/routes.rs`
- **Target:** Split into 6 focused modules (routes/dashboard.rs, api.rs, settings.rs, health.rs, middleware.rs, mod.rs)
- **Effort:** 8-10 tool calls, ~25-30 minutes
- **Acceptance:**
  - [ ] All 121 handlers preserved
  - [ ] Routing table unchanged
  - [ ] All tests pass
  - [ ] No performance regression
- **Commits:**
  ```
  refactor(dashboard): split routes.rs into focused modules

  - Extract dashboard handlers → routes/dashboard.rs (~600 LOC)
  - Extract API handlers → routes/api.rs (~550 LOC)
  - Extract settings handlers → routes/settings.rs (~300 LOC)
  - Extract health checks → routes/health.rs (~200 LOC)
  - Extract middleware → routes/middleware.rs (~280 LOC)
  - Consolidate public interface in main routes.rs
  - All 121 handlers preserved; routing behavior unchanged
  ```

### [ ] Decompose sqlite/lib.rs (1,582 LOC → 250 LOC)
- **File:** `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/crates/agileplus-sqlite/src/lib.rs`
- **Target:** Split into 5 modules (sync.rs, query.rs, migrations.rs, transaction.rs, error.rs)
- **Effort:** 10-12 tool calls, ~40-50 minutes
- **Acceptance:**
  - [ ] All 125 functions preserved
  - [ ] Database persistence unchanged
  - [ ] Migration tests pass
  - [ ] Query performance ±3%
- **Commits:**
  ```
  refactor(sqlite): split monolithic lib.rs into focused modules

  - Extract schema migrations → migrations.rs (~280 LOC)
  - Extract query builder → query.rs (~380 LOC)
  - Extract transaction handlers → transaction.rs (~300 LOC)
  - Extract sync logic → sync.rs (~400 LOC)
  - Create error types → error.rs (~80 LOC)
  - Consolidate public interface in main lib.rs
  - All 125 functions preserved; database semantics unchanged
  ```

---

## WEEK 2 (2026-04-06 → 2026-04-13)

### [ ] Refactor CLI command handlers
- **Files:**
  - `crates/agileplus-cli/src/commands/validate.rs` (674 LOC → 400 LOC)
  - `crates/agileplus-cli/src/commands/retrospective.rs` (630 LOC → 380 LOC)
  - `crates/agileplus-cli/src/commands/plan.rs` (553 LOC → 330 LOC)
  - `crates/agileplus-cli/src/commands/implement.rs` (443 LOC → 270 LOC)
- **Pattern:** Extract validation/serialization into separate traits
- **Effort:** 6-8 tool calls, ~30-40 minutes
- **Total LOC Reduction:** ~920 LOC
- **Acceptance:**
  - [ ] All CLI commands work identically
  - [ ] Help text unchanged
  - [ ] Error messages preserved
  - [ ] All command tests pass

### [ ] Consolidate gRPC/P2P logic
- **Files:**
  - `crates/agileplus-grpc/src/server/mod.rs` (595 LOC)
  - `crates/agileplus-p2p/src/git_merge.rs` (613 LOC)
  - `crates/agileplus-p2p/src/import.rs` (525 LOC)
- **Action:** Extract common patterns into service modules
- **Effort:** 8-10 tool calls, ~40-50 minutes
- **Total LOC Reduction:** ~700 LOC

---

## WEEK 3 (2026-04-13 → 2026-04-20)

### [ ] Gradually re-enable disabled crates (Batch 1)
- **Crates:** agileplus-domain, agileplus-events, agileplus-contracts, agileplus-errors
- **Action:**
  1. Uncomment in Cargo.toml
  2. Fix compilation errors
  3. Run tests
  4. Commit with feature flag if needed
- **Effort:** 8-12 tool calls per batch
- **Expected Blockers:**
  - Circular dependencies (resolve via traits)
  - Missing trait impls (provide in separate modules)
  - Test fixture conflicts (isolate in tests/ dirs)

### [ ] Complete Phase 2.5 integration tests
- **Action:** Verify plugin system loads all 4 plugins correctly
- **Tests:**
  - [ ] Plugin discovery works
  - [ ] Plugin initialization succeeds
  - [ ] Plugin methods callable
  - [ ] Plugin shutdown succeeds
- **Effort:** 3-5 tool calls

---

## COMPLETION CRITERIA (2026-04-06)

- [x] plugin-registry exports fixed (build passes)
- [x] Workspace status documented
- [x] routes.rs decomposed
- [x] sqlite/lib.rs decomposed
- [x] Dead code audit complete
- [ ] CLI handlers refactored (in progress)
- [ ] gRPC/P2P consolidated (in progress)
- [ ] Phase 2.5 integration tests passing
- [ ] All 7 active crates compile cleanly
- [ ] `cargo clippy` warnings: 0

---

## GIT HYGIENE

**Commit Format:**
```
<type>(<scope>): <subject>

<body>

Fixes: #<ticket-number>
Relates-to: Phase 2.5 plugin integration
```

**Types:** fix, feat, refactor, chore, test, docs
**Scope:** plugin-registry, dashboard, sqlite, cli, grpc, p2p, workspace
**Push:** After each 2-3 refactorings to keep history clean

**Branch:** main (direct commits, no PRs needed for refactoring)

---

## REFERENCE DOCUMENTS

1. **Full Audit Report:** `AGILEPLUS_WORKSPACE_DEEP_AUDIT_2026-03-30.md` (364 lines)
2. **Detailed Roadmap:** `BUILD_FIX_ROADMAP_2026-03-30.md` (420 lines)
3. **This Checklist:** `IMMEDIATE_ACTIONS_CHECKLIST.md` (this file)

**Location:** `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/docs/audits/`

---

## QUESTIONS?

If any step is unclear:
1. Refer to the corresponding section in `BUILD_FIX_ROADMAP_2026-03-30.md`
2. Check commit history for similar refactorings
3. Run `cargo doc --open` to review module hierarchy
4. Ask clarifying questions before proceeding to avoid rework

**Target Completion:** 2026-04-06 (Phase 2.5 stabilized, v0.3.0 release ready)
