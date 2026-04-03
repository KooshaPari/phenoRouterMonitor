# Mold Linker - Troubleshooting Guide

**Status**: Reference Documentation
**Date**: 2026-03-31
**Scope**: Diagnosis and resolution of mold linker issues

---

## Common Issues & Resolutions

### Issue #1: "mold: command not found"

**Symptoms**:
```
error: linker `cc` not found
error: Unable to locate mold binary
```

**Cause**: mold is not installed or not in PATH

**Resolution**:

**On Linux (Ubuntu/Debian)**:
```bash
# Install via apt
sudo apt update
sudo apt install -y mold

# Verify
which mold
mold --version
```

**On macOS**:
```bash
# Install via Homebrew
brew install mold

# Or skip (ld64 is already fast on macOS)
echo "Note: mold is optional on macOS; ld64 is already optimized"
```

**On Other Linux Distros**:
```bash
# Option A: Download prebuilt binary
MOLD_VERSION="v2.4.1"
curl -L -o mold.tar.gz \
  "https://github.com/rui314/mold/releases/download/${MOLD_VERSION}/mold-${MOLD_VERSION}-x86_64-linux.tar.gz"
tar xzf mold.tar.gz
sudo mv mold-* /usr/local/bin/

# Option B: Compile from source
git clone https://github.com/rui314/mold.git
cd mold
mkdir build && cd build
cmake ..
cmake --build . -j$(nproc)
sudo cmake --install .
```

**Verification**:
```bash
mold --version
# Expected: Mold 2.4.x release (or newer)
```

---

### Issue #2: Build fails with "undefined reference to symbol"

**Symptoms**:
```
error: undefined reference to `__libc_start_main'
error: undefined reference to `malloc'
```

**Cause**: Mold version incompatibility or linker script issue

**Resolution**:

**Step 1: Check mold version**
```bash
mold --version
```

**Step 2: Ensure version is current**
```bash
# On Linux
sudo apt update && sudo apt upgrade -y mold

# On macOS
brew upgrade mold

# Or check latest release: https://github.com/rui314/mold/releases
```

**Step 3: Check Rust toolchain**
```bash
rustc --version
cargo --version
```

**Step 4: If issue persists, fallback to default linker**
```bash
# Temporarily disable mold
RUSTFLAGS="" cargo build --release --workspace

# Or comment out in .cargo/config.toml
# rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

**Step 5: Report issue**
- Check mold GitHub issues: https://github.com/rui314/mold/issues
- Include: mold version, Rust version, OS, error message

---

### Issue #3: Binary size increased or decreased unexpectedly

**Symptoms**:
```
ls -lh target/release/my_app
# Size changed significantly from previous build
```

**Cause**: LTO interactions with mold linker, or false alarm

**Resolution**:

**Step 1: Verify the difference**
```bash
# Build with mold
cargo build --release --workspace
SIZE_WITH_MOLD=$(stat -f%z target/release/my_app 2>/dev/null || stat -c%s target/release/my_app)

# Build without mold
RUSTFLAGS="" cargo build --release --workspace
SIZE_WITHOUT_MOLD=$(stat -f%z target/release/my_app 2>/dev/null || stat -c%s target/release/my_app)

# Compare
echo "With mold:    $SIZE_WITH_MOLD bytes"
echo "Without mold: $SIZE_WITHOUT_MOLD bytes"
PERCENT=$(echo "scale=2; ($SIZE_WITH_MOLD - $SIZE_WITHOUT_MOLD) / $SIZE_WITHOUT_MOLD * 100" | bc)
echo "Difference:   ${PERCENT}%"
```

**Step 2: Check if within tolerance**
```
±5% is normal
±10% is acceptable
>10% may indicate issue
```

**Step 3: If >10% difference, investigate LTO settings**

```bash
# Check profile.release in Cargo.toml
grep -A 5 "profile.release" Cargo.toml

# Expected:
# [profile.release]
# opt-level = "z"           # Optimize for size
# lto = true                # Link-time optimization
# codegen-units = 1         # Single codegen unit
# strip = true              # Strip symbols
```

**Step 4: If still concerned, use default linker for comparison**
```bash
# Ensure both builds use identical settings
RUSTFLAGS="" cargo build --release --workspace
ls -lh target/release/my_app

# Then with mold
cargo build --release --workspace
ls -lh target/release/my_app

# Compare hex dumps (detailed check)
objdump -h target/release/my_app | head -10
```

**Step 5: Report if binary corruption suspected**
- Ensure both binaries are executable
- Run binary tests: `./target/release/my_app --version`
- If tests fail, open issue with binary hashes

---

### Issue #4: GitHub Actions job fails with "Permission denied" or apt errors

**Symptoms**:
```
error: sudo: apt-get: command not found
error: Permission denied while trying to connect to Docker daemon
```

**Cause**: Runner permissions, apt cache, or environment setup

**Resolution**:

**For apt failures**:
```yaml
- name: Install mold linker
  run: |
    sudo apt update -y
    sudo apt install -y mold
```

**Key points**:
- `-y` flag auto-confirms installation
- `sudo` required for apt on GitHub Actions runners
- Add `-y` to prevent interactive prompts

**For permission errors**:
```yaml
- name: Install mold linker
  run: |
    sudo -E apt-get update
    sudo -E apt-get install -y mold
```

**For Docker/socket errors**:
- Skip if running in container environment
- Docker runners have different setup
- Use `ubuntu-latest` (standard VM runner)

**Test the fix**:
```bash
# Verify locally first
sudo apt update && sudo apt install -y mold
mold --version  # Should succeed
```

---

### Issue #5: Incremental builds slower than expected

**Symptoms**:
```
cargo build (after small change): 8-10s (expected: <5s)
```

**Cause**: Incremental compilation overhead, mold not used for partial links

**Resolution**:

**This is expected behavior** — mold optimizes full (clean) builds.

For incremental builds:
- Linker phase is smaller (~1-2s)
- Compilation phase dominates (~3-8s)
- Mold savings are less pronounced on incremental

**If truly slow**, check:
```bash
# Verify incremental compilation is enabled
grep -A 2 "profile.dev\|profile.test" .cargo/config.toml
# Should show: incremental = true

# Check cache status
rm -rf target/.cargo-ok  # Clear artifact cache
cargo build
# Second build should be faster
```

**Optimization for incremental**:
- Mold has less effect on incremental
- Focus optimization on full clean builds
- Incremental is primarily CPU-bound (compilation), not linker-bound

---

### Issue #6: Mold works locally but fails in CI

**Symptoms**:
```
Local: cargo build --release  (12s with mold) ✓
CI:    cargo build --release  (45s, no mold) ✗
```

**Cause**: mold not installed in CI, or config not applied

**Resolution**:

**Step 1: Verify CI workflow has mold install step**
```yaml
- name: Install mold linker
  run: |
    sudo apt update
    sudo apt install -y mold
    echo "Mold version: $(mold --version)"
```

**Step 2: Verify .cargo/config.toml is in repo**
```bash
git status .cargo/config.toml
# Should be tracked in git

git show HEAD:.cargo/config.toml | grep mold
# Should show rustflags with mold
```

**Step 3: Check CI uses ubuntu-latest**
```yaml
runs-on: ubuntu-latest  # ✓ Correct
# runs-on: macos-latest  # ✗ Wrong (mold not needed on macOS)
```

**Step 4: Debug CI by adding verbose logging**
```yaml
- name: Debug mold availability
  run: |
    which mold && mold --version
    echo "RUSTFLAGS=${RUSTFLAGS}"
    cargo --version
```

**Step 5: Ensure cache isn't stale**
```yaml
- uses: Swatinem/rust-cache@v2
  with:
    cache-all-crates: true
```

---

## Recovery & Rollback

### Quick Rollback (If Mold Causes Issues)

**Option 1: Comment out in code** (fastest)
```bash
# Edit .cargo/config.toml
nano .cargo/config.toml
# Comment out: # rustflags = ["-C", "link-arg=-fuse-ld=mold"]
cargo build --release
```

**Option 2: Override with environment**
```bash
RUSTFLAGS="" cargo build --release
```

**Option 3: Full revert (if broken config)**
```bash
git checkout .cargo/config.toml
cargo build --release
```

**Verification**:
```bash
# Ensure build succeeds without mold
cargo test --workspace --release
cargo clippy --workspace -- -D warnings
```

---

## Getting Help

### Resources

1. **Mold GitHub Issues**: https://github.com/rui314/mold/issues
   - Search for similar issues
   - Create detailed issue with: version, OS, error message, reproducible case

2. **Rust Linker Forum**: https://discourse.rust-lang.org/
   - Tag: `linker`, `performance`
   - Experienced community

3. **Local Debugging**
   - Enable verbose output: `cargo build -vv --release`
   - Check system logs: `dmesg | tail -20` (Linux)
   - Monitor resources: `top`, `htop`, `iotop`

### How to Report a Mold Linker Bug

Include:
```
- Operating System: Ubuntu 22.04 (or macOS 14.x, or ...)
- Mold version: mold --version
- Rust version: rustc --version
- Reproducible case: Minimal code that fails
- Error message: Full output
- Environment: CI vs local, runner type
```

Example:
```
OS: Ubuntu 22.04 (GitHub Actions)
Mold: 2.4.1 release
Rust: 1.93.1
Error: undefined reference to `__libc_start_main'
Reproducible: cargo clean && cargo build --release --workspace
```

---

## Troubleshooting Checklist

- [ ] Verify mold is installed: `which mold && mold --version`
- [ ] Check Rust toolchain: `rustc --version && cargo --version`
- [ ] Ensure .cargo/config.toml has rustflags: `grep rustflags .cargo/config.toml`
- [ ] Test fallback without mold: `RUSTFLAGS="" cargo build`
- [ ] Verify binary runs: `./target/release/my_app --help`
- [ ] Run test suite: `cargo test --workspace --release`
- [ ] Check linker warnings: `cargo build -vv 2>&1 | grep -i "warning\|error"`
- [ ] Compare binary sizes: Before and after mold
- [ ] Review CI logs: If CI issue, check GitHub Actions output

---

## WP2.5 Part 1 Status: COMPLETE ✓

**Deliverable**: Troubleshooting guide with 6+ common issues ✓

**Next**: Rollback runbook, monitoring strategy, CHANGELOG update
