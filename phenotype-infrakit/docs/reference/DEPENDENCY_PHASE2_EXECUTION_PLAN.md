# Dependency Phase 2: Execution Plan
## Consolidation, Dead Code Removal & Build Optimization

**Date Created:** 2026-03-31
**Repository:** KooshaPari/phenotype-infrakit
**Scope:** 28 Rust crates + cross-workspace consolidation
**Duration:** 2-3 days (16-24 hours total)
**Status:** 🟡 READY FOR LAUNCH

---

## Executive Summary

This execution plan organizes **11 work packages** across **4 key areas**:

1. **Quick Wins** (4h) — Dead code detection & optional feature removal
2. **Consolidation** (4h) — Duplicate error types & feature flags
3. **Performance Optimizations** (8h) — Lazy-init patterns, unsafe audits, dead code cleanup
4. **Testing & Finalization** (4h) — Performance benchmarking & validation

**Estimated Impact:**
- Incremental build time: **-20% to -30%**
- Binary size (minimal): **-15% to -25%**
- Binary size (full): **-5% to -10%**
- Test execution time: **-10% to -15%**
- Dependencies removed: **8-12 crates**

---

## Work Package Breakdown

### Day 1: Quick Wins & Foundation (8 hours)

#### Morning (4 hours): Quick Wins

**WP1: Add cargo-udeps CI Check (2 hours)**

*Objective:* Identify and flag unused dependencies in CI pipeline

*Tasks:*
1. Install cargo-udeps: `cargo install cargo-udeps`
2. Create `.github/workflows/cargo-udeps.yml` workflow
3. Configure to run on PR creation
4. Add to pre-commit hooks (local)
5. Generate baseline report for all 28 crates

*Files to Create:*
- `.github/workflows/cargo-udeps.yml` — CI workflow
- `scripts/check-unused-deps.sh` — Local check script

*Acceptance Criteria:*
- [ ] cargo-udeps installs without errors
- [ ] CI workflow creates passing check on main
- [ ] Baseline report shows unused deps per crate
- [ ] False positives documented
- [ ] All 28 crates scanned successfully

*Effort:* 2 hours
*Depends On:* None
*Rollback:* Delete workflow files; remove from pre-commit

---

**WP2: Remove anyhow from Library Crates (2 hours)**

*Objective:* Replace anyhow error handling with thiserror in 12 lib crates

*Context:* anyhow is meant for applications, not libraries. Library crates should use thiserror for concrete error types.

*Crates to Refactor:*
1. phenotype-error-core
2. phenotype-config-core
3. phenotype-crypto
4. phenotype-git-core
5. phenotype-health
6. phenotype-http-client-core
7. phenotype-logging
8. phenotype-macros
9. phenotype-state-machine
10. phenotype-validation
11. phenotype-contracts
12. phenotype-process

*Tasks per Crate:*
1. Search for `use anyhow::{Result, anyhow}` imports
2. Replace with proper `thiserror::Error` definitions
3. Convert `anyhow::Error` return types to concrete error types
4. Update any `context()` calls to `.map_err()`
5. Test: `cargo test -p <crate>`

*Files to Modify:*
- `crates/phenotype-error-core/src/lib.rs` — Add ErrorType enum
- `crates/phenotype-config-core/src/lib.rs` — Add ConfigError
- ...12 total crates

*Acceptance Criteria:*
- [ ] Zero anyhow imports remaining in lib crates
- [ ] All error types implement thiserror::Error
- [ ] All tests pass for 12 crates
- [ ] Dependency graph updated
- [ ] No new clippy warnings introduced

*Effort:* 2 hours (12 crates × 10 min average)
*Depends On:* WP1 (for baseline)
*Rollback:* `git checkout -- <modified-files>`

---

#### Afternoon (4 hours): Consolidation

**WP3: Consolidate Optional Features (2 hours)**

*Objective:* Merge redundant feature flags across crates

*Issue:* Multiple crates define similar feature names with different semantics:
- `async` (5 crates)
- `serde` (8 crates)
- `tracing` (6 crates)
- `testing` (4 crates)

*Goals:*
- Standardize feature naming across workspace
- Reduce feature duplication
- Create feature coordination matrix

*Tasks:*
1. Audit `Cargo.toml` files for feature definitions across all crates
2. Identify redundant features:
   - List all features per crate
   - Group by semantic purpose
   - Flag conflicts/duplicates
3. Create standardized feature set:
   - `default` — minimal deps (no async, no serde)
   - `async` — tokio + async-trait
   - `serde` — serde serialization
   - `tracing` — instrumentation
   - `testing` — test fixtures + fakes
   - `full` — all features enabled
4. Update all 28 crate `Cargo.toml` files
5. Verify workspace builds with all combinations

*Files to Create:*
- `docs/reference/FEATURE_FLAGS_COORDINATION.md` — Matrix of all features

*Files to Modify:*
- 28 × `crates/*/Cargo.toml`

*Acceptance Criteria:*
- [ ] All crates use standardized feature names
- [ ] No conflicting feature semantics
- [ ] `cargo build --features=full` succeeds
- [ ] Feature matrix documented
- [ ] No build-time warnings introduced

*Effort:* 2 hours
*Depends On:* WP1
*Rollback:* `git checkout -- crates/*/Cargo.toml`

---

**WP4: Merge Duplicate Error Types (2 hours)**

*Objective:* Consolidate 45+ error enums into 8 canonical types

*Issue:* Multiple crates define similar error types (DbError, ConfigError, ParseError appears 5+ times)

*Consolidation Strategy:*
1. Identify canonical error types in phenotype-error-core:
   - `FileSystemError` (for I/O)
   - `ParseError` (for parsing failures)
   - `ValidationError` (for constraint violations)
   - `ConfigError` (for configuration issues)
   - `RuntimeError` (for execution failures)
   - `NetworkError` (for HTTP/RPC)
   - `CryptoError` (for cryptographic ops)
   - `DatabaseError` (for storage)

2. Per crate, identify duplicates:
   - Search for `enum.*Error`
   - Check for semantic overlap with canonical types
   - Mark for consolidation

3. Migrate 12 crates to use canonical types

4. Move 45+ error definitions to `phenotype-error-core`

*Files to Modify:*
- `crates/phenotype-error-core/src/lib.rs` — Add consolidated error types
- 12 crates' `src/lib.rs` files — Remove duplicate error defs, re-export from error-core

*Acceptance Criteria:*
- [ ] All 45+ duplicate error types removed
- [ ] 8 canonical error types defined in error-core
- [ ] All tests pass with consolidated errors
- [ ] Zero orphaned error definitions
- [ ] Code compiles without clippy warnings

*Effort:* 2 hours
*Depends On:* WP2
*Rollback:* `git checkout -- crates/phenotype-error-core/src/lib.rs` + dependent files

---

### Day 2: Performance Optimizations (8 hours)

#### Full Day: Performance Work

**WP5: Lazy-Initialize Regex Patterns (2 hours)**

*Objective:* Replace compile-time regex with lazy_static/once_cell for 8+ patterns

*Issue:* Some crates compile regex patterns at module load time, even when rarely used

*Crates with Regex Patterns:*
1. phenotype-validation
2. phenotype-string
3. phenotype-git-core
4. phenotype-logging
5. phenotype-process
6. phenotype-http-client-core
7. phenotype-macros (2-3 patterns)
8. phenotype-crypto

*Tasks per Pattern:*
1. Find regex compilation in `src/lib.rs`
2. Wrap with `once_cell::sync::Lazy`:
   ```rust
   static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(...).unwrap());
   ```
3. Update references from `PATTERN.is_match()` to `PATTERN.is_match()`
4. Verify same behavior (unit tests)
5. Benchmark impact: `cargo build --release` time

*Expected Impact per Crate:*
- Initial compilation: +5-10ms overhead
- Total compile time: -50-100ms (deferred initialization)
- Runtime impact: negligible

*Files to Create:*
- `docs/reference/REGEX_LAZY_INIT_AUDIT.md` — Pattern inventory

*Files to Modify:*
- 8 crates' `src/lib.rs` — Lazy static definitions

*Acceptance Criteria:*
- [ ] All regex patterns lazily initialized
- [ ] Unit tests pass (100%)
- [ ] No behavioral changes
- [ ] Benchmarking shows compile-time improvement
- [ ] Runtime perf neutral/improved

*Effort:* 2 hours (8 patterns × 15 min)
*Depends On:* WP1 (baseline)
*Rollback:* `git checkout -- crates/*/src/lib.rs` (regex files)

---

**WP6: Remove Dead Code (4 hours)**

*Objective:* Eliminate 45+ `#[allow(dead_code)]` suppressions and orphaned functions

*Dead Code Categories:*

1. **Legacy test utilities** (600 LOC)
   - Location: `crates/phenotype-test-infra/src/fixtures/`
   - Action: Archive to `.archive/test-fixtures-legacy/`
   - Keep: Only used fixtures referenced in current tests

2. **Unused internal helpers** (300 LOC)
   - In: `phenotype-string`, `phenotype-macros`, `phenotype-logging`
   - Action: Search for `#[allow(dead_code)]`, verify unused, remove
   - Process: `cargo check -p <crate>` to verify cleanup

3. **Deprecated public APIs** (200 LOC)
   - Mark with `#[deprecated]` instead of removing
   - Document migration path in CHANGELOG
   - Remove in next major version

4. **Conditional compilation blocks** (150 LOC)
   - Review `#[cfg(...)]` gated code
   - Remove if guard conditions never true
   - Keep: Platform-specific code (e.g., `#[cfg(unix)]`)

5. **Archived test files** (40 KB)
   - Location: `.archive/tests/`
   - Already identified; audit removal safety
   - Action: Move to `.archive/` (non-versioned)

*Tasks:*
1. Run `cargo clippy --all-targets -- -W dead_code` per crate
2. For each `#[allow(dead_code)]` suppression:
   - Verify it's actually unused
   - Check for tests that reference it
   - Remove suppression + code, or mark `#[deprecated]`
3. Archive files instead of deleting
4. Update CHANGELOG with removals
5. Run full test suite

*Files to Modify:*
- 45+ files across 12 crates
- CHANGELOG.md — Document removals

*Files to Create/Move:*
- `.archive/dead-code/` — Consolidated removals (for reference)

*Acceptance Criteria:*
- [ ] All `#[allow(dead_code)]` reviewed
- [ ] 45+ suppressions eliminated
- [ ] 1,200+ LOC removed
- [ ] All tests pass (100%)
- [ ] Zero clippy dead_code warnings
- [ ] CHANGELOG updated
- [ ] Removed code archived

*Effort:* 4 hours
*Depends On:* WP1, WP4
*Rollback:* `git checkout -- crates/*/src/` + restore from `.archive/`

---

**WP7: Audit Unsafe Blocks (2 hours)**

*Objective:* Review all unsafe code (8 blocks total) for correctness

*Current Unsafe Blocks:*
1. `phenotype-crypto/src/lib.rs` — 2 blocks (crypto operations)
2. `phenotype-process/src/lib.rs` — 2 blocks (signal handling, fork)
3. `phenotype-http-client-core/src/lib.rs` — 1 block (FFI for curl wrapper)
4. `phenotype-async-traits/src/lib.rs` — 1 block (Pin projection)
5. `phenotype-macros/src/lib.rs` — 2 blocks (proc macro reflection)

*Audit Checklist per Block:*
- [ ] Is safety comment accurate?
- [ ] Are invariants documented?
- [ ] Could this be replaced with safe Rust?
- [ ] Are preconditions validated?
- [ ] Is there a test case for unsafe path?

*Tasks:*
1. List all `unsafe` blocks with locations
2. For each block:
   - Review safety comment
   - Check for missing `#[deny(unsafe_code)]` guards
   - Consider safe alternatives
   - Verify test coverage
3. Create inline documentation
4. Recommend refactoring for 2-3 blocks (future work)
5. Generate audit report

*Files to Create:*
- `docs/reference/UNSAFE_CODE_AUDIT_2026-03-31.md`

*Files to Modify:*
- 5 crates with unsafe blocks — Add/improve safety comments

*Acceptance Criteria:*
- [ ] All 8 unsafe blocks documented
- [ ] Safety invariants explained
- [ ] Tests cover unsafe paths
- [ ] No unsafe code eliminated (too risky for now)
- [ ] Audit report published
- [ ] Recommendations for Phase 3 documented

*Effort:* 2 hours
*Depends On:* None
*Rollback:* `git checkout -- crates/*/src/lib.rs` (comments only)

---

### Day 3: Testing & Finalization (4 hours)

#### Full Day: Validation & Documentation

**WP8: Performance Benchmarking (2 hours)**

*Objective:* Measure build time & binary size improvements

*Baseline Metrics (from prior work):*
- Cold build: 81.2s
- Incremental build: 0.9s
- Binary size (release, full features): ~850 MB
- Binary size (release, minimal): ~45 MB

*Benchmarking Tasks:*

1. **Pre-Optimization Baseline** (if not already done):
   ```bash
   cargo clean
   time cargo build --release --workspace
   time cargo build --release --workspace --features=full
   ls -lh target/release/phenotype-* | awk '{sum+=$5} END {print sum " bytes"}'
   ```

2. **After Each Major WP**:
   - Record time after WP2 (anyhow removal)
   - Record time after WP4 (error consolidation)
   - Record time after WP5 (lazy regex)
   - Record time after WP6 (dead code removal)

3. **Test Execution Benchmarking**:
   ```bash
   time cargo test --release --workspace
   # Capture: total time, number of tests
   ```

4. **Dependency Count**:
   ```bash
   cargo tree | grep -c '^[├└]'  # Count direct deps
   cargo tree --all | wc -l       # Count transitive deps
   ```

5. **Create Comparison Report**:
   - Before/After table
   - Per-WP impact
   - Cumulative improvements
   - Cost/benefit analysis

*Files to Create:*
- `docs/reports/DEPENDENCY_PHASE2_BENCHMARK_RESULTS.md`
- `docs/reports/performance-baseline-2026-03-31.json` — Raw metrics

*Acceptance Criteria:*
- [ ] Baseline metrics recorded
- [ ] Post-optimization metrics captured
- [ ] All intermediate checkpoints documented
- [ ] Report shows >15% improvement on primary metrics
- [ ] Comparison visualizations created (table/chart)

*Effort:* 2 hours (mostly automated; ~30 min human time)
*Depends On:* WP2, WP4, WP5, WP6 (completed)
*Rollback:* No rollback needed (measurement-only)

---

**WP9: Documentation & Completion Notes (2 hours)**

*Objective:* Document all changes, decisions, and learnings

*Documentation Tasks:*

1. **CHANGELOG.md Updates** (30 min):
   - Document all removals (dead code)
   - Note dependency reductions
   - List error type consolidations
   - Add feature flag changes

2. **Architecture Decision Records** (30 min):
   - `docs/adr/ADR-015-CONSOLIDATED-ERROR-TYPES.md`
   - `docs/adr/ADR-016-LAZY-REGEX-INITIALIZATION.md`
   - `docs/adr/ADR-017-FEATURE-FLAG-COORDINATION.md`

3. **Completion Report** (30 min):
   - Summary of 11 work packages
   - Metrics impact (build time, binary size, deps)
   - Recommendations for Phase 3
   - Lessons learned

4. **Update Master Documentation** (30 min):
   - Update `ARCHITECTURE.md` with new patterns
   - Update `PLAN.md` with completed phases
   - Add cross-references in `README.md`

*Files to Create:*
- `docs/reports/DEPENDENCY_PHASE2_COMPLETION_REPORT.md` (main deliverable)
- `docs/adr/ADR-015-CONSOLIDATED-ERROR-TYPES.md`
- `docs/adr/ADR-016-LAZY-REGEX-INITIALIZATION.md`
- `docs/adr/ADR-017-FEATURE-FLAG-COORDINATION.md`

*Files to Modify:*
- CHANGELOG.md
- ARCHITECTURE.md
- PLAN.md
- README.md

*Acceptance Criteria:*
- [ ] All ADRs written
- [ ] Completion report published
- [ ] All metrics documented
- [ ] Recommendations for Phase 3 clear
- [ ] No outstanding TODOs
- [ ] Peer review completed

*Effort:* 2 hours
*Depends On:* All prior WPs
*Rollback:* `git checkout -- docs/` (revert doc changes only)

---

## Work Package Summary Table

| WP | Title | Effort | Day | Duration | Depends | Status |
|:---|:------|:------:|:---:|:--------:|:-------:|:------:|
| 1 | cargo-udeps CI | 2h | D1-AM | 2h | — | Pending |
| 2 | Remove anyhow | 2h | D1-AM | 2h | WP1 | Pending |
| 3 | Consolidate features | 2h | D1-PM | 2h | WP1 | Pending |
| 4 | Merge error types | 2h | D1-PM | 2h | WP2 | Pending |
| 5 | Lazy regex | 2h | D2 | 2h | WP1 | Pending |
| 6 | Dead code removal | 4h | D2 | 4h | WP1,WP4 | Pending |
| 7 | Unsafe audit | 2h | D2 | 2h | — | Pending |
| 8 | Benchmarking | 2h | D3 | 2h | WP2-6 | Pending |
| 9 | Documentation | 2h | D3 | 2h | All WPs | Pending |
| **TOTAL** | | **20h** | 2-3d | **20h** | — | **Pending** |

---

## Dependency Graph (DAG)

```
WP1 (cargo-udeps)
├── WP2 (Remove anyhow) ─── WP4 (Merge errors) ─── WP6 (Dead code)
├── WP3 (Consolidate features)                        ↓
├── WP5 (Lazy regex) ────────────────────────────────┤
└── WP7 (Unsafe audit)                               ↓
                                                     WP8 (Benchmarking)
                                                     ↓
                                                     WP9 (Documentation)
```

**Critical Path:** WP1 → WP2 → WP4 → WP6 → WP8 → WP9 (13 hours)
**Parallel Opportunities:** WP3, WP5, WP7 can run simultaneously during D1-PM & D2

---

## Daily Schedule

### Day 1: Foundation & Quick Wins (8 hours)

**Morning (4 hours, 09:00-13:00)**
- 09:00-11:00: WP1 (cargo-udeps) — Solo agent
- 11:00-13:00: WP2 (Remove anyhow) — Solo or pair agent

**Afternoon (4 hours, 14:00-18:00)**
- 14:00-16:00: WP3 (Consolidate features) — Parallel agent
- 14:00-16:00: WP4 (Merge errors) — Main agent (depends on WP2)
- 16:00-18:00: Complete WP3/WP4 reviews + testing

**Checkpoint (End of D1):**
- [ ] All 12 crates migrated away from anyhow
- [ ] Feature flags standardized across 28 crates
- [ ] Error type consolidation complete
- [ ] Build verification passes

---

### Day 2: Performance Optimization (8 hours)

**Full Day (08:00-17:00, 1h lunch)**
- 08:00-10:00: WP5 (Lazy regex) — Solo agent
- 08:00-10:00: WP7 (Unsafe audit) — Parallel agent (independent)
- 10:00-14:00: WP6 (Dead code removal) — Main agent (4h work)
- 14:00-17:00: Testing, verification, intermediate benchmarking

**Parallel Tracks:**
- Track A (Build Optimization): WP5 (regex) → WP6 (dead code)
- Track B (Code Quality): WP7 (unsafe audit)

**Checkpoint (End of D2):**
- [ ] 8+ regex patterns lazily initialized
- [ ] 45+ `#[allow(dead_code)]` suppressions eliminated
- [ ] 1,200+ LOC of dead code removed
- [ ] All unsafe blocks audited & documented
- [ ] Intermediate benchmarks show 10-15% improvement
- [ ] All tests pass

---

### Day 3: Validation & Documentation (4 hours)

**Morning (2 hours, 09:00-11:00)**
- WP8 (Performance Benchmarking) — Solo agent
  - Capture final metrics
  - Generate comparison report
  - Analyze improvements

**Afternoon (2 hours, 14:00-16:00)**
- WP9 (Documentation & Completion) — Solo or pair agent
  - Write ADRs
  - Create completion report
  - Update CHANGELOG/ARCHITECTURE

**Final Checkpoint:**
- [ ] All benchmarks completed & reported
- [ ] All ADRs written
- [ ] Completion report published
- [ ] All metrics documented
- [ ] PR ready for submission

---

## Resource Allocation

### Agent Distribution

**Option A: Sequential (1 Agent)**
- Single agent executes all 9 WPs sequentially
- Duration: 20 hours → 2.5 days
- Simplicity: High
- Risk: Medium (single point of failure)

**Option B: Parallel (Recommended)**
- **Day 1 Morning (Agent A):** WP1 → WP2 (critical path)
- **Day 1 Afternoon (Agent A):** WP4 (depends on WP2)
- **Day 1 Afternoon (Agent B):** WP3 (independent)
- **Day 2 (Agent A):** WP5 → WP6 (critical path)
- **Day 2 (Agent B):** WP7 (independent)
- **Day 3 (Agent A):** WP8 → WP9 (critical path)

**Duration:** 20 hours → 1.5-2 days (wall-clock)
**Agents:** 2 concurrent
**Speedup:** ~3-4x vs sequential

**Option C: Aggressive Parallel (4-5 Agents)**
- All independent WPs in parallel
- WP1: Agent 1 (foundation)
- WP2+3+5+7: 4 agents (parallel after WP1)
- WP4+6: Agents 1+2 (serial chain, higher throughput)
- WP8+9: Agent 1 (validation)

**Duration:** 20 hours → ~3-4 hours (wall-clock)
**Agents:** 4-5 concurrent
**Speedup:** ~5-6x vs sequential

---

## Risk & Mitigation

### Risk Assessment

| Risk | Severity | Probability | Mitigation |
|:-----|:--------:|:-----------:|:-----------|
| Dead code removal breaks tests | High | Low (10%) | Comprehensive test suite; rollback per-crate |
| Feature flag combinations fail | Medium | Medium (20%) | Test matrix; CI validation |
| Unsafe code audit misses issues | Medium | Low (5%) | Peer review; external security audit |
| Performance regression | Low | Low (10%) | Benchmarking after each WP |
| Crate dependencies on removed types | Medium | Medium (15%) | Dependency auditing; cargo check |
| Error type migration incomplete | High | Low (5%) | Scripted migration; compiler errors as guide |

### Rollback Strategy

**Per-WP Rollback (Recommended):**
```bash
# Rollback WP2 (anyhow removal)
git checkout -- crates/phenotype-*/src/lib.rs
cargo test --workspace

# Rollback WP4 (error consolidation)
git checkout -- crates/phenotype-error-core/src/lib.rs
git checkout -- crates/phenotype-*/src/lib.rs
cargo build --workspace

# Rollback WP6 (dead code removal)
git checkout -- crates/*/src/
git restore . --source=HEAD~1  # Restore from prior commit
```

**Full Rollback (Worst Case):**
```bash
git reset --hard HEAD~20  # Reset to before Phase 2 start
git push -f origin main   # Force push (only if necessary)
```

**Prevention:**
- [ ] Tag `phase2-start` before beginning
- [ ] Create branch `wip/phase2-consolidation` for all work
- [ ] Test after each WP completion
- [ ] Merge to main only after final validation

---

## Success Criteria Checklist

### Overall Phase 2 Success
- [ ] All 9 work packages completed
- [ ] Zero test failures across entire workspace
- [ ] Build time improved by >15%
- [ ] Binary size reduced by >5%
- [ ] Dependency count reduced by 8-12 crates
- [ ] Dead code suppressions eliminated (45+ → 0)
- [ ] All changes documented in ADRs
- [ ] Completion report published

### Per-WP Acceptance

**WP1: cargo-udeps**
- [ ] CI workflow passes on main
- [ ] Baseline unused dependency report generated
- [ ] Zero false positives in report

**WP2: Remove anyhow**
- [ ] Zero `use anyhow::` imports in 12 lib crates
- [ ] All 12 crates compile without warnings
- [ ] All tests pass (100%)

**WP3: Consolidate features**
- [ ] 28 crates use standardized feature names
- [ ] Feature matrix documented
- [ ] `cargo build --features=full --release` succeeds

**WP4: Merge errors**
- [ ] 8 canonical error types in error-core
- [ ] 45+ duplicate error defs removed
- [ ] All tests pass with consolidated errors
- [ ] No orphaned error definitions

**WP5: Lazy regex**
- [ ] 8+ regex patterns wrapped in `once_cell::Lazy`
- [ ] All tests pass
- [ ] Compile-time benchmark shows improvement

**WP6: Dead code removal**
- [ ] All `#[allow(dead_code)]` reviewed & removed
- [ ] 1,200+ LOC removed
- [ ] Removed code archived in `.archive/`
- [ ] All tests pass
- [ ] Zero clippy dead_code warnings

**WP7: Unsafe audit**
- [ ] All 8 unsafe blocks documented
- [ ] Safety comments explain invariants
- [ ] Recommendations for Phase 3 documented
- [ ] Audit report published

**WP8: Benchmarking**
- [ ] Before/after metrics captured
- [ ] Per-WP impact measured
- [ ] Report shows cumulative improvements
- [ ] Comparison table/charts created

**WP9: Documentation**
- [ ] 3+ ADRs written
- [ ] CHANGELOG updated
- [ ] Completion report published
- [ ] All metrics documented
- [ ] Recommendations for Phase 3 clear

---

## Time Tracking Template

```markdown
# Phase 2 Time Log (2026-03-31)

## Day 1

### WP1: cargo-udeps (2h)
- Start: 09:00
- End: 11:00
- Actual: 2h 15m
- Notes: ...

### WP2: Remove anyhow (2h)
- Start: 11:00
- End: 13:00
- Actual: 1h 45m
- Notes: ...

### WP3: Consolidate features (2h)
- Start: 14:00
- End: 16:00
- Actual: 2h 30m
- Notes: ...

### WP4: Merge errors (2h)
- Start: 16:00
- End: 18:00
- Actual: 2h 10m
- Notes: ...

**Day 1 Total:** 8h 50m (vs planned 8h)

---

## Day 2

### WP5: Lazy regex (2h)
...

### WP7: Unsafe audit (2h)
...

### WP6: Dead code removal (4h)
...

**Day 2 Total:** ...

---

## Day 3

### WP8: Benchmarking (2h)
...

### WP9: Documentation (2h)
...

**Day 3 Total:** ...

---

## Overall Stats
- Total Planned: 20h
- Total Actual: [TBD]
- Variance: [TBD]
```

---

## Validation Checklist

Before considering Phase 2 complete, verify:

**Build & Compilation:**
- [ ] `cargo build --workspace` succeeds (no errors/warnings)
- [ ] `cargo build --release --workspace` succeeds
- [ ] `cargo build --all-features --release` succeeds
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo fmt --check` passes

**Testing:**
- [ ] `cargo test --workspace` passes (100%)
- [ ] `cargo test --release --workspace` passes
- [ ] `cargo test --all-features --workspace` passes
- [ ] Coverage metrics unchanged (no regression)

**Dependency Analysis:**
- [ ] `cargo tree | grep -c '^[├└]'` (direct deps reduced by 8-12)
- [ ] `cargo udeps` shows zero unused dependencies
- [ ] No circular dependency warnings

**Performance:**
- [ ] Incremental build: -20% vs baseline
- [ ] Cold build: -15% vs baseline
- [ ] Binary size: -5% vs baseline
- [ ] Test execution: -10% vs baseline

**Documentation:**
- [ ] All ADRs written and peer-reviewed
- [ ] CHANGELOG updated with all changes
- [ ] Completion report published
- [ ] All metrics documented
- [ ] Recommendations for Phase 3 clear

---

## Phase 3 Recommendations

### Quick Wins (Future)
1. **Profile regex compilation** — Identify hottest patterns for further optimization
2. **Benchmark serialization** — serde vs alternative formats
3. **Audit proc macros** — Reduce compile-time overhead

### Medium-Term (2-3 weeks)
1. **Extract plugin system** — Implement federation pattern from Phase 2A analysis
2. **Feature flag optimization** — Create binary size report per feature combination
3. **Parallel test execution** — Evaluate nextest for faster CI

### Long-Term (Next Quarter)
1. **Crate extraction** — Move high-value crates to separate repos
2. **Polyrepo federation** — Module Federation for runtime composition
3. **Performance profiling** — Wall-time and memory benchmarks across all crates

---

## Next Steps

1. **Kickoff Meeting (30 min)**
   - Review this plan
   - Assign agents/owners
   - Confirm timeline

2. **Pre-Phase Setup (30 min)**
   - Create branch: `wip/phase2-consolidation`
   - Tag: `phase2-start` on main
   - Set up time tracking
   - Prepare CI workflows

3. **Execute Phase 2** (16-20 hours over 2-3 days)
   - Follow daily schedule
   - Track time and blockers
   - Test after each WP
   - Communicate progress

4. **Validation & Merge (2 hours)**
   - Run full validation checklist
   - Peer review all changes
   - Merge to main
   - Create PR with completion report

5. **Retrospective (30 min)**
   - What went well?
   - What could improve?
   - Recommendations for Phase 3

---

## Document History

| Date | Author | Status | Changes |
|:-----|:-------|:------:|:--------|
| 2026-03-31 | System | DRAFT | Initial execution plan created |
| — | — | READY | Awaiting kickoff |

---

## References

- Dependency Analysis: `/docs/reference/RUST_DEPENDENCY_ANALYSIS_COMPLETE.md`
- Validation Framework: `/docs/reference/DEPENDENCY_PHASE2_VALIDATION.md`
- Phase 2A (Extraction): `/docs/reference/PHASE2_EXECUTION_CHECKLIST.md`
- Architecture: `/ARCHITECTURE.md`
- Roadmap: `/PLAN.md`

---

**Status:** 🟡 READY FOR LAUNCH
**Maintainer:** Phenotype Architecture Team
**Next Review:** Upon Phase 2 completion

