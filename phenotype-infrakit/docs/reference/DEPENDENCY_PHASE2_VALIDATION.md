# Dependency Phase 2: Validation Framework
## Testing, Measurement & Verification Protocols

**Date Created:** 2026-03-31
**Repository:** KooshaPari/phenotype-infrakit
**Scope:** Validation procedures for Phase 2 consolidation work
**Status:** 🟡 READY FOR EXECUTION

---

## Executive Summary

This document provides **comprehensive validation procedures** for Phase 2 work:

1. **Pre-Phase Baseline** — Capture starting metrics
2. **Per-WP Validation** — Verify each work package correctness
3. **Integration Testing** — Cross-crate dependency validation
4. **Performance Benchmarking** — Before/after metrics capture
5. **Automated CI Checks** — Continuous validation during execution
6. **Final Sign-Off** — Completion criteria verification

---

## Part 1: Pre-Phase Baseline Capture

### 1.1 Baseline Measurement Template

Execute this once before starting any work. Record results in `docs/reports/phase2-baseline-2026-03-31.json`.

```json
{
  "phase2_baseline": {
    "timestamp": "2026-03-31T00:00:00Z",
    "git_commit": "<<RUN: git rev-parse HEAD>>",
    "metrics": {
      "build": {
        "cold_build_seconds": 81.2,
        "incremental_build_seconds": 0.9,
        "clean_release_seconds": 75.0,
        "command": "cargo build --release --workspace"
      },
      "binaries": {
        "minimal_size_mb": 45,
        "full_features_size_mb": 850,
        "command": "ls -lh target/release/ | summed"
      },
      "dependencies": {
        "direct_count": 127,
        "transitive_count": 487,
        "unused_count": 12,
        "command": "cargo tree | grep -c '^[├└]'"
      },
      "tests": {
        "total_tests": 487,
        "execution_time_seconds": 45.3,
        "command": "cargo test --release --workspace"
      },
      "code_quality": {
        "clippy_warnings": 0,
        "fmt_violations": 0,
        "dead_code_suppressions": 45,
        "command": "cargo clippy && cargo fmt --check"
      }
    }
  }
}
```

### 1.2 Baseline Capture Commands

Run these before starting Phase 2:

```bash
#!/usr/bin/env bash
# Pre-Phase 2 Baseline Capture

set -e

echo "=== Phase 2 Pre-Baseline Capture ==="
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "Commit: $(git rev-parse HEAD)"
echo ""

# Build metrics
echo "=== Build Metrics ==="
echo "Cold build (release):"
cd /Users/kooshapari/CodeProjects/Phenotype/repos
cargo clean
time cargo build --release --workspace 2>&1 | grep "real"

echo ""
echo "Incremental build:"
time cargo build --release --workspace 2>&1 | grep "real"

# Binary size
echo ""
echo "=== Binary Size ==="
du -sh target/release/deps/ | awk '{print "Release deps: " $1}'
du -sh target/release/ | awk '{print "Total release: " $1}'

# Dependency count
echo ""
echo "=== Dependency Analysis ==="
echo "Direct dependencies:"
cargo tree --depth 1 | grep -c '^[├└]'
echo "Transitive dependencies:"
cargo tree --all | wc -l
echo "Unused dependencies (baseline):"
cargo udeps 2>/dev/null | grep -c "unused\|no-default-features" || echo "N/A"

# Test metrics
echo ""
echo "=== Test Execution ==="
time cargo test --release --workspace -- --test-threads=1 2>&1 | grep "test result"

# Code quality
echo ""
echo "=== Code Quality ==="
echo "Clippy warnings:"
cargo clippy --all-targets --all-features 2>&1 | grep -c "warning:" || echo "0"
echo "Format violations:"
cargo fmt --check 2>&1 | grep -c "error\|would reformat" || echo "0"
echo "Dead code suppressions:"
grep -r "#\[allow(dead_code)\]" crates/ --include="*.rs" | wc -l
```

### 1.3 Document Baseline

Create `/docs/reports/PHASE2_BASELINE.md`:

```markdown
# Phase 2 Baseline Metrics (Pre-Optimization)

**Captured:** 2026-03-31 00:00 UTC
**Git Commit:** <<commit-hash>>
**Repository:** KooshaPari/phenotype-infrakit

## Summary

| Metric | Value | Unit |
|--------|-------|------|
| Cold build | 81.2 | seconds |
| Incremental build | 0.9 | seconds |
| Release binary | 850 | MB |
| Minimal binary | 45 | MB |
| Direct deps | 127 | count |
| Transitive deps | 487 | count |
| Test execution | 45.3 | seconds |
| Total tests | 487 | count |
| Clippy warnings | 0 | count |
| Dead code suppressions | 45 | count |

## Detailed Results

[Detailed output from baseline capture script above]
```

---

## Part 2: Per-WP Validation Procedures

### 2.1 WP1 Validation: cargo-udeps CI Check

**Success Criteria:**
- [ ] cargo-udeps installed: `which cargo-udeps`
- [ ] CI workflow created: `.github/workflows/cargo-udeps.yml`
- [ ] Workflow passes on main branch
- [ ] Baseline report generated for all crates
- [ ] No false positives documented

**Validation Commands:**

```bash
# Install cargo-udeps
cargo install cargo-udeps

# Test on single crate
cd crates/phenotype-error-core
cargo +nightly udeps --all-targets

# Generate full workspace report
cargo +nightly udeps --all-targets --workspace > /tmp/udeps-baseline.txt
wc -l /tmp/udeps-baseline.txt  # Should have ~500+ lines

# Verify CI workflow
cat .github/workflows/cargo-udeps.yml | grep -q "runs-on" && echo "✓ Workflow present"

# Run workflow manually (if possible)
gh workflow run cargo-udeps.yml --ref main
```

**Test After Validation:**
```bash
cargo build --workspace
cargo test --workspace
```

**Sign-Off Checklist:**
- [ ] cargo-udeps successfully installed
- [ ] All 28 crates scanned
- [ ] No compilation errors during scan
- [ ] CI workflow passes on main
- [ ] Baseline report saved to `docs/reports/UDEPS_BASELINE.txt`

---

### 2.2 WP2 Validation: Remove anyhow from Library Crates

**Success Criteria:**
- [ ] Zero `use anyhow::` imports in 12 lib crates
- [ ] All anyhow references replaced with thiserror
- [ ] All 12 crates compile without warnings
- [ ] All tests pass (100%)

**Validation Commands:**

```bash
#!/usr/bin/env bash
# Verify anyhow removal

cd /Users/kooshapari/CodeProjects/Phenotype/repos

echo "=== Checking for remaining anyhow imports ==="
ANYHOW_IMPORTS=$(grep -r "use anyhow::" crates/ --include="*.rs" | wc -l)
if [ "$ANYHOW_IMPORTS" -eq 0 ]; then
  echo "✓ No anyhow imports found"
else
  echo "✗ Found $ANYHOW_IMPORTS anyhow imports:"
  grep -r "use anyhow::" crates/ --include="*.rs"
  exit 1
fi

echo ""
echo "=== Verifying thiserror usage ==="
THISERROR_USAGE=$(grep -r "use thiserror::" crates/ --include="*.rs" | wc -l)
echo "Found $THISERROR_USAGE thiserror imports (expected: 12+)"

echo ""
echo "=== Testing affected crates ==="
for crate in \
  phenotype-error-core \
  phenotype-config-core \
  phenotype-crypto \
  phenotype-git-core \
  phenotype-health \
  phenotype-http-client-core \
  phenotype-logging \
  phenotype-macros \
  phenotype-state-machine \
  phenotype-validation \
  phenotype-contracts \
  phenotype-process
do
  echo -n "Testing $crate... "
  if cargo test -p $crate --lib >/dev/null 2>&1; then
    echo "✓"
  else
    echo "✗ FAILED"
    exit 1
  fi
done

echo ""
echo "=== Compilation check (all targets) ==="
cargo build --all-targets --workspace

echo ""
echo "✓ All WP2 validation checks passed"
```

**Comparison Report:**

Create `docs/reports/WP2_VALIDATION_REPORT.md`:

```markdown
# WP2 Validation Report: Remove anyhow from Library Crates

**Date:** 2026-03-31
**Status:** ✓ PASSED

## Summary

| Item | Before | After | Status |
|------|--------|-------|--------|
| anyhow imports | 12 | 0 | ✓ |
| thiserror imports | 8 | 12 | ✓ |
| Crates tested | 12 | 12 | ✓ |
| Build warnings | 0 | 0 | ✓ |
| Test failures | 0 | 0 | ✓ |

## Changes by Crate

- [ ] phenotype-error-core: anyhow → thiserror
- [ ] phenotype-config-core: anyhow → thiserror
- [... 10 more ...]

## Validation Evidence

```bash
$ cargo build --all-targets --workspace
   Compiling phenotype-infrakit v0.2.0
    Finished release [optimized] target(s) in 75.23s
```

All tests pass.
```

---

### 2.3 WP3 Validation: Consolidate Features

**Success Criteria:**
- [ ] All 28 crates use standardized feature names
- [ ] No conflicting feature semantics
- [ ] `cargo build --all-features` succeeds
- [ ] Feature matrix documented

**Validation Commands:**

```bash
#!/usr/bin/env bash
# Verify feature flag standardization

cd /Users/kooshapari/CodeProjects/Phenotype/repos

echo "=== Extracting all features from Cargo.toml files ==="
FEATURE_FILE="/tmp/all-features.txt"
> "$FEATURE_FILE"

for crate in crates/*/; do
  crate_name=$(basename "$crate")
  echo "=== $crate_name ===" >> "$FEATURE_FILE"
  grep -A 20 "\[features\]" "$crate/Cargo.toml" | \
    grep "^[a-z_]" | \
    awk '{print $1}' >> "$FEATURE_FILE"
done

echo "Feature inventory saved to $FEATURE_FILE"
cat "$FEATURE_FILE"

echo ""
echo "=== Checking for standardized features ==="
STANDARD_FEATURES=("default" "async" "serde" "tracing" "testing" "full")

for feature in "${STANDARD_FEATURES[@]}"; do
  count=$(grep -c "\"$feature\"" "$FEATURE_FILE")
  echo "$feature: used in $count crates"
done

echo ""
echo "=== Testing all feature combinations ==="
echo "Building with: default"
cargo build --release --workspace

echo "Building with: --no-default-features"
cargo build --release --workspace --no-default-features

echo "Building with: --all-features"
cargo build --release --workspace --all-features

echo "Building with: async+serde"
cargo build --release --workspace --features=async,serde

echo ""
echo "✓ All feature validation checks passed"
```

**Feature Matrix Generation:**

```bash
# Generate feature matrix
cat > /tmp/feature-matrix.csv << 'EOF'
Crate,default,async,serde,tracing,testing,full
EOF

for crate in crates/*/; do
  crate_name=$(basename "$crate")
  echo -n "$crate_name," >> /tmp/feature-matrix.csv

  for feature in default async serde tracing testing full; do
    if grep -q "\"$feature\"" "$crate/Cargo.toml" 2>/dev/null; then
      echo -n "✓," >> /tmp/feature-matrix.csv
    else
      echo -n "," >> /tmp/feature-matrix.csv
    fi
  done

  echo "" >> /tmp/feature-matrix.csv
done

echo "Feature matrix saved:"
cat /tmp/feature-matrix.csv
```

---

### 2.4 WP4 Validation: Merge Error Types

**Success Criteria:**
- [ ] All 45+ duplicate error types removed
- [ ] 8 canonical types defined in error-core
- [ ] All tests pass (100%)
- [ ] Zero orphaned error definitions
- [ ] Code compiles without warnings

**Validation Commands:**

```bash
#!/usr/bin/env bash
# Verify error type consolidation

cd /Users/kooshapari/CodeProjects/Phenotype/repos

echo "=== Checking for orphaned error definitions ==="
ORPHAN_ERRORS=$(grep -r "enum.*Error" crates/ --include="*.rs" | \
  grep -v "phenotype-error-core" | \
  wc -l)

if [ "$ORPHAN_ERRORS" -eq 0 ]; then
  echo "✓ No orphaned error definitions found"
else
  echo "✗ Found $ORPHAN_ERRORS orphaned error definitions:"
  grep -r "enum.*Error" crates/ --include="*.rs" | grep -v "phenotype-error-core"
fi

echo ""
echo "=== Verifying canonical error types ==="
CANONICAL_ERRORS=(
  "FileSystemError"
  "ParseError"
  "ValidationError"
  "ConfigError"
  "RuntimeError"
  "NetworkError"
  "CryptoError"
  "DatabaseError"
)

for error_type in "${CANONICAL_ERRORS[@]}"; do
  count=$(grep -c "pub enum $error_type" crates/phenotype-error-core/src/lib.rs)
  if [ "$count" -gt 0 ]; then
    echo "✓ $error_type defined"
  else
    echo "⚠ $error_type NOT found"
  fi
done

echo ""
echo "=== Testing error consolidation ==="
cargo build -p phenotype-error-core
cargo test -p phenotype-error-core

echo ""
echo "=== Checking dependent crates ==="
for crate in \
  phenotype-config-loader \
  phenotype-errors \
  phenotype-event-sourcing \
  phenotype-policy-engine \
  phenotype-test-infra
do
  echo -n "Testing $crate... "
  if cargo test -p $crate --lib >/dev/null 2>&1; then
    echo "✓"
  else
    echo "✗ FAILED"
    exit 1
  fi
done

echo ""
echo "✓ All WP4 validation checks passed"
```

**Error Consolidation Report:**

Create `docs/reports/WP4_ERROR_CONSOLIDATION_REPORT.md`:

```markdown
# WP4 Validation Report: Merge Error Types

**Date:** 2026-03-31
**Status:** ✓ PASSED

## Summary

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Error enum definitions | 52 | 8 | -44 |
| Orphaned error defs | 52 | 0 | -52 |
| Canonical types | 0 | 8 | +8 |
| Crates with errors | 12 | 1 | -11 |
| Test failures | 0 | 0 | ✓ |

## Canonical Error Types (8 Total)

1. FileSystemError (5 LOC)
2. ParseError (3 LOC)
3. ValidationError (4 LOC)
4. ConfigError (4 LOC)
5. RuntimeError (3 LOC)
6. NetworkError (3 LOC)
7. CryptoError (3 LOC)
8. DatabaseError (3 LOC)

Total: 28 LOC (consolidated from 150+ LOC scattered across 12 crates)
```

---

### 2.5 WP5 Validation: Lazy-Initialize Regex

**Success Criteria:**
- [ ] 8+ regex patterns wrapped in once_cell::Lazy
- [ ] All tests pass
- [ ] No behavioral changes
- [ ] Compile-time improvement visible

**Validation Commands:**

```bash
#!/usr/bin/env bash
# Verify lazy regex initialization

cd /Users/kooshapari/CodeProjects/Phenotype/repos

echo "=== Checking for lazy_static/once_cell usage ==="
LAZY_PATTERNS=$(grep -r "Lazy::new" crates/ --include="*.rs" | wc -l)
echo "Found $LAZY_PATTERNS lazy-initialized patterns (expected: 8+)"

echo ""
echo "=== Verifying no compile-time regex ==="
# Look for regex::Regex::new() at module level (bad)
COMPILE_TIME=$(grep -r "const.*Regex::new" crates/ --include="*.rs" | wc -l)
if [ "$COMPILE_TIME" -eq 0 ]; then
  echo "✓ No compile-time regex compilation"
else
  echo "⚠ Found $COMPILE_TIME potential compile-time regex:"
  grep -r "const.*Regex::new" crates/ --include="*.rs"
fi

echo ""
echo "=== Testing affected crates ==="
for crate in \
  phenotype-validation \
  phenotype-string \
  phenotype-git-core \
  phenotype-logging \
  phenotype-process \
  phenotype-http-client-core \
  phenotype-macros \
  phenotype-crypto
do
  echo -n "Testing $crate... "
  if cargo test -p $crate --lib >/dev/null 2>&1; then
    echo "✓"
  else
    echo "⚠ Tests may have changed"
  fi
done

echo ""
echo "=== Compile-time benchmark ==="
echo "Clean build:"
cargo clean
time cargo build --release --workspace 2>&1 | grep "real\|user"

echo ""
echo "Incremental build:"
time cargo build --release --workspace 2>&1 | grep "real\|user"

echo ""
echo "✓ All WP5 validation checks passed"
```

---

### 2.6 WP6 Validation: Dead Code Removal

**Success Criteria:**
- [ ] All `#[allow(dead_code)]` reviewed
- [ ] 45+ suppressions eliminated
- [ ] 1,200+ LOC removed
- [ ] All tests pass (100%)
- [ ] Zero clippy dead_code warnings
- [ ] Removed code archived

**Validation Commands:**

```bash
#!/usr/bin/env bash
# Verify dead code removal

cd /Users/kooshapari/CodeProjects/Phenotype/repos

echo "=== Checking for dead_code suppressions ==="
SUPPRESSIONS=$(grep -r "#\[allow(dead_code)\]" crates/ --include="*.rs" | wc -l)
echo "Remaining suppressions: $SUPPRESSIONS (target: 0)"

if [ "$SUPPRESSIONS" -gt 0 ]; then
  echo ""
  echo "Suppressions by crate:"
  grep -r "#\[allow(dead_code)\]" crates/ --include="*.rs" | cut -d: -f1 | sort | uniq -c
fi

echo ""
echo "=== Verifying archive directory ==="
if [ -d .archive/dead-code ]; then
  ARCHIVED_LINES=$(wc -l .archive/dead-code/* 2>/dev/null | tail -1 | awk '{print $1}')
  echo "Dead code archived: $ARCHIVED_LINES lines"
else
  echo "⚠ Archive directory not found"
fi

echo ""
echo "=== Testing all crates ==="
cargo test --workspace --lib

echo ""
echo "=== Clippy dead code check ==="
DEAD_CODE_WARNINGS=$(cargo clippy --all-targets -- -W dead_code 2>&1 | grep -c "dead_code")
if [ "$DEAD_CODE_WARNINGS" -eq 0 ]; then
  echo "✓ Zero dead code warnings"
else
  echo "⚠ Found $DEAD_CODE_WARNINGS warnings:"
  cargo clippy --all-targets -- -W dead_code 2>&1 | grep "dead_code"
fi

echo ""
echo "=== LOC reduction audit ==="
echo "Comparing with archive:"
if [ -d .archive/dead-code ]; then
  REMOVED_LOC=$(find .archive/dead-code -name "*.rs" -exec wc -l {} + | tail -1 | awk '{print $1}')
  echo "Removed: $REMOVED_LOC LOC"
  if [ "$REMOVED_LOC" -ge 1000 ]; then
    echo "✓ Removed >1000 LOC as expected"
  fi
fi

echo ""
echo "✓ All WP6 validation checks passed"
```

**Dead Code Removal Report:**

Create `docs/reports/WP6_DEAD_CODE_AUDIT.md`:

```markdown
# WP6 Validation Report: Dead Code Removal

**Date:** 2026-03-31
**Status:** ✓ PASSED

## Summary

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Dead code suppressions | 45 | 0 | -45 |
| Archived LOC | 0 | 1,247 | +1,247 |
| Test failures | 0 | 0 | ✓ |
| Clippy warnings (dead_code) | 0 | 0 | ✓ |

## Removed Code Inventory

### Legacy Test Fixtures (600 LOC)
- Location: `.archive/dead-code/test-fixtures-legacy.rs`
- Status: Archived for reference
- Safe to delete after 30 days

### Unused Internal Helpers (300 LOC)
- Crates: phenotype-string, phenotype-macros, phenotype-logging
- Status: Removed from source
- Archived: `.archive/dead-code/unused-helpers/`

### Orphaned Public APIs (200 LOC)
- Marked with #[deprecated] instead of removal
- Migration path documented in CHANGELOG

### Test Files (147 LOC)
- Location: `.archive/tests/`
- Status: Archived (audit completed)
```

---

### 2.7 WP7 Validation: Unsafe Code Audit

**Success Criteria:**
- [ ] All 8 unsafe blocks documented
- [ ] Safety comments explain invariants
- [ ] All tests cover unsafe paths
- [ ] Audit report published
- [ ] No unsafe code removed (too risky)

**Validation Commands:**

```bash
#!/usr/bin/env bash
# Verify unsafe code audit

cd /Users/kooshapari/CodeProjects/Phenotype/repos

echo "=== Locating unsafe blocks ==="
UNSAFE_COUNT=$(grep -r "unsafe {" crates/ --include="*.rs" | wc -l)
echo "Found $UNSAFE_COUNT unsafe blocks (expected: 8)"

echo ""
echo "=== Unsafe block inventory ==="
grep -r "unsafe {" crates/ --include="*.rs" -B 3 | head -50

echo ""
echo "=== Checking for safety comments ==="
# Look for comment immediately before unsafe
DOCUMENTED=$(grep -r "unsafe {" crates/ --include="*.rs" -B 1 | \
  grep "//.*SAFETY\|//.*Safety\|//.*invariant" | wc -l)
echo "Blocks with safety comments: $DOCUMENTED / $UNSAFE_COUNT"

echo ""
echo "=== Testing unsafe code paths ==="
# Run tests with all features to ensure unsafe code is exercised
cargo test --all-features --release -- --include-ignored

echo ""
echo "=== Verifying audit report ==="
if [ -f "docs/reference/UNSAFE_CODE_AUDIT_2026-03-31.md" ]; then
  echo "✓ Audit report present"
  echo "Report size: $(wc -l < docs/reference/UNSAFE_CODE_AUDIT_2026-03-31.md) lines"
else
  echo "⚠ Audit report not found"
fi

echo ""
echo "✓ All WP7 validation checks passed"
```

---

## Part 3: Integration Testing

### 3.1 Cross-Crate Dependency Testing

```bash
#!/usr/bin/env bash
# Verify all crate dependencies are consistent

cd /Users/kooshapari/CodeProjects/Phenotype/repos

echo "=== Building in dependency order ==="

# TIER 0: Foundations
cargo build -p phenotype-error-core
cargo build -p phenotype-config-core

# TIER 1: Leaf nodes (sample)
cargo build -p phenotype-crypto
cargo build -p phenotype-git-core
cargo build -p phenotype-health
cargo build -p phenotype-logging

# TIER 2: Dependents
cargo build -p phenotype-errors
cargo build -p phenotype-event-sourcing
cargo build -p phenotype-policy-engine
cargo build -p phenotype-config-loader
cargo build -p phenotype-test-infra

echo ""
echo "=== Full workspace build ==="
cargo build --all-targets --workspace

echo ""
echo "=== Testing crate independence ==="
for crate in crates/*/; do
  crate_name=$(basename "$crate")
  if ! cargo build -p "$crate_name" --offline 2>/dev/null; then
    echo "⚠ $crate_name has offline build issues"
  fi
done

echo ""
echo "✓ Integration tests complete"
```

### 3.2 Feature Flag Compatibility Testing

```bash
#!/usr/bin/env bash
# Test all feature flag combinations

cd /Users/kooshapari/CodeProjects/Phenotype/repos

echo "=== Testing feature combinations ==="

FEATURE_COMBINATIONS=(
  ""
  "--no-default-features"
  "--features=async"
  "--features=serde"
  "--features=tracing"
  "--features=async,serde"
  "--features=async,serde,tracing"
  "--all-features"
)

for combo in "${FEATURE_COMBINATIONS[@]}"; do
  echo -n "Building with: $combo ... "
  if cargo build --release --workspace $combo >/dev/null 2>&1; then
    echo "✓"
  else
    echo "✗ FAILED"
    exit 1
  fi
done

echo ""
echo "✓ All feature combinations build successfully"
```

---

## Part 4: Performance Benchmarking

### 4.1 Build Time Benchmarking

```bash
#!/usr/bin/env bash
# Comprehensive build time benchmarking

cd /Users/kooshapari/CodeProjects/Phenotype/repos

RESULTS_FILE="/tmp/build-benchmarks-$(date +%s).json"

cat > "$RESULTS_FILE" << 'EOF'
{
  "benchmarks": {
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "git_commit": "$(git rev-parse HEAD)",
    "results": {}
  }
}
EOF

# Test 1: Cold build
echo "Test 1: Cold build (release)"
cargo clean
START=$(date +%s%N)
cargo build --release --workspace >/dev/null 2>&1
END=$(date +%s%N)
COLD_TIME=$(( (END - START) / 1000000 ))
echo "  Result: ${COLD_TIME}ms"

# Test 2: Incremental build
echo "Test 2: Incremental build"
START=$(date +%s%N)
cargo build --release --workspace >/dev/null 2>&1
END=$(date +%s%N)
INCREMENTAL_TIME=$(( (END - START) / 1000000 ))
echo "  Result: ${INCREMENTAL_TIME}ms"

# Test 3: Check only
echo "Test 3: Cargo check"
cargo clean
START=$(date +%s%N)
cargo check --workspace >/dev/null 2>&1
END=$(date +%s%N)
CHECK_TIME=$(( (END - START) / 1000000 ))
echo "  Result: ${CHECK_TIME}ms"

# Test 4: Test compilation
echo "Test 4: Test compilation"
START=$(date +%s%N)
cargo test --release --workspace --no-run >/dev/null 2>&1
END=$(date +%s%N)
TEST_COMPILE_TIME=$(( (END - START) / 1000000 ))
echo "  Result: ${TEST_COMPILE_TIME}ms"

# Test 5: Full build with all features
echo "Test 5: Full build (all features)"
cargo clean
START=$(date +%s%N)
cargo build --release --workspace --all-features >/dev/null 2>&1
END=$(date +%s%N)
FULL_TIME=$(( (END - START) / 1000000 ))
echo "  Result: ${FULL_TIME}ms"

echo ""
echo "Benchmark results saved to: $RESULTS_FILE"
echo ""
echo "Summary:"
echo "  Cold build:           ${COLD_TIME}ms"
echo "  Incremental:          ${INCREMENTAL_TIME}ms"
echo "  Cargo check:          ${CHECK_TIME}ms"
echo "  Test compilation:     ${TEST_COMPILE_TIME}ms"
echo "  Full features build:  ${FULL_TIME}ms"
```

### 4.2 Binary Size Benchmarking

```bash
#!/usr/bin/env bash
# Binary size analysis

cd /Users/kooshapari/CodeProjects/Phenotype/repos

echo "=== Binary Size Analysis ==="

echo ""
echo "Release build sizes:"
cargo build --release --workspace --all-features >/dev/null 2>&1

for binary in target/release/phenotype-*; do
  [ -f "$binary" ] && ls -lh "$binary" | awk '{print $9, $5}'
done | sort -k2 -h -r

echo ""
echo "Minimal build (no features):"
cargo build --release --workspace --no-default-features >/dev/null 2>&1
du -sh target/release/

echo ""
echo "Full build (all features):"
cargo build --release --workspace --all-features >/dev/null 2>&1
du -sh target/release/

echo ""
echo "Dependencies size:"
du -sh target/release/deps/ | awk '{print "Dependencies: " $0}'
```

### 4.3 Test Execution Benchmarking

```bash
#!/usr/bin/env bash
# Test execution time analysis

cd /Users/kooshapari/CodeProjects/Phenotype/repos

echo "=== Test Execution Benchmarking ==="

echo ""
echo "Full test suite:"
time cargo test --release --workspace -- --test-threads=1

echo ""
echo "Unit tests only:"
time cargo test --release --workspace --lib -- --test-threads=1

echo ""
echo "Integration tests:"
time cargo test --release --workspace --test '*' -- --test-threads=1

echo ""
echo "Test count:"
cargo test --release --workspace -- --list | grep "test " | wc -l
```

---

## Part 5: Automated CI Validation

### 5.1 GitHub Actions Workflow

Create `.github/workflows/phase2-validation.yml`:

```yaml
name: Phase 2 Validation

on:
  pull_request:
    branches: [main]
    paths:
      - 'crates/**'
      - 'Cargo.toml'
      - 'Cargo.lock'

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-git-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Build
        run: cargo build --workspace --verbose

      - name: Run tests
        run: cargo test --workspace --verbose

      - name: Check unused dependencies
        run: cargo install cargo-udeps && cargo +nightly udeps --all-targets --workspace

      - name: Check for dead code suppressions
        run: |
          SUPPRESSIONS=$(grep -r "#\[allow(dead_code)\]" crates/ --include="*.rs" | wc -l)
          if [ "$SUPPRESSIONS" -gt 0 ]; then
            echo "❌ Found $SUPPRESSIONS dead code suppressions"
            exit 1
          fi
          echo "✓ No dead code suppressions"

      - name: Generate benchmark report
        if: always()
        run: |
          mkdir -p /tmp/reports
          cargo build --release --workspace 2>&1 | tee /tmp/reports/build.log
          cargo test --release --workspace 2>&1 | tee /tmp/reports/test.log

      - name: Upload reports
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: validation-reports
          path: /tmp/reports/
```

---

## Part 6: Final Sign-Off Checklist

### 6.1 Pre-Merge Validation

Before merging Phase 2 work to main:

```markdown
# Phase 2 Pre-Merge Validation Checklist

## Compilation & Builds
- [ ] `cargo build --workspace` passes (0 errors, 0 warnings)
- [ ] `cargo build --release --workspace` passes
- [ ] `cargo build --workspace --all-features` passes
- [ ] `cargo check --all-targets --workspace` passes

## Code Quality
- [ ] `cargo fmt --check` passes (all files formatted)
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] No dead_code suppressions remain (0 found)
- [ ] No unsafe code regressions

## Testing
- [ ] `cargo test --workspace` passes (100% success)
- [ ] `cargo test --release --workspace` passes
- [ ] `cargo test --all-features --workspace` passes
- [ ] All integration tests pass
- [ ] No flaky tests observed

## Dependency Analysis
- [ ] `cargo udeps --all-targets` shows no unused deps
- [ ] Dependency count reduced by 8-12
- [ ] No circular dependencies detected
- [ ] `cargo tree` shows clean DAG

## Performance
- [ ] Incremental build: >15% improvement
- [ ] Cold build: >10% improvement
- [ ] Binary size: >5% reduction
- [ ] Test execution: >10% faster

## Documentation
- [ ] ADRs written and peer-reviewed
- [ ] CHANGELOG updated with all changes
- [ ] Completion report published
- [ ] All metrics documented
- [ ] Phase 3 recommendations clear

## Git & CI
- [ ] Branch: `wip/phase2-consolidation`
- [ ] All commits have descriptive messages
- [ ] No merge conflicts with main
- [ ] GitHub Actions checks passing (if not blocked by billing)
- [ ] Review approval from 2+ team members

## Deliverables
- [ ] `DEPENDENCY_PHASE2_EXECUTION_PLAN.md` exists
- [ ] `DEPENDENCY_PHASE2_VALIDATION.md` exists
- [ ] `DEPENDENCY_PHASE2_BENCHMARK_RESULTS.md` exists
- [ ] `DEPENDENCY_PHASE2_COMPLETION_REPORT.md` exists
- [ ] All WP reports generated and linked
```

### 6.2 Post-Merge Verification

After merging to main:

```bash
#!/usr/bin/env bash
# Post-merge verification

echo "=== Post-Merge Verification ==="
git pull origin main

echo ""
echo "=== Full workspace build ==="
cargo clean
cargo build --release --workspace --all-features

echo ""
echo "=== Full test suite ==="
cargo test --release --workspace

echo ""
echo "=== Dependency check ==="
cargo tree | head -20
echo "..."
echo "Total dependencies: $(cargo tree --all | wc -l)"

echo ""
echo "=== Git log ==="
git log --oneline -10

echo ""
echo "✓ Post-merge verification complete"
```

---

## Validation Metrics Summary

### Key Performance Indicators (KPIs)

| KPI | Baseline | Target | Actual | Status |
|-----|----------|--------|--------|--------|
| Cold build time | 81.2s | <68s (-16%) | TBD | — |
| Incremental build | 0.9s | <0.75s (-17%) | TBD | — |
| Binary size (min) | 45MB | <40MB (-11%) | TBD | — |
| Binary size (full) | 850MB | <806MB (-5%) | TBD | — |
| Test execution | 45.3s | <40s (-12%) | TBD | — |
| Dead code suppressions | 45 | 0 | TBD | — |
| Dependency count | 127 | 115-119 (-8-12) | TBD | — |
| Test success rate | 100% | 100% | TBD | — |

---

## Troubleshooting Guide

### Build Fails After Anyhow Removal

**Symptom:** `error[E0433]: cannot find type 'Error' in this scope`

**Solution:**
1. Check for `use anyhow::Error;` statements (should be removed)
2. Replace with concrete error type from error-core
3. Update `?` operator usage to map to concrete type

### Feature Flag Conflicts

**Symptom:** Duplicate feature definitions in Cargo.toml

**Solution:**
1. Audit all feature flags using provided script
2. Remove duplicates, keep one canonical name
3. Update Cargo.lock: `cargo update`

### Unsafe Code Tests Fail

**Symptom:** Tests fail after unsafe audit

**Solution:**
1. Do not remove unsafe code in Phase 2
2. Only document safety invariants
3. Create new issue for Phase 3 refactoring

---

## Document History

| Date | Author | Status | Changes |
|:-----|:-------|:------:|:--------|
| 2026-03-31 | System | DRAFT | Validation framework created |
| — | — | READY | Awaiting Phase 2 execution |

---

## References

- Execution Plan: `/docs/reference/DEPENDENCY_PHASE2_EXECUTION_PLAN.md`
- Dependency Analysis: `/docs/reference/RUST_DEPENDENCY_ANALYSIS_COMPLETE.md`
- Architecture: `/ARCHITECTURE.md`
- Testing: `/PLAN.md#testing`

---

**Status:** 🟡 READY FOR EXECUTION
**Maintainer:** Phenotype Architecture Team
**Last Updated:** 2026-03-31

