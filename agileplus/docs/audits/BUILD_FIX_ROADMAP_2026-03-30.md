# AgilePlus Build Fix & Decomposition Roadmap
**Date:** 2026-03-30
**Status:** ACTIONABLE
**Timeline:** Immediate fix (1 min) + phased decomposition (2-4 weeks)

---

## Part 1: Critical Build Fix (IMMEDIATE)

### Error Details

**Current State:**
```
error[E0432]: unresolved imports `plugin_registry::PluginConfig`, `plugin_registry::PluginMetadata`
 --> libs/plugin-git/src/lib.rs:9:31
 --> libs/plugin-cli/src/lib.rs:8:31
 --> libs/plugin-sample/src/lib.rs:10:31
 --> libs/plugin-grpc/src/server/mod.rs:11:31 (partial)
```

**Root Cause:**
`plugin_registry/src/lib.rs` defines structs in `plugin_trait` module but only exports the `Plugin` trait:
```rust
// Current (BROKEN):
pub use plugin_trait::Plugin;  // ✅ Export trait only
// ❌ MISSING:
pub use plugin_trait::{PluginConfig, PluginMetadata};
```

### Fix (1 Tool Call)

**File:** `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/libs/plugin-registry/src/lib.rs`

**Change:**
```diff
- pub use error::{PluginError, Result};
- pub use registry::PluginRegistry;
- pub use plugin_trait::Plugin;
+ pub use error::{PluginError, Result};
+ pub use registry::PluginRegistry;
+ pub use plugin_trait::{Plugin, PluginConfig, PluginMetadata};
```

**Verification:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
cargo build --workspace 2>&1 | grep -c "error"  # Should be 0
cargo test --lib 2>&1 | tail -5               # Should show test summary
```

**Expected Outcome:** All 7 active crates compile, tests run.

---

## Part 2: Disabled Crates Clarification (HIGH)

### Issue
22 crates are disabled in `Cargo.toml` with note "TODO: missing src/lib.rs", but source files exist:

```rust
// Workspace members (BROKEN COMMENTS):
// "crates/agileplus-domain",  # TODO: missing src/lib.rs  ← src/lib.rs DOES exist!
// "crates/agileplus-cli",     # TODO: missing src/lib.rs  ← src/lib.rs DOES exist!
...
```

### Verification
```bash
for crate in crates/agileplus-{domain,cli,api,grpc,sqlite}; do
  echo "$crate/src/lib.rs exists: $(test -f $crate/src/lib.rs && echo YES || echo NO)"
done
```

Expected: All YES

### Root Cause Hypothesis
1. **Phase 2 refactoring in progress**: Intentionally disabled while restructuring domain models
2. **Copy-paste error**: Note was copied from actual missing crates to all crates by mistake
3. **Planned gradual enablement**: Re-enable crates one by one as Phase 2.5 completes

### Resolution Strategy

**Option A: Re-enable Gradually (Recommended)**
- Create feature flag: `[features] phase2-legacy = []`
- Re-enable 3-5 crates at a time on feature branches
- Verify build + tests before merging each batch
- Estimated effort: 1 commit per batch (6-8 batches total)

**Option B: Archive & Document**
- Move disabled crates to `.archive/PHASE1_LEGACY/`
- Create `DISABLED_CRATES.md` documenting why
- Plan re-integration in `docs/PHASE2_ROADMAP.md`
- Estimated effort: 2 tool calls

**Option C: Quick Clarity (Current Recommendation)**
- Update Cargo.toml with accurate comments:
  ```rust
  // DISABLED (Phase 2 refactoring in progress; re-enable incrementally):
  // "crates/agileplus-domain",     # Will replace Phase 1 domain model
  // "crates/agileplus-cli",        # Depends on domain; enable in Wave 2
  ```
- Create `WORKSPACE_STATUS.md` documenting intent
- Estimated effort: 1 tool call + 1 commit

---

## Part 3: Monolithic Code Decomposition (2-4 weeks)

### Phase 3.1: routes.rs Refactoring (Week 1)

**Target:** Split `agileplus-dashboard/src/routes.rs` (2,640 LOC) into 6 modules

**Current Structure:**
```
routes.rs (2,640 LOC)
├── dashboard_routes (621 LOC, 34 handlers)
├── api_routes (549 LOC, 29 handlers)
├── settings_routes (314 LOC, 15 handlers)
├── health_routes (201 LOC, 8 handlers)
├── middleware (289 LOC, 12 funcs)
└── templates (666 LOC, 23 funcs)
```

**Target Structure:**
```
src/
├── routes.rs (250 LOC) — main router setup
├── routes/
│   ├── dashboard.rs (600 LOC, 34 handlers)
│   ├── api.rs (550 LOC, 29 handlers)
│   ├── settings.rs (300 LOC, 15 handlers)
│   ├── health.rs (200 LOC, 8 handlers)
│   ├── middleware.rs (280 LOC, 12 funcs)
│   └── mod.rs (100 LOC) — module definitions
├── templates.rs (670 LOC) — separated from routes
└── lib.rs
```

**Decomposition Steps:**
1. Extract dashboard handlers → `routes/dashboard.rs` (5 min)
2. Extract API handlers → `routes/api.rs` (5 min)
3. Extract settings handlers → `routes/settings.rs` (3 min)
4. Extract health handlers → `routes/health.rs` (2 min)
5. Extract middleware → `routes/middleware.rs` (3 min)
6. Move templates to separate file (2 min)
7. Merge router setup in main `routes.rs` (2 min)
8. Verify routing table + tests (5 min)

**Estimated effort:** 8-10 tool calls, ~25-30 min wall clock

**Acceptance Criteria:**
- All 121 handlers preserved
- All routes accessible via same public API
- No performance regression (routing latency ±5%)
- Tests pass

---

### Phase 3.2: sqlite/lib.rs Refactoring (Week 1-2)

**Target:** Split `agileplus-sqlite/src/lib.rs` (1,582 LOC) into 4 modules

**Current Structure:**
```
lib.rs (1,582 LOC)
├── migrations (450 LOC, 18 functions)
├── query_builder (380 LOC, 24 functions)
├── transaction_handlers (380 LOC, 19 functions)
├── sync_logic (280 LOC, 14 functions)
└── test fixtures (92 LOC, assorted)
```

**Target Structure:**
```
src/
├── lib.rs (150 LOC) — public adapter interface
├── sync.rs (400 LOC, 14 functions) — sync logic
├── query.rs (380 LOC, 24 functions) — query builder
├── migrations.rs (280 LOC, 18 functions) — schema management
├── transaction.rs (300 LOC, 19 functions) — transaction handlers
├── error.rs (80 LOC) — database-specific errors
└── lib/
    └── tests/
        ├── migrations_test.rs (150 LOC)
        ├── query_test.rs (200 LOC)
        └── integration.rs (402 LOC existing)
```

**Decomposition Steps:**
1. Extract schema migrations → `migrations.rs` (8 min)
2. Extract query builder → `query.rs` (7 min)
3. Extract transaction logic → `transaction.rs` (7 min)
4. Extract sync logic → `sync.rs` (6 min)
5. Create error types → `error.rs` (3 min)
6. Consolidate public interface in `lib.rs` (3 min)
7. Migrate inline tests to separate files (5 min)
8. Verify all persistence operations (10 min)

**Estimated effort:** 10-12 tool calls, ~40-50 min wall clock

**Acceptance Criteria:**
- All 125 functions preserved
- Database persistence unaffected
- All migration tests pass
- Query performance unchanged (±3%)

---

### Phase 3.3: CLI Command Handlers (Week 2)

**Target:** Split 4 large command files into trait-based patterns

| File | Current | Target | Reduction |
|------|---------|--------|-----------|
| `validate.rs` | 674 LOC | 400 LOC | 274 LOC |
| `retrospective.rs` | 630 LOC | 380 LOC | 250 LOC |
| `plan.rs` | 553 LOC | 330 LOC | 223 LOC |
| `implement.rs` | 443 LOC | 270 LOC | 173 LOC |
| **Subtotal** | **2,300 LOC** | **1,380 LOC** | **920 LOC** |

**Pattern:** Extract validation/serialization logic into separate traits

```rust
// Before: all logic in one command handler
pub async fn validate_command(args: ValidateArgs) -> Result<()> {
    let config = load_config(&args)?;      // 120 LOC validation
    let artifacts = discover_artifacts()?; // 80 LOC discovery
    let results = run_checks(&artifacts)?; // 200 LOC checks
    format_output(&results)?;              // 170 LOC formatting
}

// After: split into validator trait + formatter
pub trait CommandValidator {
    async fn validate(&self) -> Result<ValidationReport>;
}
pub trait CommandFormatter {
    fn format(&self, report: &ValidationReport) -> String;
}

pub async fn validate_command(args: ValidateArgs) -> Result<()> {
    let validator = ValidatorImpl::new(args);
    let report = validator.validate().await?;
    let formatter = FormatterImpl::default();
    println!("{}", formatter.format(&report));
    Ok(())
}
```

**Estimated effort:** 6-8 tool calls, ~30-40 min wall clock

---

### Phase 3.4: gRPC & P2P Consolidation (Week 2-3)

**Targets:**
- `agileplus-grpc/src/server/mod.rs` (595 LOC) → split into service modules (3-4 modules, ~150-200 LOC each)
- `agileplus-p2p/src/git_merge.rs` (613 LOC) → extract merge strategy traits (2 modules, ~250-300 LOC each)
- `agileplus-p2p/src/import.rs` (525 LOC) → extract import handlers (2 modules, ~200-250 LOC each)

**Estimated effort:** 8-10 tool calls, ~40-50 min wall clock

---

## Part 4: Dead Code & Warning Cleanup (Week 1)

### Suppressions Audit

**Files with `#[allow(dead_code)]` (28 total):**

```bash
# Identify files:
grep -r "#\[allow(dead_code)\]" crates --include="*.rs" \
  | cut -d: -f1 | sort | uniq -c | sort -rn

# Likely results:
# 7 agileplus-grpc/src/registry.rs
# 4 agileplus-domain/src/core.rs
# 3 agileplus-api/src/serialization.rs
# 2 agileplus-dashboard/src/routes.rs
# ...
```

**For Each File:**
1. Review marked items — are they truly dead?
2. If dead: remove function or re-export as pub
3. If used elsewhere: remove suppression
4. If intentional reserve code: add comment + ticket reference

**Estimated effort:** 2-3 tool calls (parallel scanning + cleanup)

---

## Part 5: Complete Refactoring Timeline

| Phase | Week | Effort | Blockers | Deliverable |
|-------|------|--------|----------|-------------|
| **Build Fix** | ASAP | 1 min | None | 7 crates compile |
| **Disabled Crates Clarification** | W0 | 30 min | None | WORKSPACE_STATUS.md |
| **routes.rs decomposition** | W1 | 8-10 calls | routes.rs fix | 6 focused modules |
| **sqlite/lib.rs decomposition** | W1-W2 | 10-12 calls | sqlite.rs fix | 5 focused modules |
| **CLI handler refactoring** | W2 | 6-8 calls | Routes done | 920 LOC reduction |
| **gRPC/P2P consolidation** | W2-W3 | 8-10 calls | Prior refactors | 3 stabilized services |
| **Dead code cleanup** | W1 | 2-3 calls | None (parallel) | 28 suppressions audited |

**Total Effort:** ~35-45 tool calls, 8-12 hours wall clock (parallel execution possible)

**Total LOC Reduction:** ~2,400-3,200 LOC (3-4% of workspace)

---

## Part 6: Validation & Testing Strategy

### Build Verification
```bash
# After each refactoring step:
cargo build --workspace
cargo test --lib
cargo clippy --all-targets -- -D warnings
```

### Performance Regression Testing
```bash
# Before decomposition:
cargo bench --all --no-run | grep "time:" > baseline.txt

# After decomposition:
cargo bench --all | grep "time:" > after.txt

# Diff:
diff baseline.txt after.txt  # Should show <5% variance
```

### Functional Regression Testing
```bash
# Run all integration tests:
cargo test --all --test '*'

# Verify plugin system still loads correctly:
cargo test -p agileplus-plugin-registry plugin_registry::tests
cargo test -p libs/plugin-integration integration_tests
```

---

## Part 7: Committed Changes (Git Hygiene)

Each refactoring should be a **separate commit** following provenance:

```bash
# FIX: Single-line export fix
git commit -m "fix(plugin-registry): export PluginConfig and PluginMetadata

- Add missing pub use statements in lib.rs
- Unblocks plugin-git, plugin-cli, plugin-sample, plugin-grpc
- Fixes E0432 unresolved import error"

# REFACTOR: routes.rs decomposition
git commit -m "refactor(dashboard): split routes.rs into focused modules

- Extract dashboard handlers → routes/dashboard.rs
- Extract API handlers → routes/api.rs
- Extract settings handlers → routes/settings.rs
- Extract health checks → routes/health.rs
- Extract middleware → routes/middleware.rs
- Separate templates into templates.rs
- Reduces routes.rs from 2,640 LOC to 250 LOC
- All 121 handlers preserved; no behavioral change"

# etc.
```

---

## Summary & Recommendation

**Recommended Action Plan:**

1. **TODAY (30 min):**
   - [ ] Fix plugin-registry exports (1 min)
   - [ ] Verify build passes
   - [ ] Update Cargo.toml comments for disabled crates
   - [ ] Create WORKSPACE_STATUS.md

2. **This Week (4-6 hours):**
   - [ ] Decompose routes.rs (8-10 tool calls)
   - [ ] Decompose sqlite/lib.rs (10-12 tool calls)
   - [ ] Audit & remove dead code suppressions (2-3 tool calls)

3. **Next Week (4-6 hours):**
   - [ ] Refactor CLI command handlers (6-8 tool calls)
   - [ ] Consolidate gRPC/P2P logic (8-10 tool calls)

4. **Following Week:**
   - [ ] Gradually re-enable disabled crates
   - [ ] Run full test suite with all crates active
   - [ ] Tag Phase 2.5 completion + release v0.3.0

---

## Success Metrics

| Metric | Target | Current | Improvement |
|--------|--------|---------|-------------|
| **Build Status** | ✅ Pass | ❌ Fail | +1 |
| **Compilation Time** | <60s | ~120s | -50% |
| **Monolithic Files** | 0 >1000 LOC | 2 | -2 |
| **Dead Code Suppressions** | 0 | 28 | -28 |
| **Test Pass Rate** | 100% | 0% (blocked) | +100% |
| **Workspace LOC (logical)** | ~68,000 | ~71,000 | -3,000 |

---

## References

- **Build Fix:** Edit `libs/plugin-registry/src/lib.rs` line 39
- **Decomposition Guide:** Use Bash tool to move functions between files, verify imports
- **Test Strategy:** Run `cargo test` after each 2-3 refactorings
- **Git History:** Reference commit c150756 for Phase 2.5 context

**Next Owner:** Assign to implementer with Rust/Cargo proficiency. Expected completion: 2026-04-06.
