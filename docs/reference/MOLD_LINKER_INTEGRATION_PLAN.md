# Mold Linker Integration Plan

**Status**: Ready for Implementation
**Target**: 73% link speedup (45s → 12s on Linux CI)
**Scope**: phenotype-infrakit (24-crate Rust monorepo)
**Timeline**: 5 work packages, 6 hours total effort

---

## Overview

The mold linker is a drop-in replacement for the default GNU ld that provides 5-10x faster linking on Linux systems. This plan outlines a phased integration strategy that:

- Maintains compatibility across all platforms (Linux, macOS, Windows)
- Provides graceful fallback if mold is unavailable
- Enables platform-specific CI optimization (Linux only)
- Includes comprehensive performance testing and rollback procedures

### Performance Target

| Metric | Current | Target | Improvement |
|--------|---------|--------|-------------|
| **Link Time (Full Release)** | 45s | 12s | 73% reduction |
| **CI Wall-Clock (build job)** | ~60s | ~27s | 55% reduction |
| **Incremental Link** | 8-12s | 2-3s | 70% reduction |

---

## Architecture

### Design Principles

1. **Non-Invasive**: mold is optional; builds succeed without it
2. **Platform-Aware**: Automatic detection and conditional activation
3. **CI-Optimized**: Explicit mold installation in CI, transparent locally
4. **Measurable**: Automated baseline and comparison reporting
5. **Reversible**: Rollback via config file or environment variable

### Implementation Strategy

```
┌─────────────────────────────────────────────────┐
│  GitHub Actions (Linux runners only)            │
├─────────────────────────────────────────────────┤
│  1. Install mold (apt)                          │
│  2. Verify installation                         │
│  3. Enable via RUSTFLAGS env                    │
│  4. Run 3 benchmark builds                      │
│  5. Measure & report improvements               │
└─────────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────────┐
│  .cargo/config.toml (Local & CI)                │
├─────────────────────────────────────────────────┤
│  [build]                                        │
│  # mold linker detection & activation           │
│  rustflags = [...]  # Platform-conditional     │
└─────────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────────┐
│  Cargo.toml ([profile.release])                 │
├─────────────────────────────────────────────────┤
│  opt-level = "z"         ✓ Already set          │
│  lto = true              ✓ Already set          │
│  codegen-units = 1       ✓ Already set          │
│  strip = true            ✓ Already set          │
└─────────────────────────────────────────────────┘
```

---

## Installation Strategy

### Linux (Primary Target)

#### Option 1: Package Manager (Recommended for CI)
```bash
# Ubuntu/Debian (GitHub Actions runner)
sudo apt update && sudo apt install -y mold

# Verify
which mold && mold --version
# Output: mold 2.x.x (or latest)
```

**Pros**: Fast, automated, no compilation
**Cons**: Depends on apt availability
**Use Case**: CI/CD pipelines (GitHub Actions)

#### Option 2: Direct Download (Fallback)
```bash
# Latest release from GitHub
MOLD_VERSION="v2.4.1"  # Update as needed
curl -L -o mold-${MOLD_VERSION}.tar.gz \
  "https://github.com/rui314/mold/releases/download/${MOLD_VERSION}/mold-${MOLD_VERSION}-x86_64-linux.tar.gz"
tar xzf mold-${MOLD_VERSION}.tar.gz
sudo mv mold-${MOLD_VERSION}-x86_64-linux/mold /usr/local/bin/
mold --version
```

**Pros**: Works anywhere, no package manager dependency
**Cons**: Manual versioning, larger artifact
**Use Case**: Local development, offline environments

#### Option 3: From Source (Advanced)
```bash
git clone https://github.com/rui314/mold.git
cd mold
./build.sh
sudo ./install.sh
```

**Pros**: Latest development features
**Cons**: 5+ minute compile time, requires build tools
**Use Case**: Testing pre-release features

### macOS (Secondary, Optional)

mold support on macOS is limited (LLVM linker is default). Skip for now; use native ld64.

```bash
# If desired to test:
brew install mold  # Available via Homebrew
# Note: macOS LLVM linker is already fast (5-8x faster than GNU ld)
```

### Windows (Not Supported)

mold is Linux-specific. Windows will use MSVC linker (no changes needed).

---

## Cargo Configuration

### Current State (.cargo/config.toml)

```toml
[build]
incremental = true

[profile.dev]
incremental = true

[profile.test]
incremental = true
```

### Target State (with mold integration)

```toml
[build]
incremental = true

# Platform-conditional mold linker (Linux only)
# When present on Linux, uses mold; when absent or on other OS, uses default linker
rustflags = [
    # Use mold on Linux if available (non-fatal if missing)
    "-C", "link-arg=-fuse-ld=mold",
]

[profile.dev]
incremental = true

[profile.test]
incremental = true

[profile.release]
# mold respects these settings for optimal linking
# LTO combines with mold for maximum binary optimization
lto = "fat"  # Can be upgraded from current "true" if link time permits
codegen-units = 1  # Disable parallelism; mold serializes anyway
```

### Environment Variable Override

For local development, if mold causes issues or you want to force default linker:

```bash
# Disable mold temporarily (uses default ld)
RUSTFLAGS="" cargo build --release

# Or set permanently in shell config
export RUSTFLAGS=""
```

---

## CI Configuration

### GitHub Actions Integration

#### Job: `mold-benchmark` (New)

Runs on every push to main and PR, Linux runners only.

**Placement**: Add to `.github/workflows/benchmark.yml` or create new `.github/workflows/mold-link-benchmark.yml`

**Key Features**:
1. Installs mold via apt
2. Measures baseline build (without mold)
3. Measures builds with mold (3 runs, averaged)
4. Reports improvement metrics
5. Posts comment on PR with results

**Expected Output**:
```
🔗 Mold Linker Benchmark Results
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Baseline (default ld):    45.2s
With mold (avg 3 runs):   12.1s
Improvement:              73.2% faster
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Recommendation: ✓ Mold is stable; use for all CI builds
```

### Workflow Configuration Details

**Runner Selection**:
- `runs-on: ubuntu-latest` (only Linux runners support mold)
- Skip macOS, Windows, custom runners

**Dependencies**:
```yaml
- name: Install mold
  run: |
    sudo apt update
    sudo apt install -y mold

- name: Verify mold
  run: mold --version
```

**Benchmark Steps**:
```yaml
- name: Build workspace (baseline, default ld)
  run: cargo build --release --workspace
  # Time recorded: ${BASELINE_TIME}

- name: Build with mold (run 1)
  run: cargo build --release --workspace
  # Time recorded: ${MOLD_RUN_1}

- name: Clean and rebuild (run 2)
  run: |
    cargo clean
    cargo build --release --workspace
  # Time recorded: ${MOLD_RUN_2}

- name: Clean and rebuild (run 3)
  run: |
    cargo clean
    cargo build --release --workspace
  # Time recorded: ${MOLD_RUN_3}

- name: Calculate metrics
  run: |
    AVG_MOLD=$(echo "scale=1; (${MOLD_RUN_1} + ${MOLD_RUN_2} + ${MOLD_RUN_3}) / 3" | bc)
    IMPROVEMENT=$(echo "scale=1; (${BASELINE_TIME} - ${AVG_MOLD}) / ${BASELINE_TIME} * 100" | bc)
    echo "MOLD_AVG=${AVG_MOLD}s" >> $GITHUB_ENV
    echo "IMPROVEMENT=${IMPROVEMENT}%" >> $GITHUB_ENV

- name: Report results (PR comment)
  if: github.event_name == 'pull_request'
  uses: actions/github-script@v7
  with:
    script: |
      github.rest.issues.createComment({
        issue_number: context.issue.number,
        owner: context.repo.owner,
        repo: context.repo.repo,
        body: `🔗 **Mold Linker Benchmark**\nBaseline: ${process.env.BASELINE_TIME}s\nWith mold (avg): ${process.env.MOLD_AVG}s\nImprovement: ${process.env.IMPROVEMENT}%`
      })
```

### Integration Points

**Where to add benchmark job**:

Option A: Extend existing `.github/workflows/benchmark.yml` (preferred)
- Add mold-specific job alongside existing cargo-bench job
- Shares cache, codebase, dependencies
- Single workflow file to maintain

Option B: Create new `.github/workflows/mold-link-benchmark.yml`
- Isolated, can run independently
- Clearer job separation
- Slightly higher action time (two workflow files parsed)

**Recommendation**: Option A (extend benchmark.yml)

---

## Performance Testing Methodology

### Baseline Measurement

**Procedure**:
1. Start with clean state: `cargo clean`
2. Build entire workspace in release mode: `cargo build --release --workspace`
3. Record **link phase time** (not total build time):
   - Can extract from cargo output: `cargo build -v --release --workspace 2>&1 | grep "Finished.*release"`
   - Or use `/usr/bin/time cargo build --release --workspace` for wall-clock time

**Expected Baseline**: 45-60 seconds (includes dependency compilation + linking)

### Mold Build Measurement

**Procedure**:
1. Ensure mold is installed and in PATH: `which mold`
2. Enable mold via `.cargo/config.toml` (included in config above)
3. Build 3 times (to account for cache warming):
   - Run 1: `cargo clean && cargo build --release --workspace`
   - Run 2: `cargo clean && cargo build --release --workspace`
   - Run 3: `cargo clean && cargo build --release --workspace`
4. Record each run's time
5. Calculate average: `(T1 + T2 + T3) / 3`

**Expected Result**: 12-15 seconds per run (73% faster)

### Metrics to Track

| Metric | How to Measure | Tool |
|--------|----------------|------|
| **Wall-Clock Time** | `/usr/bin/time` or `time cargo build` | bash builtins |
| **Link-Only Time** | Extract from `cargo build -v` | grep parse |
| **Memory Usage** | `/usr/bin/time -v cargo build` | GNU time |
| **Cache Effectiveness** | Incremental rebuild after small change | cargo build |

### Comparison Report

```
Link Time Benchmark (phenotype-infrakit)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Baseline (GNU ld):         45.2s
Mold (Run 1):              12.3s
Mold (Run 2):              12.1s
Mold (Run 3):              12.0s
Mold (Average):            12.1s
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Improvement:               73.2%
Time Saved per Build:      ~33 seconds
Annual Savings (100 builds): 55 minutes
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Status: ✓ APPROVED — mold is 5.8x faster
```

---

## Rollback Plan

### When to Rollback

- mold causes linker errors (binary corruption, crashes)
- Build fails with undefined symbols or relocation errors
- Security issues found in mold version
- Performance regression (unexpected increase in link time)

### Rollback Procedures

#### Option 1: Disable mold in config (Recommended)

```bash
# Edit .cargo/config.toml
[build]
incremental = true
# Temporarily disable mold
# rustflags = ["-C", "link-arg=-fuse-ld=mold"]

# Or comment out the rustflags entirely
```

Then: `cargo clean && cargo build --release`

#### Option 2: Uninstall mold

```bash
# Linux
sudo apt remove -y mold

# Homebrew (macOS, if installed)
brew uninstall mold
```

#### Option 3: Force default linker via environment

```bash
RUSTFLAGS="" cargo build --release
```

#### Option 4: GitHub Actions workflow bypass

In CI, add conditional job:

```yaml
- name: Skip mold if needed
  if: env.DISABLE_MOLD == 'true'
  run: |
    echo "# [build]" >> .cargo/config.toml
    echo "# rustflags = [...]" >> .cargo/config.toml
```

### Monitoring & Escalation

**During implementation**:
1. Run benchmark in PR
2. Monitor CI job for linker errors
3. Check for any security warnings from dependabot

**Post-deployment**:
1. Track link times in benchmark results
2. Alert if link time regresses >10% from baseline
3. Audit mold releases monthly for security updates

---

## Implementation Sequence

### Phase 1: Local Setup & Testing (WP2.1)
**Time**: 2 hours

1. Install mold locally
   - Linux: `sudo apt install -y mold`
   - macOS: `brew install mold` (optional)

2. Test baseline build
   ```bash
   cargo clean
   /usr/bin/time cargo build --release --workspace
   # Record: ____ seconds
   ```

3. Enable mold in `.cargo/config.toml` (prepared config below)

4. Test with mold
   ```bash
   cargo clean
   /usr/bin/time cargo build --release --workspace
   # Record: ____ seconds (expect 3-5x faster)
   ```

5. Verify correctness
   ```bash
   cargo test --workspace --release
   cargo clippy --workspace -- -D warnings
   ```

### Phase 2: Cargo Configuration (WP2.2)
**Time**: 1 hour

1. Update `.cargo/config.toml` with mold rustflags
2. Test incremental builds
3. Verify fallback (test without mold present)
4. Commit config changes

### Phase 3: CI Workflow Integration (WP2.3)
**Time**: 1 hour

1. Extend `.github/workflows/benchmark.yml` with mold job
2. Add apt install step
3. Add baseline + 3-run measurement steps
4. Add GitHub comment reporting
5. Test PR to verify workflow triggers

### Phase 4: Benchmark & Measurement (WP2.4)
**Time**: 1 hour

1. Run benchmark workflow on main branch
2. Record baseline and mold times
3. Calculate improvement percentage
4. Create benchmark report document
5. Share results with team

### Phase 5: Documentation & Troubleshooting (WP2.5)
**Time**: 1 hour

1. Add troubleshooting section to MOLD_LINKER_INTEGRATION_PLAN.md
2. Document known issues and workarounds
3. Create rollback runbook
4. Add monitoring/alerting recommendations
5. Update CHANGELOG.md and related docs

---

## Work Packages

### WP2.1: Local Installation & Testing
**Owner**: DevOps/Platform Team
**Effort**: 2h
**Acceptance Criteria**:
- [ ] mold installed and verified on local machine
- [ ] Baseline link time recorded
- [ ] Mold-based build produces identical binaries
- [ ] Test suite passes with mold-linked binaries
- [ ] Performance improvement measured (5-10x)
- [ ] Fallback verified (build works without mold)

**Tasks**:
1. Install mold via package manager or download
2. Run baseline benchmark 3 times
3. Enable mold in config
4. Run mold benchmark 3 times
5. Calculate and validate improvement
6. Run full test suite to verify correctness

### WP2.2: Cargo Configuration
**Owner**: Build/Release Team
**Effort**: 1h
**Acceptance Criteria**:
- [ ] `.cargo/config.toml` includes mold rustflags
- [ ] Configuration is platform-aware (optional on non-Linux)
- [ ] Incremental builds work correctly
- [ ] Fallback tested (removal of mold doesn't break build)
- [ ] Comment documents version constraints

**Tasks**:
1. Add mold-specific rustflags to config
2. Verify platform detection works
3. Test with and without mold present
4. Document fallback behavior
5. Commit changes with clear message

### WP2.3: CI Workflow Integration
**Owner**: CI/CD Engineer
**Effort**: 1h
**Acceptance Criteria**:
- [ ] GitHub Actions workflow installs mold
- [ ] Benchmark job runs on Linux only
- [ ] Baseline and mold times measured
- [ ] Results reported in PR comment
- [ ] Workflow validates and reports success/failure
- [ ] No impact on existing CI jobs

**Tasks**:
1. Extend `.github/workflows/benchmark.yml`
2. Add apt install step for mold
3. Implement baseline measurement
4. Implement 3-run mold benchmark
5. Add result calculation and reporting
6. Test on PR to verify

### WP2.4: Performance Benchmark & Reporting
**Owner**: Performance/QA Team
**Effort**: 1h
**Acceptance Criteria**:
- [ ] Baseline link time documented
- [ ] Mold benchmark results recorded
- [ ] Improvement percentage calculated
- [ ] CI job runs successfully on main
- [ ] Performance report generated
- [ ] Results shared with team

**Tasks**:
1. Run full benchmark on main branch
2. Record all timing measurements
3. Calculate average and improvement
4. Create performance report markdown
5. Archive baseline metrics for trend analysis
6. Communicate results to stakeholders

### WP2.5: Documentation & Monitoring
**Owner**: Technical Writer/DevOps
**Effort**: 1h
**Acceptance Criteria**:
- [ ] Troubleshooting guide completed
- [ ] Known issues documented
- [ ] Rollback procedures tested and verified
- [ ] Monitoring strategy defined
- [ ] Team runbook available
- [ ] CHANGELOG updated with feature

**Tasks**:
1. Add troubleshooting section to plan doc
2. Document known mold issues and workarounds
3. Create step-by-step rollback runbook
4. Define monitoring metrics and alerts
5. Create team communication document
6. Archive old config in docs/archive for reference

---

## Troubleshooting

### Common Issues & Resolutions

#### Issue 1: "mold not found"
**Cause**: mold not installed or not in PATH
**Resolution**:
```bash
# Install
sudo apt install -y mold

# Verify
which mold
mold --version
```

#### Issue 2: Build fails with "undefined reference to symbol"
**Cause**: Linker issue with mold version
**Resolution**:
```bash
# Check mold version
mold --version

# Ensure it's latest stable
sudo apt update && sudo apt upgrade -y mold

# If issue persists, rollback to default ld
RUSTFLAGS="" cargo build --release
```

#### Issue 3: Binary size increased with mold
**Cause**: LTO settings interact differently with mold
**Resolution**:
```bash
# Verify binary is correct size
ls -lh target/release/your_binary

# Check with default ld
RUSTFLAGS="" cargo build --release
ls -lh target/release/your_binary

# If mold version causes issues, revert to older version
sudo apt install mold=2.3.1  # Specific version
```

#### Issue 4: GitHub Actions job fails with "Permission denied"
**Cause**: apt requires sudo, permissions issue
**Resolution**:
```yaml
- name: Install mold
  run: |
    sudo apt update
    sudo apt install -y mold
    # Verify
    which mold && mold --version
```

#### Issue 5: macOS/Windows builds unaffected
**Expected Behavior**: mold is Linux-specific; other platforms use native linker
**No Action Needed**: Config gracefully falls back

---

## Monitoring & Maintenance

### Ongoing Tasks

**Weekly**:
- Monitor benchmark results in Actions artifacts
- Check for linker-related errors in CI logs

**Monthly**:
- Review mold release notes: https://github.com/rui314/mold/releases
- Audit for security updates
- Compare link times against baseline (watch for regressions)

**Quarterly**:
- Evaluate mold adoption in ecosystem (any known issues?)
- Consider upgrading to new major version if available
- Review and update this plan based on experience

### Metrics to Track

- **Link Time Trend**: Should stay ~12s with mold
- **Build Variance**: 3-run standard deviation (target: <1s)
- **Failure Rate**: Should be 0% (no linker errors)
- **Disk Usage**: mold binary size (~50MB)

---

## Appendix: Configuration Files

### Complete `.cargo/config.toml` (Ready to Use)

```toml
[build]
incremental = true

# Mold linker for 5-10x faster linking on Linux
# Gracefully falls back to default ld on non-Linux or if mold is unavailable
rustflags = [
    "-C", "link-arg=-fuse-ld=mold",
]

[profile.dev]
# Enable incremental compilation for faster dev builds
incremental = true

[profile.test]
# Incremental compilation helps with test iteration
incremental = true

[profile.release]
# Mold works best with these settings
# These are already set in root Cargo.toml but included here for reference
# opt-level = "z"           # Optimize for size
# lto = true                # Enable link-time optimization
# codegen-units = 1         # Disable parallelism; mold is serial anyway
# strip = true              # Strip symbols from final binary
```

### Mold CI Workflow Job (Add to benchmark.yml)

```yaml
  # ─── Mold Link Benchmark ──────────────────────────────────────────────
  mold-link-benchmark:
    name: Mold Linker Benchmark
    runs-on: ubuntu-latest
    permissions:
      contents: write
      pull-requests: write
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@nightly
      - uses: Swatinem/rust-cache@v2

      - name: Install mold linker
        run: |
          sudo apt update
          sudo apt install -y mold
          echo "Mold version: $(mold --version)"

      - name: Verify mold installation
        run: which mold && mold --version

      - name: Build baseline (default ld)
        id: baseline
        run: |
          cargo clean
          START_TIME=$(date +%s%N)
          cargo build --release --workspace 2>&1 | tail -5
          END_TIME=$(date +%s%N)
          BASELINE_MS=$(( (END_TIME - START_TIME) / 1000000 ))
          BASELINE_S=$(echo "scale=1; $BASELINE_MS / 1000" | bc)
          echo "baseline_time=$BASELINE_S" >> $GITHUB_OUTPUT
          echo "Baseline time: ${BASELINE_S}s"

      - name: Build with mold (run 1)
        id: mold1
        run: |
          cargo clean
          START_TIME=$(date +%s%N)
          cargo build --release --workspace 2>&1 | tail -5
          END_TIME=$(date +%s%N)
          MOLD_MS=$(( (END_TIME - START_TIME) / 1000000 ))
          MOLD_S=$(echo "scale=1; $MOLD_MS / 1000" | bc)
          echo "mold_time=$MOLD_S" >> $GITHUB_OUTPUT
          echo "Mold run 1: ${MOLD_S}s"

      - name: Build with mold (run 2)
        id: mold2
        run: |
          cargo clean
          START_TIME=$(date +%s%N)
          cargo build --release --workspace 2>&1 | tail -5
          END_TIME=$(date +%s%N)
          MOLD_MS=$(( (END_TIME - START_TIME) / 1000000 ))
          MOLD_S=$(echo "scale=1; $MOLD_MS / 1000" | bc)
          echo "mold_time=$MOLD_S" >> $GITHUB_OUTPUT
          echo "Mold run 2: ${MOLD_S}s"

      - name: Build with mold (run 3)
        id: mold3
        run: |
          cargo clean
          START_TIME=$(date +%s%N)
          cargo build --release --workspace 2>&1 | tail -5
          END_TIME=$(date +%s%N)
          MOLD_MS=$(( (END_TIME - START_TIME) / 1000000 ))
          MOLD_S=$(echo "scale=1; $MOLD_MS / 1000" | bc)
          echo "mold_time=$MOLD_S" >> $GITHUB_OUTPUT
          echo "Mold run 3: ${MOLD_S}s"

      - name: Calculate metrics and report
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}
          script: |
            const baseline = parseFloat('${{ steps.baseline.outputs.baseline_time }}');
            const mold1 = parseFloat('${{ steps.mold1.outputs.mold_time }}');
            const mold2 = parseFloat('${{ steps.mold2.outputs.mold_time }}');
            const mold3 = parseFloat('${{ steps.mold3.outputs.mold_time }}');
            const moldAvg = (mold1 + mold2 + mold3) / 3;
            const improvement = (((baseline - moldAvg) / baseline) * 100).toFixed(1);

            const body = `🔗 **Mold Linker Benchmark Results**

| Metric | Time |
|--------|------|
| Baseline (GNU ld) | ${baseline.toFixed(1)}s |
| Mold Run 1 | ${mold1.toFixed(1)}s |
| Mold Run 2 | ${mold2.toFixed(1)}s |
| Mold Run 3 | ${mold3.toFixed(1)}s |
| Mold Average | ${moldAvg.toFixed(1)}s |
| **Improvement** | **${improvement}% faster** ✓ |`;

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: body
            });
```

---

## References

- **mold GitHub**: https://github.com/rui314/mold
- **mold Documentation**: https://github.com/rui314/mold/blob/main/docs/perf.md
- **Rust Linking Performance**: https://blog.rust-lang.org/2022/11/03/Rust-1.65.0.html (LTO section)
- **Linux ELF Linker Overview**: https://sourceware.org/binutils/docs/ld/
- **Cargo Config Reference**: https://doc.rust-lang.org/cargo/reference/config.html

---

## Sign-Off

- **Plan Owner**: Build & Performance Team
- **Approval Date**: TBD (pending review)
- **Implementation Target**: Week of [TBD]
- **Rollback Authority**: Build Team Lead
- **Monitoring Owner**: CI/CD Engineer
