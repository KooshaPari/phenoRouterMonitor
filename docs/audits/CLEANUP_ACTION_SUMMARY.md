# Workspace Cleanup — Quick Action Summary

**Date**: 2026-03-30
**Status**: Audit Complete — Ready for Execution
**Full Report**: `docs/audits/WORKSPACE_ORPHANS_AND_STALE_2026-03-30.md`
**Cleanup Script**: `scripts/workspace-cleanup.sh`

---

## One-Line Summary

Remove 13 orphaned stub crates, resolve 4 dirty worktrees, and clean up 2 stale ephemeral worktrees.

---

## Critical Issues (Run This Week)

### 1. Remove Ephemeral Worktrees from /tmp/
```bash
git worktree remove /private/tmp/phenotype-pr-workspace --force
git worktree remove /private/tmp/pr-236-resolution --force
```
**Why**: Both are >80 commits behind main and orphaned in temporary directory.

### 2. Recover Detached HEAD
```bash
cd .worktrees/feat/cache-adapter-impl
git status  # Verify 1 file change
git checkout -b feat/cache-adapter-impl-recovery
git add -A && git commit -m "Recovery: work from detached HEAD"
# Then create PR or decide merge strategy
```
**Why**: Detached HEAD is risky; needs attachment to branch.

### 3. Rebase Infrastructure Worktree
```bash
cd .worktrees/infrastructure/phase1-routing-aggregation
git rebase origin/main  # Resolve 4 commits behind
```
**Why**: 4 commits behind main; needs to merge or rebase.

### 4. Commit Root Worktree Changes
```bash
git add -A
git commit -m "chore: workspace state checkpoint — 2026-03-30"
```
**Why**: 47 files modified in root; prevents merge conflicts during cleanup.

---

## High-Priority Actions (Week 1)

### 5. Archive 13 Orphaned Stub Crates
```bash
mkdir -p .archive/orphaned-stubs-2026-03-30

mv crates/phenotype-async-traits \
   crates/phenotype-config-loader \
   crates/phenotype-contract \
   crates/phenotype-cost-core \
   crates/phenotype-crypto \
   crates/phenotype-error-macros \
   crates/phenotype-event-sourcing \
   crates/phenotype-http-client-core \
   crates/phenotype-macros \
   crates/phenotype-mcp \
   crates/phenotype-ports-canonical \
   crates/phenotype-process \
   crates/phenotype-test-infra \
   .archive/orphaned-stubs-2026-03-30/

git add .archive/orphaned-stubs-2026-03-30
git commit -m "chore: archive 13 orphaned stub crates — all 1-LOC stubs"
```
**Why**: All are empty placeholders (1 line of comment each); unclutters workspace and improves CI scan time.

### 6. Create PRs for Ready Worktrees
```bash
# From .worktrees/feat/phenotype-macros
gh pr create --title "feat: phenotype macros module" --body "Complete macro utilities"

# From .worktrees/phenotype-errors/consolidate
gh pr create --title "feat: unified error consolidation" --body "Consolidated error handling"
```
**Why**: Both are clean, synced, and ready to merge.

### 7. Review & Resolve Dirty Worktrees
```bash
# phenotype-crypto-complete (7 files modified)
cd .worktrees/feat/phenotype-crypto-complete
git status
# Decision: merge into feature PR, rebase, or stash

# phase2-routes-dashboard (6 files modified)
cd .worktrees/phase2-routes-dashboard
git status
# Decision: merge into feature PR, rebase, or stash
```
**Why**: Both have uncommitted work that must be resolved before cleanup.

---

## Automated Cleanup (Optional)

Use the cleanup script for semi-automated execution:

```bash
# Dry-run (preview all actions)
./scripts/workspace-cleanup.sh critical
./scripts/workspace-cleanup.sh phase1

# Execute critical phase
./scripts/workspace-cleanup.sh critical --execute

# Execute all phases
./scripts/workspace-cleanup.sh all -f
```

---

## Worktree Status Reference

| Worktree | Branch | Status | Action |
|----------|--------|--------|--------|
| `.worktrees/docs/adr-002-new` | `docs/adr-002-event-sourcing-strategy` | ✓ Clean | Keep (merged, documentation) |
| `.worktrees/feat/phenosdk-decompose-core` | `feat/phenosdk-decompose-core` | ✓ Clean | Keep (Phase 2 work) |
| `.worktrees/phenotype-string` | `feat/phenotype-string-complete` | ✓ Clean | Keep (merged into main) |
| `.worktrees/feat/phenotype-macros` | `feat/phenotype-macros` | → Ready | Create PR + merge |
| `.worktrees/phenotype-errors/consolidate` | `consolidate` | → Ready | Create PR + merge |
| `.worktrees/feat/cache-adapter-impl` | **DETACHED** | ⚠️ Risky | Recover to branch |
| `.worktrees/feat/phenotype-crypto-complete` | `feat/phenotype-crypto-complete` | ⚠️ Dirty | Review + merge/stash |
| `.worktrees/phase2-routes-dashboard` | `phase2-routes-dashboard` | ⚠️ Dirty | Review + merge/stash |
| `.worktrees/infrastructure/phase1-routing-aggregation` | `infrastructure/phase1-routing-aggregation` | ⚠️ Behind | Rebase on main |
| `/tmp/phenotype-pr-workspace` | `fix/add-http-client-core` | ✗ Remove | Delete (88 behind) |
| `/tmp/pr-236-resolution` | `resolve-236` | ✗ Remove | Delete (orphaned) |
| `.../decompose-sqlite-adapter` | `refactor/decompose-sqlite-adapter` | ? External | Standardize location |

---

## Orphaned Crates Reference

All 13 are **stubs with 1 line of comment**. No implementation, no tests, no dependencies.

```
phenotype-async-traits      → // phenotype-async-traits
phenotype-config-loader     → // phenotype-config-loader
phenotype-contract          → // phenotype-contract
phenotype-cost-core         → // phenotype-cost-core
phenotype-crypto            → // phenotype-crypto
phenotype-error-macros      → // phenotype-error-macros
phenotype-event-sourcing    → // phenotype-event-sourcing
phenotype-http-client-core  → // phenotype-http-client-core
phenotype-macros            → // phenotype-macros
phenotype-mcp               → // phenotype-mcp
phenotype-ports-canonical   → (no src/ dir)
phenotype-process           → // phenotype-process
phenotype-test-infra        → // phenotype-test-infra
```

---

## Verification Checklist

After cleanup:

- [ ] `git worktree list` shows only 9 worktrees (remove 2 from /tmp)
- [ ] `cargo check --workspace` builds cleanly
- [ ] `cargo clippy --workspace -- -D warnings` shows 0 warnings
- [ ] No crates/ subdirectories with Cargo.toml outside workspace members
- [ ] Root Cargo.toml workspace members matches filesystem crates/
- [ ] `.archive/orphaned-stubs-2026-03-30/` contains 13 crates
- [ ] All dirty worktrees resolved (merged/stashed)
- [ ] No detached HEADs in active worktrees

---

## Timeline

| Phase | Timeline | Items | Status |
|-------|----------|-------|--------|
| **Critical** | This week | 1, 2, 3, 4 | Start now |
| **Phase 1** | Week 1 | 5, 6, 7 | After critical |
| **Phase 2** | Week 2-3 | Standardization, monitoring | Long-term |

---

## Questions?

- **Full details**: `docs/audits/WORKSPACE_ORPHANS_AND_STALE_2026-03-30.md` (466 lines)
- **Automated execution**: `scripts/workspace-cleanup.sh --help`
- **Audit data**: Generated via `find`, `git worktree list`, `cargo metadata`

**Generated**: 2026-03-30
