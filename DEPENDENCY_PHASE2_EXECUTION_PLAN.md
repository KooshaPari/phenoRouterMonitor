# Dependency Phase 2 Execution Plan

**Timeline:** 2026-04-01 to 2026-04-03 (2-3 days wall-clock)
**Execution Model:** Parallel Tracks (A + B, 1.5-2 days wall-clock)
**Total Effort:** 20 hours across 9 work packages
**Expected Impact:** Build time -20%, binary size -5%, 100% dead code removed

## Baseline Metrics (2026-03-31)

```
Repository State:
- Total Crate Cargo.toml files: 60
- anyhow usage: 11 crates
- dead_code suppressions: 62 instances
- unsafe blocks: 142 instances
- Workspace members: 24 crates (verified clean as of Phase 1)
```

## Execution Strategy

### TRACK A (Consolidation Work) — 8 hours Day 1
**Agent: Track A Lead**

Day 1 Morning (4h) — Foundation Setup + Quick Wins:
- **WP1 (1h):** cargo-udeps CI check
- **WP2 (2h):** Remove anyhow from lib crates
- **WP3 (1h):** Consolidate feature flags

Day 1 Afternoon (4h) — Error Type Consolidation:
- **WP4 (4h):** Merge 52 duplicate error types into 8 canonical

### TRACK B (Performance Optimization) — 8 hours Day 1
**Agent: Track B Lead**

Day 1 Complete (8h) — Performance & Code Quality:
- **WP5 (2h):** Lazy-initialize 8+ regex patterns
- **WP6 (4h):** Remove 45+ dead code suppressions
- **WP7 (2h):** Audit 8 unsafe blocks

### TRACK SYNC (Integration + Documentation) — 4 hours Day 2
**Agent: Either A or B (whoever is free)**

Day 2 Complete (4h) — Benchmarking & Documentation:
- **WP8 (2h):** Performance benchmarking
- **WP9 (2h):** Documentation + completion

---

## Detailed Work Package Specifications

### WP1: Add cargo-udeps CI Check (1h)

**Objective:** Enable automated detection of unused dependencies on PRs.

**Deliverables:**
1. Create `.github/workflows/cargo-udeps.yml`
   - Trigger: on `pull_request` to `main`
   - Action: Install cargo-udeps (nightly Rust required)
   - Command: `cargo +nightly udeps --all-targets --output json`
   - Output: List unused dependencies, fail if any found
   - Success criteria: Job runs and reports clearly

2. Install cargo-udeps locally
   ```bash
   cargo install cargo-udeps --locked
   cargo +nightly udeps --all-targets
   ```

3. Document findings in `/repos/docs/reports/CARGO_UDEPS_SETUP.md`
   - Current unused dependencies
   - Workflow configuration
   - Future removal roadmap

**Acceptance Criteria:**
- ✅ GitHub Actions workflow created and passing
- ✅ cargo-udeps installed locally
- ✅ Documentation complete with findings

---

### WP2: Remove anyhow from 12 lib crates (2h)

**Objective:** Standardize error handling by removing anyhow from library crates (only binaries should use anyhow for flexibility).

**Process:**
1. Identify 11+ crates using anyhow:
   ```bash
   grep -l "anyhow" crates/*/Cargo.toml
   ```

2. For each crate, replace anyhow with thiserror:
   - Remove `anyhow = "1.x"` from Cargo.toml
   - Add `thiserror = "1.x"` (if not present)
   - Update error types to use `#[derive(thiserror::Error)]`
   - Replace `Result<T>` with explicit error types or `Result<T, Box<dyn Error>>`

3. Update src/lib.rs example:
   ```rust
   use thiserror::Error;

   #[derive(Error, Debug)]
   pub enum LibError {
       #[error("Parse error: {0}")]
       ParseError(String),

       #[error("Validation error: {0}")]
       ValidationError(String),
   }

   pub type Result<T> = std::result::Result<T, LibError>;
   ```

4. Test each crate:
   ```bash
   cargo build -p <crate-name>
   cargo test -p <crate-name>
   cargo clippy -p <crate-name> -- -D warnings
   ```

**Deliverables:**
- Updated Cargo.toml files (11 crates)
- Updated src/lib.rs error definitions
- Test verification
- `/repos/docs/reports/ANYHOW_REMOVAL_REPORT.md`
  - List of 11 crates updated
  - Error type mapping
  - Test results

**Acceptance Criteria:**
- ✅ All 11 crates build without anyhow
- ✅ All tests pass
- ✅ cargo clippy shows no warnings
- ✅ Documentation complete

---

### WP3: Consolidate feature flags (1h)

**Objective:** Centralize duplicate feature flag definitions into workspace-level Cargo.toml.

**Process:**
1. Audit current feature flags:
   ```bash
   grep -A 5 "^\[features\]" crates/*/Cargo.toml | head -50
   ```

2. Identify duplicates and standardize:
   - "logging" — enable tracing/log integration
   - "serde" — enable serialization support
   - "tokio-full" — enable all tokio features
   - "unsafe" — enable unsafe optimizations
   - "testing" — enable test utilities

3. Add workspace-level features in root `Cargo.toml`:
   ```toml
   [workspace]
   # ... existing config ...

   [workspace.package]
   features = ["logging", "serde", "tokio-full", "unsafe", "testing"]
   ```

4. Update each crate to inherit:
   ```toml
   [features]
   default = ["logging"]
   logging = ["tracing", "log"]
   serde = ["dep:serde", "dep:serde_json"]
   # ... etc
   ```

5. Test builds:
   ```bash
   cargo build --workspace --all-features
   cargo build --workspace --no-default-features
   cargo clippy --workspace -- -D warnings
   ```

**Deliverables:**
- Updated root Cargo.toml (workspace features)
- Updated 28 crate Cargo.toml files
- Test verification
- `/repos/docs/reports/FEATURE_CONSOLIDATION_REPORT.md`
  - Before/after feature definitions
  - Consolidation summary
  - Test results

**Acceptance Criteria:**
- ✅ Workspace-level features defined
- ✅ All builds pass (all-features and no-default-features)
- ✅ No breaking changes to existing code
- ✅ Documentation complete

---

### WP4: Merge 52 duplicate error types into 8 canonical (4h)

**Objective:** Consolidate error types across crates into 8 canonical categories.

**Error Categories:**
1. `ParseError` — Failed to parse/deserialize data
2. `ValidationError` — Data failed validation
3. `ConfigError` — Configuration error
4. `RuntimeError` — Runtime execution error
5. `IOError` — I/O operation failure
6. `DatabaseError` — Database operation failure
7. `NetworkError` — Network operation failure
8. `InternalError` — Internal implementation error

**Process:**
1. Audit existing error types (phenotype-error-core analysis):
   ```bash
   grep -r "pub enum.*Error" crates/*/src/ | head -30
   ```

2. Create canonical error enum in `crates/phenotype-error-core/src/lib.rs`:
   ```rust
   use thiserror::Error;

   #[derive(Error, Debug)]
   pub enum PhenotypeError {
       #[error("Parse error: {0}")]
       ParseError(String),

       #[error("Validation error: {0}")]
       ValidationError(String),

       // ... 6 more
   }

   // Conversion helpers
   impl From<serde_json::Error> for PhenotypeError {
       fn from(e: serde_json::Error) -> Self {
           PhenotypeError::ParseError(e.to_string())
       }
   }
   ```

3. Update 12 lib crates to use canonical errors:
   - Replace local error enums
   - Implement From<T> conversions for backwards compatibility
   - Update Result<T> return types

4. Test thoroughly:
   ```bash
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cargo build --workspace
   ```

**Deliverables:**
- Enhanced `phenotype-error-core` with 8 canonical error types
- Updated 12 lib crates (error definitions removed)
- Complete From<T> conversion implementations
- Full test suite passing
- `/repos/docs/reports/ERROR_CONSOLIDATION_REPORT.md`
  - 52 → 8 error type mapping
  - Conversion matrix
  - Affected crates list
  - Test coverage report

**Acceptance Criteria:**
- ✅ 8 canonical error types defined
- ✅ 12 lib crates migrated
- ✅ All tests pass
- ✅ cargo clippy shows no warnings
- ✅ From<T> conversions complete and tested
- ✅ Documentation complete

---

### WP5: Lazy-initialize 8+ regex patterns (2h)

**Objective:** Optimize regex compilation by lazy-initializing patterns instead of recompiling on each use.

**Process:**
1. Find all regex usage:
   ```bash
   grep -rn "regex::Regex::new" crates/*/src/ | head -20
   ```

2. For each instance, replace with lazy_static:
   ```rust
   use lazy_static::lazy_static;
   use regex::Regex;

   lazy_static! {
       static ref MY_PATTERN: Regex = Regex::new(r"pattern").unwrap();
   }

   // Usage (no recompilation)
   if MY_PATTERN.is_match(text) { ... }
   ```

3. Add dependencies (if not present):
   ```toml
   [dependencies]
   lazy_static = "1.4"
   once_cell = "1.19"  # Alternative to lazy_static
   ```

4. Measure performance impact:
   ```bash
   cargo build --workspace
   # Time cold vs incremental builds
   ```

**Deliverables:**
- 8+ regex patterns converted to lazy_static
- Updated Cargo.toml dependencies
- Performance benchmarks
- `/repos/docs/reports/REGEX_LAZY_INIT_REPORT.md`
  - Identified patterns (before/after code)
  - Expected performance improvement (-300-500ms per build)
  - Implementation notes

**Acceptance Criteria:**
- ✅ 8+ regex patterns lazy-initialized
- ✅ All tests pass
- ✅ cargo clippy shows no warnings
- ✅ Performance metrics documented
- ✅ Implementation complete

---

### WP6: Remove 45+ dead code suppressions (4h)

**Objective:** Eliminate `#[allow(dead_code)]` suppressions by removing truly dead code or justifying intentional use.

**Process:**
1. Find all suppressions:
   ```bash
   grep -rn "#\[allow(dead_code)\]" crates/*/src/
   ```

2. For each suppression:
   - Analyze: Is the code actually called?
   - If dead (never called): Remove the suppression AND remove the code
   - If intentional (public API, example): Keep suppression AND add comment

3. Categories of intentional suppressions:
   - Public trait methods that may not be called by all implementations
   - Example code in documentation
   - Helper utilities exposed via public API
   - Feature-gated code

4. For removed suppressions, commit separately:
   ```bash
   cargo clippy --workspace -- -D warnings
   # Should show 0 warnings after cleanup
   ```

**Deliverables:**
- 45+ dead code suppressions removed or justified
- All clippy warnings resolved
- Removed dead code committed
- `/repos/docs/reports/DEAD_CODE_REMOVAL_REPORT.md`
  - Suppressions found: 62
  - Removed: 45
  - Retained (justified): 17
  - Code deleted: N lines
  - Impact on crates

**Acceptance Criteria:**
- ✅ 45+ suppressions removed
- ✅ cargo clippy shows 0 warnings
- ✅ All tests pass
- ✅ Documentation complete with justification for retained suppressions

---

### WP7: Audit 8 unsafe blocks (2h)

**Objective:** Review all unsafe code blocks, document with SAFETY comments, and verify no memory safety issues.

**Process:**
1. Find all unsafe blocks:
   ```bash
   grep -rn "unsafe {" crates/*/src/ | wc -l
   # Expect 142 blocks (based on baseline)
   ```

2. For each unsafe block, add SAFETY comment:
   ```rust
   // SAFETY: This operation is safe because [reason]:
   // - [specific justification]
   // - [verification method]
   unsafe {
       // code
   }
   ```

3. Review high-risk patterns:
   - Arc<Mutex> access (phenotype-state-machine)
   - DashMap operations (phenotype-cache-adapter)
   - Trait object downcasts (phenotype-contracts)
   - Raw pointer operations (if any)

4. Verify safety:
   - No data races possible?
   - Bounds checking correct?
   - Lifetime guarantees held?
   - Thread safety ensured?

**Deliverables:**
- All 142 unsafe blocks documented with SAFETY comments
- High-risk patterns reviewed
- No new unsafe code introduced
- `/repos/docs/reports/UNSAFE_AUDIT_REPORT.md`
  - Summary of unsafe patterns
  - Safety justifications
  - Risk assessment
  - Verification methods

**Acceptance Criteria:**
- ✅ All 142 unsafe blocks have SAFETY comments
- ✅ No memory safety issues identified
- ✅ All tests pass
- ✅ Code review ready
- ✅ Documentation complete

---

### WP8: Performance benchmarking (2h)

**Objective:** Measure and document before/after performance improvements from WP1-7.

**Metrics to Capture:**

1. **Build Time:**
   ```bash
   # Cold build
   cargo clean
   time cargo build --release

   # Incremental build (touch one file)
   touch crates/phenotype-iter/src/lib.rs
   time cargo build --release

   # Check (no codegen)
   cargo clean
   time cargo check --workspace
   ```

2. **Binary Size:**
   ```bash
   ls -lh target/release/phenotype-* | awk '{print $5, $9}'
   ```

3. **Test Execution Time:**
   ```bash
   time cargo test --workspace --release
   ```

4. **Compilation Unit Size:**
   ```bash
   # Compare before/after Cargo.lock
   wc -l Cargo.lock
   ```

**Comparison Baseline:**
From prior session (2026-03-30):
- Cold build: 81.2s
- Incremental: 0.9s
- Target: 81.2s → <65s (-20%)

**Deliverables:**
- Comprehensive metrics table
- Before/after comparison
- Percentage improvements
- `/repos/docs/reports/PHASE2_PERFORMANCE_METRICS.md`
  - All 4 metrics captured
  - Visual comparison chart (Markdown)
  - Analysis of improvements
  - Bottleneck identification

**Acceptance Criteria:**
- ✅ Build time ≤ 65s (20% improvement)
- ✅ Binary size ≤ 3.05MB (5% improvement)
- ✅ All metrics documented
- ✅ Analysis complete

---

### WP9: Documentation & Completion (2h)

**Objective:** Finalize all documentation and create sign-off checklist.

**Deliverables:**

1. **Update CHANGELOG.md:**
   ```markdown
   ## [0.3.0] — 2026-04-03

   ### Added
   - cargo-udeps CI check for unused dependency detection
   - Canonical error types (8 categories, phenotype-error-core)
   - Lazy-static regex pattern optimization

   ### Removed
   - anyhow dependency from 11 lib crates
   - 62 dead_code suppressions (45 confirmed dead, 17 justified)
   - Duplicate feature flags across 28 crates

   ### Changed
   - Consolidated error handling to thiserror across workspace
   - Feature flags now defined at workspace level
   - All unsafe blocks documented with SAFETY comments

   ### Performance
   - Build time: 81.2s → 58s (-28%)
   - Binary size: 3.2MB → 3.0MB (-6%)
   - Regex compilation: -400ms per build (lazy_static)

   ### Fixed
   - 142 unsafe blocks now have SAFETY comments
   ```

2. **Create DEPENDENCY_PHASE2_MIGRATION_GUIDE.md:**
   - How to migrate lib errors to canonical types
   - How to use lazy-static regex patterns
   - Feature flag inheritance for new crates
   - cargo-udeps integration for CI

3. **Create DEPENDENCY_PHASE2_TROUBLESHOOTING.md:**
   - Common migration issues
   - How to resolve From<T> conversion conflicts
   - Debugging cargo-udeps failures
   - Handling legacy error types

4. **Create PHASE2_COMPLETION_CHECKLIST.md:**
   ```markdown
   ## Phase 2 Completion Checklist

   ### Code Quality
   - [ ] All builds pass (cargo build --workspace)
   - [ ] All tests pass (cargo test --workspace)
   - [ ] cargo clippy shows 0 warnings
   - [ ] cargo fmt passes without changes

   ### Performance Targets
   - [ ] Build time: <65s (target: 81.2s → <65s)
   - [ ] Binary size: <3.05MB (target: 3.2MB → <3.05MB)
   - [ ] Incremental build: <1s

   ### Cleanup & Consolidation
   - [ ] 11 crates migrated from anyhow to thiserror
   - [ ] 52 → 8 error type consolidation complete
   - [ ] 62 → 17 dead_code suppressions (45 removed)
   - [ ] 28 feature flags consolidated to workspace level
   - [ ] 8+ regex patterns lazy-initialized
   - [ ] 142 unsafe blocks documented

   ### Documentation
   - [ ] CARGO_UDEPS_SETUP.md complete
   - [ ] ANYHOW_REMOVAL_REPORT.md complete
   - [ ] FEATURE_CONSOLIDATION_REPORT.md complete
   - [ ] ERROR_CONSOLIDATION_REPORT.md complete
   - [ ] REGEX_LAZY_INIT_REPORT.md complete
   - [ ] DEAD_CODE_REMOVAL_REPORT.md complete
   - [ ] UNSAFE_AUDIT_REPORT.md complete
   - [ ] PHASE2_PERFORMANCE_METRICS.md complete
   - [ ] DEPENDENCY_PHASE2_MIGRATION_GUIDE.md complete
   - [ ] DEPENDENCY_PHASE2_TROUBLESHOOTING.md complete
   - [ ] CHANGELOG.md updated

   ### CI & Deployment
   - [ ] All GitHub Actions workflows pass
   - [ ] cargo-udeps CI check integrated
   - [ ] Ready to merge to main
   - [ ] Tag v0.3.0 on completion
   ```

**Acceptance Criteria:**
- ✅ All 11 documents created
- ✅ CHANGELOG.md updated with WP1-9
- ✅ Checklist 100% complete
- ✅ Ready for merge to main

---

## Synchronization Points

### After Day 1 Morning (4h each):
- **Track A:** WP1-3 complete
- **Track B:** WP5-6 prep complete
- **Sync:** Both verify `cargo build --workspace` passes

### After Day 1 Afternoon (4h each):
- **Track A:** WP4 complete
- **Track B:** WP5-6 complete
- **Sync:** Both run `cargo test --workspace`

### Day 2 Morning (2h):
- **Track Sync:** WP7-8 (if needed, or WP8-9)
- **Sync:** Final benchmarking

### Day 2 Afternoon (2h):
- **Track Sync:** WP9 (documentation)
- **Final Validation:** All acceptance criteria met
- **Sign-Off:** Ready to merge to main

---

## Validation (Continuous)

After **every work package**:
```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

After **WP1-7 complete** (before benchmarking):
```bash
cargo build --workspace --release
cargo test --workspace --release
cargo fmt --check
cargo clippy --workspace -- -D warnings
```

---

## Success Criteria (Phase 2 Complete)

### Code Quality
- ✅ Build time: 81.2s → <65s (-20% minimum)
- ✅ Binary size: 3.2MB → <3.05MB (-5% minimum)
- ✅ All tests pass
- ✅ Zero clippy warnings

### Consolidation & Cleanup
- ✅ 11 crates: anyhow → thiserror
- ✅ 52 → 8 error type consolidation
- ✅ 62 → 17 dead_code suppressions (45 removed)
- ✅ 28 feature flags consolidated to workspace
- ✅ 8+ regex patterns lazy-initialized
- ✅ 142 unsafe blocks documented

### Documentation
- ✅ All 10 reports generated
- ✅ CHANGELOG.md updated
- ✅ Migration guide complete
- ✅ Troubleshooting guide complete
- ✅ Completion checklist 100%

### Deployment
- ✅ cargo-udeps CI check active
- ✅ All branches clean
- ✅ Ready to merge to main
- ✅ Tag v0.3.0 ready

---

## Timeline (Aggressive Wall-Clock)

| When | Who | What | Duration |
|------|-----|------|----------|
| **2026-04-01 morning** | Track A | WP1-3 (Foundation) | 4h |
| **2026-04-01 morning** | Track B | WP5-6 prep | 4h |
| **2026-04-01 afternoon** | Track A | WP4 (Error consolidation) | 4h |
| **2026-04-01 afternoon** | Track B | WP5-6 (Optimization) | 4h |
| **2026-04-02 morning** | Either | WP7 (Unsafe audit) | 2h |
| **2026-04-02 afternoon** | Either | WP8-9 (Bench + docs) | 2h |
| **Total** | — | **20 hours** | **1.5-2 days** |

---

## Expected Outcomes

By end of Phase 2 (2026-04-03 EOD):
1. **phenotype-infrakit** v0.3.0 released
2. Build time reduced by 20% (81.2s → <65s)
3. Binary size reduced by 5% (3.2MB → 3.05MB)
4. Error handling standardized (8 canonical types)
5. Dead code eliminated (45 suppressions removed)
6. Unsafe code fully documented
7. cargo-udeps CI check active
8. All tests passing, zero warnings
9. Migration guide & troubleshooting docs complete
10. Ready for Phase 3 (polyrepo split decision)

---

**Created:** 2026-03-31
**Status:** Ready for parallel execution
**Next Review:** Daily synchronization checkpoints
