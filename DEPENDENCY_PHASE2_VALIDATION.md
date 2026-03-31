# Dependency Phase 2 — Validation Template & Checklist

**Last Updated:** 2026-03-31
**Status:** Ready for execution (2026-04-01)

---

## Pre-Execution Baseline (2026-03-31)

### Repository State
```
Total Crates: 24 production + multi-dependency workspace
Cargo.toml files: 60 (across crates, workspace, examples)
Working directory: /Users/kooshapari/CodeProjects/Phenotype/repos
Canonical branch: main
Current commit: a67fff87b (as of 2026-03-31 morning)
Rust version: 1.93.1 (Homebrew)
Cargo version: 1.93.1 (Homebrew)
```

### Dependency Metrics (Baseline)

| Metric | Count | Target |
|--------|-------|--------|
| anyhow usage (crates) | 11 | 0 |
| dead_code suppressions | 62 | ≤17 (45 removed) |
| unsafe blocks | 142 | Documented (SAFETY comments) |
| Feature flag definitions | Duplicated across 28 Cargo.toml | 1 workspace-level |
| Regex pattern compilations | 8+ at runtime | 1 lazy_static |

### Performance Baseline (Measured 2026-03-30)

| Metric | Baseline | Target | % Improvement |
|--------|----------|--------|----------------|
| Cold build time | 81.2s | <65s | -20% |
| Incremental build | 0.9s | <1.0s | <0% (maintain) |
| Binary size (release) | 3.2 MB | <3.05 MB | -5% |
| Test execution time | ~45s | <40s | -11% |
| Cargo check | ~15s | <12s | -20% |

---

## Validation Checkpoints

### ✅ Daily Checkpoint 1: After WP1-3 (Track A Morning)

**Expected Completion:** 2026-04-01 ~12:00 PM (after 4h)

**Track A Deliverables:**
1. ✅ `.github/workflows/cargo-udeps.yml` created
2. ✅ `cargo-udeps` installed locally
3. ✅ Unused dependencies identified
4. ✅ `docs/reports/CARGO_UDEPS_SETUP.md` complete
5. ✅ Feature flags consolidated (28 Cargo.toml → workspace)
6. ✅ `docs/reports/FEATURE_CONSOLIDATION_REPORT.md` complete
7. ✅ Both WP builds verified to pass

**Validation Commands:**
```bash
# WP1: Verify udeps workflow
ls -la .github/workflows/cargo-udeps.yml
cargo install --list | grep cargo-udeps

# WP3: Verify feature consolidation
grep -A 5 "^\[workspace.package\]" Cargo.toml | head -10
grep -c "^\[features\]" crates/*/Cargo.toml  # Should be <5 (many inherit)

# Build test
cargo build --workspace --all-features
cargo build --workspace --no-default-features
cargo clippy --workspace -- -D warnings
```

**Expected Results:**
- ✅ Workflow file exists
- ✅ cargo-udeps installed
- ✅ Features consolidated
- ✅ Build passes
- ✅ No clippy warnings

---

### ✅ Daily Checkpoint 2: After WP5-6 (Track B Afternoon)

**Expected Completion:** 2026-04-01 ~4:00 PM (after 4h Track B work)

**Track B Deliverables:**
1. ✅ 8+ regex patterns converted to lazy_static
2. ✅ `docs/reports/REGEX_LAZY_INIT_REPORT.md` complete
3. ✅ 45+ dead_code suppressions removed
4. ✅ `docs/reports/DEAD_CODE_REMOVAL_REPORT.md` complete
5. ✅ cargo clippy shows 0 warnings

**Validation Commands:**
```bash
# WP5: Verify regex lazy_static
grep -c "lazy_static!" crates/*/src/*.rs  # Should be 8+
cargo build --workspace
time cargo build --workspace --release  # Measure build time

# WP6: Verify dead code removal
grep -c "#\[allow(dead_code)\]" crates/*/src/*.rs  # Should be ≤17
cargo clippy --workspace -- -D warnings  # Should be 0 warnings

# Full test
cargo test --workspace
```

**Expected Results:**
- ✅ 8+ lazy_static instances
- ✅ ≤17 dead_code suppressions remaining
- ✅ 0 clippy warnings
- ✅ All tests pass
- ✅ Build time trending toward -20% target

---

### ✅ Integration Checkpoint: After WP1-7 Complete (Day 1 EOD)

**Expected Completion:** 2026-04-01 ~5:00 PM (after both tracks complete 8h each)

**Cumulative Deliverables:**
1. ✅ cargo-udeps CI workflow active
2. ✅ 11 crates migrated from anyhow → thiserror (WP2)
3. ✅ Feature flags consolidated (WP3)
4. ✅ 52 → 8 error type consolidation (WP4)
5. ✅ 8+ regex patterns lazy-initialized (WP5)
6. ✅ 45+ dead_code suppressions removed (WP6)
7. ✅ 5 reports generated (WP1-6)

**Validation Commands:**
```bash
# Full build & test
cargo clean
time cargo build --workspace --release  # Measure cold build
cargo test --workspace --release
cargo clippy --workspace -- -D warnings

# Verify consolidations
grep -l "anyhow" crates/*/Cargo.toml  # Should be 0 (or only bin crates)
grep -c "#\[allow(dead_code)\]" crates/*/src/*.rs  # Should be ≤17
cargo +nightly udeps --all-targets 2>/dev/null | tail -20

# Binary size check
ls -lh target/release/phenotype-* | awk '{sum += $5} END {print "Total:", sum}'
```

**Expected Results:**
- ✅ Cold build: <65s (20% improvement from 81.2s baseline)
- ✅ All tests pass
- ✅ 0 clippy warnings
- ✅ anyhow removed from lib crates
- ✅ 52 → 8 error consolidation verified
- ✅ ≤17 dead_code suppressions remaining
- ✅ Binary size: <3.05 MB (5% improvement from 3.2MB baseline)

---

### ✅ Final Checkpoint: After WP8-9 (Day 2 EOD)

**Expected Completion:** 2026-04-02 ~4:00 PM (after 4h final work)

**Final Deliverables:**
1. ✅ 7 consolidated reports (WP1-7)
2. ✅ PHASE2_PERFORMANCE_METRICS.md (WP8)
3. ✅ CHANGELOG.md updated (WP9)
4. ✅ DEPENDENCY_PHASE2_MIGRATION_GUIDE.md (WP9)
5. ✅ DEPENDENCY_PHASE2_TROUBLESHOOTING.md (WP9)
6. ✅ PHASE2_COMPLETION_CHECKLIST.md (WP9)

**Final Validation Commands:**
```bash
# Complete build cycle
cargo clean
cargo build --workspace --release
cargo test --workspace --release
cargo fmt --check
cargo clippy --workspace -- -D warnings

# Verify all metrics
echo "=== Build Time ===" && time cargo check --workspace
echo "=== Binary Size ===" && ls -lh target/release/phenotype-* | tail -5
echo "=== Test Count ===" && cargo test --workspace -- --list | wc -l

# Verify documentation
ls -la docs/reports/*.md docs/reference/DEPENDENCY_PHASE2_*.md
wc -l CHANGELOG.md
```

**Acceptance Criteria (ALL MUST PASS):**
- ✅ Cold build time: 81.2s → <65s (-20%)
- ✅ Binary size: 3.2MB → <3.05MB (-5%)
- ✅ All tests pass (cargo test --workspace)
- ✅ cargo clippy shows 0 warnings
- ✅ cargo fmt passes
- ✅ 11 crates: anyhow → thiserror
- ✅ 52 → 8 error type consolidation
- ✅ 62 → ≤17 dead_code suppressions (45 removed minimum)
- ✅ 8+ regex patterns lazy-initialized
- ✅ 142 unsafe blocks have SAFETY comments (WP7)
- ✅ 6 consolidation reports generated
- ✅ 3 documentation guides complete
- ✅ CHANGELOG.md updated
- ✅ Completion checklist 100%

---

## Failure Recovery Procedures

### If Build Fails

```bash
# 1. Identify the failure
cargo build --workspace 2>&1 | head -50

# 2. Check specific crate
cargo build -p <failing-crate> -- verbose

# 3. Revert the last WP
git diff HEAD~1
git checkout -- <files>

# 4. Report and retry
```

### If Tests Fail

```bash
# 1. Run with backtrace
RUST_BACKTRACE=1 cargo test --workspace -- --nocapture

# 2. Test single crate
cargo test -p <failing-crate> -- --nocapture

# 3. Check if pre-existing
git stash
cargo test --workspace
git stash pop

# 4. Fix or report as blocker
```

### If cargo-udeps Fails

```bash
# May require nightly Rust
rustup update nightly
cargo +nightly install cargo-udeps --force

# Try again
cargo +nightly udeps --all-targets
```

### If Lazy-Static Conflicts Occur

```bash
# May need once_cell instead
cargo add once_cell
# Replace: lazy_static! -> once_cell::sync::Lazy
# Or: lazy_static! -> once_cell::sync::OnceCell
```

---

## Metrics Collection During Execution

### Real-Time Build Metrics

**After WP1 (pre-WP2):**
```bash
time cargo build --workspace --release
# Expected: ~81s (baseline)
```

**After WP3 (post feature consolidation):**
```bash
time cargo build --workspace --release
# Expected: ~79s (minor improvement)
```

**After WP5 (post lazy-static):**
```bash
time cargo build --workspace --release
# Expected: ~75s (regex caching benefit)
```

**After WP6 (post dead code removal):**
```bash
time cargo build --workspace --release
# Expected: ~65s (incremental optimization)
```

### Binary Size Tracking

```bash
# After each major WP
cargo build --workspace --release
du -sh target/release/phenotype-*
# Track cumulative size reduction
```

---

## Daily Validation Commands (Run Each Checkpoint)

```bash
#!/bin/bash
set -e

echo "=== Cargo Build ==="
cargo build --workspace

echo "=== Cargo Test ==="
cargo test --workspace

echo "=== Cargo Clippy ==="
cargo clippy --workspace -- -D warnings

echo "=== Cargo Format Check ==="
cargo fmt --check

echo "=== Metrics ==="
echo "Build artifacts: $(du -sh target | cut -f1)"
echo "Crate count: $(find crates -name 'Cargo.toml' | wc -l)"
echo "anyhow usage: $(grep -l 'anyhow' crates/*/Cargo.toml | wc -l)"
echo "dead_code suppressions: $(grep -r '#\[allow(dead_code)\]' crates/ | wc -l)"

echo "✅ All validation checks passed!"
```

---

## Sign-Off Checklist (WP9 Final)

Use this when WP9 is complete:

```markdown
# Phase 2 Sign-Off

## Code Quality (Must Pass)
- [ ] cargo build --workspace ✅
- [ ] cargo test --workspace ✅
- [ ] cargo clippy --workspace -- -D warnings (0 warnings) ✅
- [ ] cargo fmt --check ✅

## Performance Targets (Must Hit)
- [ ] Build time: <65s (was 81.2s) ✅
- [ ] Binary size: <3.05MB (was 3.2MB) ✅
- [ ] Incremental build: <1.0s ✅

## Consolidations (Must Complete)
- [ ] 11 crates: anyhow → thiserror ✅
- [ ] 52 → 8 error types ✅
- [ ] 62 → ≤17 dead_code suppressions ✅
- [ ] 28 feature flags → workspace level ✅
- [ ] 8+ regex patterns → lazy_static ✅
- [ ] 142 unsafe blocks documented ✅

## Documentation (Must Deliver)
- [ ] CARGO_UDEPS_SETUP.md ✅
- [ ] ANYHOW_REMOVAL_REPORT.md ✅
- [ ] FEATURE_CONSOLIDATION_REPORT.md ✅
- [ ] ERROR_CONSOLIDATION_REPORT.md ✅
- [ ] REGEX_LAZY_INIT_REPORT.md ✅
- [ ] DEAD_CODE_REMOVAL_REPORT.md ✅
- [ ] UNSAFE_AUDIT_REPORT.md ✅
- [ ] PHASE2_PERFORMANCE_METRICS.md ✅
- [ ] DEPENDENCY_PHASE2_MIGRATION_GUIDE.md ✅
- [ ] DEPENDENCY_PHASE2_TROUBLESHOOTING.md ✅
- [ ] CHANGELOG.md updated ✅

## Deployment Ready
- [ ] cargo-udeps CI check active ✅
- [ ] All branches clean (main synced) ✅
- [ ] Tag v0.3.0 prepared ✅
- [ ] Ready for merge ✅

## Final Sign-Off
- **Phase 2 Status:** ✅ COMPLETE
- **Date:** 2026-04-02
- **Signed By:** [Agent Team]
- **Merged to main:** [Yes/No]
```

---

## Next Phase (Phase 3) Entry Criteria

Before starting Phase 3, verify:
- ✅ All Phase 2 acceptance criteria met
- ✅ All reports generated and reviewed
- ✅ Zero outstanding issues
- ✅ Main branch is clean and deployable
- ✅ v0.3.0 tag created
- ✅ Polyrepo split decision researched (for Phase 3)

**Phase 3 Focus:** Architectural decisions (polyrepo vs. monorepo, macros audit, GC optimization)

---

**Created:** 2026-03-31
**Ready:** Yes (awaiting 2026-04-01 execution start)
**Last Reviewed:** 2026-03-31 12:00 PM
