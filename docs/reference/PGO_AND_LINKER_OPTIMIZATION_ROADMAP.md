# PGO and Linker Optimization Roadmap

**Document:** Profile-Guided Optimization (PGO) & Linker Strategy
**Scope:** phenotype-infrakit (Rust 1.93, 23 workspace crates)
**Date:** 2026-03-30
**Status:** Design & Planning Phase

---

## Executive Summary

This roadmap outlines a phased approach to optimize the phenotype-infrakit build pipeline and runtime performance through Profile-Guided Optimization (PGO) and modern linker adoption. Current builds already use Link-Time Optimization (LTO), but are bottlenecked by the slow GNU `ld` linker on CI. By adopting mold (Linux CI) and implementing PGO, we can achieve:

- **Binary size:** 5–15% reduction
- **Link time:** 5–10x speedup on Linux CI
- **Runtime perf:** 5–20% improvement (workload-dependent)
- **CI overhead:** +30–50% for PGO builds (mitigated by optional jobs)

---

## Current State Analysis

### Build Configuration

**File:** `/Cargo.toml` [profile.release]

```toml
[profile.release]
opt-level = "z"         # Optimize for size
lto = true              # Link-Time Optimization enabled
codegen-units = 1       # Single unit for better optimization (slower compile)
strip = true            # Remove debug symbols
```

**Analysis:**
- ✅ LTO already enabled (good for optimization)
- ✅ Size optimization active (opt-level = "z")
- ⚠️ Debug symbols stripped (blocks BOLT integration later)
- ⚠️ Linker not specified in Cargo.toml (uses system default)

### Rust Version & Stability

- **Current:** Rust 1.93.1 (2026-02-11)
- **MSRV:** 1.75 (defined in Cargo.toml)
- **PGO Status:** Stabilized in Rust 1.86 (December 2024) ✅
- **Recommendation:** PGO is production-ready for Rust 1.75+

### CI Build Pipeline

**File:** `/github/workflows/ci.yml`

Current jobs:
- `rust-check`, `rust-build`, `rust-msrv` (workspace tests)
- `core-check`, `core-build`, `core-msrv` (monorepo tests)
- `rust-coverage` (tarpaulin code coverage)

**Current linker setup:**
- **macOS (local):** ld64 (Apple's native linker) — good default
- **Linux CI:** GNU ld (system default) — slow for LTO builds
- **Explicit config:** None — relies on system defaults

### Current Linker Performance (Estimated)

Based on Rust LTO + size optimization:
- **Link time (current):** ~30–60 seconds on ubuntu-latest
- **Binary size:** ~15–20 MB (release binary)
- **Bottleneck:** GNU ld is single-threaded for parts of LTO

---

## Linker Comparison & Selection

### Linker Options for Rust

| Linker | Platform | Speed Vs ld | LTO Support | Status | Notes |
|--------|----------|-----------|------------|--------|-------|
| **mold** | Linux/Windows | 5–10x faster | Excellent | Production | Modern, used by Chrome/LLVM; not on macOS |
| **lld** | Linux/Windows | 2–5x faster | Excellent | Stable | LLVM-based; good fallback for mold |
| **gold** | Linux/Windows | ~2x faster | Good | Mature | GNU; older; deprecated in LLVM 17+ |
| **ld** | All platforms | Baseline | Limited | Stable | Default; slow for LTO; production fallback |
| **ld64** | macOS | N/A (native) | Excellent | Default | Optimized for macOS; leave as-is |

### Recommended Strategy

1. **macOS (local development):** Keep ld64 as default (no change needed)
2. **Linux CI:** Migrate to mold (5–10x speedup for LTO)
3. **Fallback:** lld if mold unavailable
4. **Production:** Use mold for release binaries (if Linux), ld for others

---

## Phase 1: Baseline Measurement & Linker Adoption (Weeks 1–2)

### 1.1 Document Current State

**Tasks:**
- [ ] Measure current link time on ubuntu-latest (extract from CI logs)
- [ ] Measure current binary size: `cargo build --release --workspace`
- [ ] Record baseline in `BENCHMARK_BASELINE.md`

**Expected output:**
```
Link time (GNU ld + LTO): ~45 seconds
Binary size (total): ~45 MB
Binary size (single crate): 1–5 MB range
```

### 1.2 Install Linker Tools

**Action:** Add mold and lld installation to CI workflow

**CI job to add:**

```yaml
linker-setup:
  name: Install Linker Tools
  runs-on: ubuntu-latest
  steps:
    - name: Install mold
      run: |
        sudo apt-get update
        sudo apt-get install -y mold lld

    - name: Verify installation
      run: |
        mold --version
        lld --version
```

**Effort:** 30 minutes (add to CI, test locally)

### 1.3 Add Linker Configuration to Cargo.toml

**Action:** Create `.cargo/config.toml` with linker overrides per platform

**File:** `/repos/.cargo/config.toml` (new)

```toml
# Linker configuration
[build]
# Use mold on Linux (5-10x faster for LTO)
# Falls back to lld if mold unavailable
rustflags = ["-C", "link-arg=-fuse-ld=mold", "-C", "link-arg=-Wl,--as-needed"]

# Alternative for when mold is unavailable:
# rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

**Note:** This config applies globally; macOS will override if needed:

```toml
[target.aarch64-apple-darwin]
# macOS: Use system ld64 (no explicit config needed)
# ld64 is optimized for Darwin and handles LTO natively

[target.x86_64-apple-darwin]
# macOS: Use system ld64 (no explicit config needed)
```

**Testing:**
```bash
# Verify mold is being used
RUSTFLAGS="-v" cargo build --release 2>&1 | grep "mold"
```

**Effort:** 45 minutes (create config, test on Linux CI)

### 1.4 Benchmark Phase 1 Results

**CI job to add:**

```yaml
linker-benchmark:
  name: Linker Performance Comparison
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v6
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2

    - name: Install mold
      run: sudo apt-get install -y mold

    - name: Build with mold (measure time)
      run: |
        /usr/bin/time -v cargo build --release --workspace 2>&1 | tee mold-build.log

    - name: Extract metrics
      run: |
        echo "Link time with mold:"
        grep "Elapsed" mold-build.log || echo "N/A"
```

**Success criteria:**
- ✅ mold installed and functional on CI
- ✅ Link time reduced by 50% or more
- ✅ Binary size unchanged
- ✅ No functional regressions

**Timeline:** 1 week (wait for CI results)

---

## Phase 2: Profile-Guided Optimization Setup (Weeks 3–4)

### 2.1 Design PGO Workflow

**PGO Process:**

1. **Instrumentation build:** Compile with `-C llvm-profile-generate`
2. **Workload execution:** Run integration tests to gather profiling data
3. **Profile merging:** Combine `.profraw` files into aggregated profile
4. **Optimized build:** Recompile with `-C llvm-profile-use=<merged_profile>`

**Workload selection:**
- Primary: Integration tests (`cargo test --release --workspace`)
- Alternative: Benchmarks (`cargo bench --no-run`)
- Custom: Add realistic load scenarios if needed

### 2.2 Implement PGO Build Jobs

**CI job to add (optional, not on critical path):**

```yaml
pgo-build:
  name: PGO Instrumentation & Optimization
  runs-on: ubuntu-latest
  continue-on-error: true
  steps:
    - uses: actions/checkout@v6
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2

    - name: Install llvm-tools
      run: rustup component add llvm-tools-preview

    - name: Build instrumented binary
      env:
        RUSTFLAGS: "-C llvm-profile-generate"
        LLVM_PROFILE_FILE: "profile-%p-%m.profraw"
      run: cargo build --release --workspace

    - name: Run workload (integration tests)
      run: |
        mkdir -p /tmp/pgo-data
        cd /tmp/pgo-data
        cp -r /home/runner/work/phenotype-infrakit/phenotype-infrakit/target/release/* .
        # Run integration tests to generate profiles
        cargo test --release --workspace || true

    - name: Merge profiles
      run: |
        llvm-profdata merge -o merged.profdata *.profraw

    - name: Build optimized binary
      env:
        RUSTFLAGS: "-C llvm-profile-use=$(pwd)/merged.profdata"
      run: cargo build --release --workspace

    - name: Compare binaries
      run: |
        echo "Instrumented vs Optimized sizes:"
        du -sh target/release/*.{bin,so,dylib} || true
```

**Alternative (simpler): Add to existing build job**

```yaml
rust-build-pgo:
  name: Rust Build (PGO Variant)
  runs-on: ubuntu-latest
  continue-on-error: true
  steps:
    - uses: actions/checkout@v6
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2

    - name: Install llvm-tools
      run: rustup component add llvm-tools-preview

    - name: Instrument & optimize
      run: |
        # Step 1: Instrumented build
        RUSTFLAGS="-C llvm-profile-generate" cargo build --release --workspace
        LLVM_PROFILE_FILE="profile-%p.profraw" cargo test --release --workspace || true

        # Step 2: Merge profiles
        llvm-profdata merge -o merged.profdata *.profraw

        # Step 3: Optimized build
        RUSTFLAGS="-C llvm-profile-use=$(pwd)/merged.profdata -C llvm-profile-use-dir=$(pwd)" \
          cargo build --release --workspace

        # Step 4: Measure improvement
        du -sh target/release/ > pgo-size.txt
        cat pgo-size.txt
```

**Effort:** 2–3 hours (implement, debug on CI, iterate)

### 2.3 Measure PGO Impact

**Metrics to track:**

| Metric | Baseline | Expected | Impact |
|--------|----------|----------|--------|
| Binary size | 45 MB | 38–40 MB | 5–15% reduction |
| Link time | 45 sec | 12 sec | 5–10x (mold) + 2–5% (PGO) |
| Runtime perf | — | +5–20% | Workload-dependent |
| CI time overhead | — | +60–90 sec | +30–50% |

**Create artifact:** `BENCHMARK_RESULTS_PHASE_2.md`

**Timeline:** 1 week (wait for CI results)

---

## Phase 3: BOLT Integration (Weeks 5–8)

### 3.1 Understand BOLT Workflow

**What is BOLT?**

BOLT (Binary Optimization and Layout Tool) is a post-link optimizer from the LLVM project. It:
- Reads compiled binary + CPU profiling data
- Reorders functions and basic blocks based on execution frequency
- Outputs optimized binary without recompilation
- Provides 3–5% additional speedup on top of PGO

**Workflow:**

```
1. Compile release binary (with -g for debug symbols)
   └─> cargo build --release

2. Profile with CPU sampling
   └─> perf record -c 100000 -F max -e cycles:u ./binary <workload>

3. Convert perf data to BOLT format
   └─> perf2bolt -p perf.data -o perf.fdata binary

4. Apply BOLT optimization
   └─> llvm-bolt binary -o binary.bolted -data=perf.fdata

5. Verify and deploy
   └─> ldd binary.bolted && ./binary.bolted <test>
```

### 3.2 BOLT Implementation Strategy

**Challenge:** BOLT is a post-link tool, not integrated into Cargo. Requires:
1. Preserving debug symbols (conflicts with `strip = true`)
2. Running perf sampling (requires Linux, not available on CI by default)
3. Manual build step (not part of normal Cargo workflow)

**Recommendation: Defer to Phase 4**

BOLT integration is high-effort, low-ROI for CI since:
- Requires disabling `strip = true` (conflicts with current config)
- Profiling data must come from real workloads (not CI synthetic tests)
- Best applied to release binaries at deployment time
- Can be added later with dedicated CI job or local tooling

**Alternative: Local-only BOLT workflow**

Document steps for developers to apply BOLT locally to release binaries:

**File:** `docs/guides/BOLT_LOCAL_OPTIMIZATION.md` (to create in Phase 3)

```markdown
# Applying BOLT to Release Binaries (Local Only)

## Prerequisites
- `perf` installed: `sudo apt-get install linux-tools-generic`
- `llvm-bolt` installed: `cargo install llvm-tools-preview`
- Debug symbols available (don't strip before BOLT)

## Steps

1. Build with debug symbols
   ```bash
   cargo build --release
   ```

2. Profile with perf
   ```bash
   perf record -c 100000 -F max -e cycles:u \
     ./target/release/<binary> <your-workload>
   ```

3. Convert to BOLT format
   ```bash
   perf2bolt -p perf.data -o perf.fdata \
     ./target/release/<binary>
   ```

4. Apply BOLT
   ```bash
   llvm-bolt ./target/release/<binary> \
     -o ./target/release/<binary>.bolted \
     -data=perf.fdata
   ```

5. Test
   ```bash
   ./target/release/<binary>.bolted <test-inputs>
   ```
```

**Timeline for Phase 3:**
- Defer main BOLT CI integration to Phase 4
- Document local workflow (1–2 hours)
- Consider if worth effort given other optimizations

---

## Phase 4: Continuous Optimization & Advanced Features (Month 2+)

### 4.1 PGO Automation in Release Pipeline

**Goal:** Make PGO builds part of the release workflow

**Implementation:**
- Separate release job that builds with PGO
- Store profile data in release artifacts
- Compare release binaries (with/without PGO) side-by-side

**CI job:**

```yaml
release-pgo:
  name: Release Build with PGO
  runs-on: ubuntu-latest
  if: startsWith(github.ref, 'refs/tags/')
  steps:
    - uses: actions/checkout@v6
    - uses: dtolnay/rust-toolchain@stable
    - name: Build PGO release binaries
      run: |
        # Instrument
        RUSTFLAGS="-C llvm-profile-generate" \
          cargo build --release --workspace

        # Profile with release tests
        cargo test --release --workspace || true

        # Optimize
        llvm-profdata merge -o merged.profdata *.profraw
        RUSTFLAGS="-C llvm-profile-use=$(pwd)/merged.profdata" \
          cargo build --release --workspace

        # Archive
        tar -czf phenotype-infrakit-pgo-${GITHUB_REF#refs/tags/}.tar.gz \
          target/release/
```

### 4.2 Runtime Benchmarking Infrastructure

**Goal:** Measure real-world perf improvements from PGO/linker changes

**Components:**
- Macro-benchmarks (realistic workloads, not micro-benchmarks)
- CI benchmark job (runs on every push)
- Historical tracking (GitHub Actions benchmark action)
- Alerting (if perf regresses >5%)

### 4.3 BOLT in Release Pipeline (Optional)

**If Phase 3 proves valuable:**
- Add BOLT step to release build
- Store pre-BOLT and post-BOLT binaries
- Profile with real production workloads
- Ship bolted binaries as default

---

## Implementation Roadmap (Timeline & Effort)

### Summary Table

| Phase | Tasks | Timeline | Effort | Blockers | Notes |
|-------|-------|----------|--------|----------|-------|
| **1** | Baseline + mold | Weeks 1–2 | 8–10h | None | Quick win, 5–10x link speedup |
| **2** | PGO setup & benchmarking | Weeks 3–4 | 12–16h | PGO stable (✅ in 1.86+) | +30–50% CI time, optional job |
| **3** | BOLT documentation | Week 5 | 4–6h | None | Defer full integration to Phase 4 |
| **4** | Continuous PGO + benchmarking | Month 2+ | 16–24h | Benchmarking infra | Ongoing work |

### Critical Path (MVP)

**Goal: Achieve mold + baseline PGO in 4 weeks**

1. **Week 1:** Install mold, measure link time (+1–2h CI time)
2. **Week 2:** Verify mold works, update `.cargo/config.toml` (+2h)
3. **Week 3:** Implement PGO instrumentation job (+3h)
4. **Week 4:** Benchmark and document results (+2h)

**Total effort:** 8–10 hours of agent time
**Expected ROI:** 5–10x link speedup, 5–15% binary size reduction

---

## Risk Assessment

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| mold not available on CI | Low | 1 day delay | Use lld fallback in CI |
| PGO profile data stale | Medium | 5% perf regression | Re-profile per release |
| mold breaks binary compatibility | Very low | Major | Test on both mold + ld builds |
| BOLT breaks unwind tables | Low | Debugging impossible | Preserve debug symbols |

### Operational Risks

| Risk | Mitigation |
|------|------------|
| **CI time increases 30–50%** | Make PGO job optional; don't block main CI |
| **Release builds get slower** | Separate release pipeline; cache PGO profiles |
| **Dev builds unaffected** (mold only on Linux) | Binaries identical; macOS uses ld64 as before |

### Fallback Strategy

If issues arise:
1. **mold fails:** Disable and use system ld (zero config change needed)
2. **PGO degrades perf:** Use `-C llvm-profile-use-dir=<cached>` with stale data
3. **CI slowdown unacceptable:** Move PGO to scheduled jobs (nightly, not on PR)

---

## Success Criteria

### Phase 1 (Weeks 1–2)

- ✅ mold installed and functional on ubuntu-latest
- ✅ `.cargo/config.toml` added with linker override
- ✅ Link time reduced to <15 seconds
- ✅ Baseline measurements documented

### Phase 2 (Weeks 3–4)

- ✅ PGO instrumentation job added to CI
- ✅ Profile merge automation working
- ✅ Binary size measured (target: 5–15% reduction)
- ✅ CI overhead acceptable (<60 seconds added)
- ✅ Results documented in `BENCHMARK_RESULTS_PHASE_2.md`

### Phase 3 (Week 5)

- ✅ BOLT workflow documented (local optimization guide)
- ✅ Decision made: defer full CI integration or proceed
- ✅ Risk assessment updated

### Phase 4 (Ongoing)

- ✅ PGO part of release workflow
- ✅ Benchmarking infrastructure operational
- ✅ Perf regressions detected automatically

---

## Installation & Configuration Instructions

### Step 1: Install mold locally (macOS with Homebrew)

```bash
# macOS note: mold not supported natively, but can test on Linux VM/Docker
# For Linux:
sudo apt-get install -y mold

# Verify
mold --version
# Output: mold 2.x.x (version may vary)
```

### Step 2: Create `.cargo/config.toml`

**File:** `/repos/.cargo/config.toml`

```toml
# Linker configuration for phenotype-infrakit
# Updated: 2026-03-30

[build]
# Use mold on all platforms (will be replaced with platform-specific logic)
# mold is 5-10x faster than GNU ld for LTO builds
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

# If mold is unavailable, set:
# rustflags = ["-C", "link-arg=-fuse-ld=lld"]
# or remove the config entirely to use system default

# Platform-specific overrides (optional)
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[target.aarch64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

# macOS: Keep default ld64 (no explicit config)
# Windows: Uses LLD natively (no config needed)
```

### Step 3: Test locally

```bash
# Verify mold is being used
cd /Users/kooshapari/CodeProjects/Phenotype/repos
RUSTFLAGS="-v" cargo build --release 2>&1 | grep -i "mold\|lld\|ld"

# Check link time
time cargo build --release --workspace
```

### Step 4: Enable PGO builds (optional, Phase 2)

```bash
# Instrument build
RUSTFLAGS="-C llvm-profile-generate" cargo build --release --workspace

# Run workload
cargo test --release --workspace

# Merge profiles
llvm-profdata merge -o merged.profdata *.profraw

# Optimized build
RUSTFLAGS="-C llvm-profile-use=$(pwd)/merged.profdata" \
  cargo build --release --workspace
```

---

## Related Documentation

- **Rust PGO Guide:** https://doc.rust-lang.org/rustc/profile-guided-optimization.html
- **mold Documentation:** https://github.com/rui314/mold
- **lld Documentation:** https://lld.llvm.org/
- **BOLT Guide:** https://github.com/llvm/llvm-project/blob/main/bolt/README.md
- **Cargo Config Reference:** https://doc.rust-lang.org/cargo/reference/config.html

---

## Appendix: Additional Optimization Opportunities

### Beyond PGO & Linkers

1. **LTO Codegen Units**
   - Current: 1 (maximum optimization, slow compile)
   - Alternative: 16–256 (faster compile, less optimization)
   - Consider split: dev=256, release=1

2. **Debuginfo in Release**
   - Current: stripped (`strip = true`)
   - Alternative: split debuginfo (`split-debuginfo = "packed"`)
   - Benefit: BOLT support + crashdump debugging

3. **Dependency Tree Pruning**
   - See: `docs/reference/CARGO_DEPENDENCY_AUDIT.md` (future)
   - Opportunity: Remove unused deps (Phase 3 work)

4. **Incremental Compilation**
   - Current: Not used in release builds
   - Future: Consider for dev builds (`incremental = true`)

5. **Polly Loop Optimizer**
   - Rust support: Limited, experimental
   - Not recommended for stable builds

---

**Document prepared by:** Claude Code (Haiku 4.5)
**Last updated:** 2026-03-30
**Next review:** After Phase 1 completion (2026-04-13)
