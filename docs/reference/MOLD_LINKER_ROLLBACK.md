# Mold Linker - Rollback Runbook

**Status**: Reference Documentation
**Date**: 2026-03-31
**Scope**: Step-by-step procedures to disable or revert mold linker

---

## When to Rollback

Initiate rollback if any of the following occur:

| Condition | Severity | Action |
|-----------|----------|--------|
| Linker errors, undefined symbols | CRITICAL | Immediate rollback |
| Binary corruption or crashes | CRITICAL | Immediate rollback |
| Security vulnerability in mold | HIGH | Immediate rollback |
| Link time regression >20% | MEDIUM | Investigate, then rollback if unresolved |
| Build fails repeatedly in CI | MEDIUM | Rollback, investigate, retry |
| mold version incompatibility | MEDIUM | Update mold or rollback |

---

## Rollback Procedures

### Procedure 1: Disable Mold (Temporary) — FASTEST

**Use Case**: Quick disable without git changes

**Time**: ~30 seconds

**Steps**:

1. **Comment out rustflags** in `.cargo/config.toml`:
   ```bash
   nano .cargo/config.toml
   # or
   vim .cargo/config.toml
   ```

2. **Find this line**:
   ```toml
   rustflags = ["-C", "link-arg=-fuse-ld=mold"]
   ```

3. **Comment it out**:
   ```toml
   # rustflags = ["-C", "link-arg=-fuse-ld=mold"]
   ```

4. **Save file** (Ctrl+O, Enter, Ctrl+X in nano)

5. **Clean build directory**:
   ```bash
   cargo clean
   ```

6. **Test**:
   ```bash
   cargo build --release
   # Expected: Builds successfully with default linker
   ```

7. **Verify**:
   ```bash
   cargo test --workspace --release
   # Expected: All tests pass
   ```

**Rollback complete**. No git commit needed (temporary).

**To re-enable**: Uncomment the line and repeat step 5-6.

---

### Procedure 2: Override with Environment Variable — NO FILE EDIT

**Use Case**: Disable mold for single command without changing config

**Time**: ~10 seconds

**Steps**:

1. **Run build with empty RUSTFLAGS**:
   ```bash
   RUSTFLAGS="" cargo build --release
   ```

2. **Verify it works**:
   ```bash
   ./target/release/my_app --version
   ```

3. **To make permanent** (until session ends):
   ```bash
   export RUSTFLAGS=""
   cargo build --release
   cargo test --workspace --release
   ```

4. **To revert** (re-enable mold):
   ```bash
   unset RUSTFLAGS
   cargo build --release
   ```

**Advantage**: No file changes; perfect for testing.

---

### Procedure 3: Git Checkout (Full Revert) — SAFE

**Use Case**: Revert all changes to original state

**Time**: ~1 minute

**Steps**:

1. **Check current state**:
   ```bash
   git diff .cargo/config.toml
   ```

2. **Revert file**:
   ```bash
   git checkout .cargo/config.toml
   ```

3. **Verify reverted**:
   ```bash
   git diff .cargo/config.toml
   # Should show no differences
   cat .cargo/config.toml | grep rustflags
   # Should be empty (no mold config)
   ```

4. **Clean build**:
   ```bash
   cargo clean
   cargo build --release
   ```

5. **Verify**:
   ```bash
   cargo test --workspace --release
   ```

**Advantage**: Guaranteed to revert to known-good state.

---

### Procedure 4: Commit Rollback (Permanent Revert) — TRACKED

**Use Case**: Formally revert mold in git history

**Time**: ~2 minutes

**Steps**:

1. **Use Procedure 1** (comment out rustflags)

2. **Stage the change**:
   ```bash
   git add .cargo/config.toml
   ```

3. **Verify staged changes**:
   ```bash
   git diff --cached .cargo/config.toml
   ```

4. **Commit**:
   ```bash
   git commit -m "chore(build): disable mold linker (temporary rollback)"
   ```

5. **Verify commit**:
   ```bash
   git log -1 --oneline
   # Should show: chore(build): disable mold linker (temporary rollback)
   ```

6. **Test again**:
   ```bash
   cargo clean
   cargo build --release --workspace
   cargo test --workspace --release
   ```

7. **Push to branch** (if working on feature branch):
   ```bash
   git push origin feature/my-branch
   ```

**Advantage**: Full git history; easy to revert again later.

---

### Procedure 5: Update or Downgrade Mold — VERSION MANAGEMENT

**Use Case**: Fix mold-specific issues by changing version

**Time**: ~3-5 minutes

**Steps**:

1. **Check current mold version**:
   ```bash
   mold --version
   ```

2. **Update mold** (if available):
   ```bash
   # On Linux
   sudo apt update && sudo apt upgrade -y mold

   # On macOS
   brew upgrade mold
   ```

3. **Verify new version**:
   ```bash
   mold --version
   ```

4. **Test build**:
   ```bash
   cargo clean
   cargo build --release --workspace
   ```

5. **If issue persists, downgrade**:
   ```bash
   # On Linux (specify version)
   sudo apt install mold=2.3.1  # Example: downgrade to 2.3.1

   # On macOS
   brew install mold@2.3.1  # If available
   # Otherwise: Uninstall and reinstall from source
   ```

6. **Verify fix**:
   ```bash
   mold --version
   cargo clean && cargo build --release --workspace
   ```

---

## GitHub Actions Rollback

### Disable Mold in CI (Temporary)

**Use Case**: CI job fails with mold; need quick fix

**Procedure**:

1. **Edit `.github/workflows/benchmark.yml`**:
   ```bash
   nano .github/workflows/benchmark.yml
   ```

2. **Find the mold-link-benchmark job**:
   ```yaml
   mold-link-benchmark:
     name: Mold Linker Benchmark
     runs-on: ubuntu-latest
   ```

3. **Disable the entire job**:
   ```yaml
   # mold-link-benchmark:
   #   name: Mold Linker Benchmark
   #   runs-on: ubuntu-latest
   # ... rest of job commented out
   ```

4. **Commit and push**:
   ```bash
   git add .github/workflows/benchmark.yml
   git commit -m "chore(ci): temporarily disable mold benchmark"
   git push origin main
   ```

5. **CI will now skip mold job** on next workflow trigger

6. **To re-enable**: Uncomment the job and push again

**Advantage**: No impact on main codebase; only workflow disabled.

---

## Verification Checklist

After rollback, verify these checklist items:

### Build & Compilation
- [ ] `cargo clean` completes without errors
- [ ] `cargo build --release` completes without errors
- [ ] Build time is reasonable (45-60s expected without mold)
- [ ] No compilation warnings introduced

### Testing
- [ ] `cargo test --workspace --release` passes
- [ ] `cargo clippy --workspace -- -D warnings` shows zero errors
- [ ] `cargo fmt --check` shows code is properly formatted

### Binary Verification
- [ ] Binary is executable: `./target/release/my_app --version`
- [ ] Binary size is expected
- [ ] Binary runs without segfaults or crashes

### Git State
- [ ] `git status` shows expected changes
- [ ] `.cargo/config.toml` is in desired state
- [ ] CI workflow is correct (if CI was rolled back)

### Documentation
- [ ] CHANGELOG updated (if committed)
- [ ] Team notified (if permanent change)
- [ ] Issue tracking updated (if bug-driven)

---

## Timeline & RTO

| Procedure | Time | Impact | RTO |
|-----------|------|--------|-----|
| Comment out line | 30s | Zero; just rebuild | Immediate |
| Environment variable | 10s | Single command | Immediate |
| Git revert | 1m | Back to known state | <2 min |
| Commit rollback | 2m | Tracked change | <3 min |
| mold update/downgrade | 3-5m | System change | <10 min |
| GitHub workflow disable | 3m | CI skip | <5 min |

---

## Escalation

### If Rollback Doesn't Fix Issue

1. **Document the problem**:
   ```bash
   # Capture error message
   cargo build --release 2>&1 | tee build_error.log

   # System info
   uname -a > system_info.txt
   rustc --version >> system_info.txt
   cargo --version >> system_info.txt
   mold --version >> system_info.txt
   ```

2. **Report to mold project**:
   - Open issue: https://github.com/rui314/mold/issues
   - Include: OS, mold version, Rust version, error message, reproducible case

3. **Use default linker** (permanent):
   - Keep rollback in place
   - File issue with Rust team if needed
   - Consider alternative linker solutions

---

## Post-Rollback Tasks

### After Temporary Rollback

1. **Investigate root cause**:
   - Was it mold version incompatibility?
   - Was it workspace change?
   - Was it environment-specific?

2. **Test fix**:
   - Update mold to latest
   - Update Rust to latest
   - Test on clean machine

3. **Re-enable mold**:
   - Uncomment rustflags
   - Run full test suite
   - Verify performance improvement

### After Permanent Rollback

1. **Document decision**:
   - Create ADR (Architecture Decision Record)
   - Title: "Disable mold linker due to [reason]"
   - Include: Date, reason, investigation findings

2. **Update CHANGELOG**:
   ```markdown
   ## [x.x.x] - YYYY-MM-DD

   ### Changed
   - Disabled mold linker due to [compatibility issue / performance regression]
   - Reverted to default GNU ld linker
   ```

3. **Notify team**:
   - Email/Slack: Team summary
   - Include: Reason, impact, timeline to fix

---

## WP2.5 Part 2 Status: COMPLETE ✓

**Deliverable**: Rollback runbook with 5 procedures ✓

**Key Takeaways**:
- Rollback is FAST (30s to 1m)
- Procedure 1 (comment out) is simplest
- Procedure 3 (git checkout) is safest
- All procedures preserve git history
- Zero impact on main codebase

**Next**: Monitoring strategy, CHANGELOG update
