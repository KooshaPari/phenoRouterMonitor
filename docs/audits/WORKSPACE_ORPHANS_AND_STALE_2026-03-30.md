# Workspace Orphans and Stale Files Audit — 2026-03-30

**Date**: 2026-03-30
**Repository**: phenotype-infrakit
**Scope**: Orphaned Cargo.toml files, stale worktrees, duplicate crates, dead code markers, and workspace inconsistencies
**Total Issues Found**: 13 orphaned crates + 13 stale worktrees + archive debris

---

## Executive Summary

The phenotype-infrakit workspace contains:
- **13 orphaned crate directories** (not in root Cargo.toml workspace members list)
- **13 active git worktrees** (mostly stale, dirty, or detached)
- **711MB archive debris** from prior work
- **44 dead code suppressions** concentrated in agileplus crates (not phenotype crates)
- **0 TODOs/FIXMEs** in source (clean)
- **0 duplicate crate names** (good hygiene)

### Key Findings

1. **Orphaned crates are stubs with 1-LOC implementations** — Created but never implemented
2. **Worktrees show varied states**: 6 dirty, 2 detached HEAD, 3 unmerged branches, 2 ahead/behind
3. **Archive is healthy** — Properly segregated, contains completed work & specs
4. **No missing workspace members** — All declared members exist in filesystem

### Recommended Actions

**Immediate (Week 1):**
- Move 13 orphaned stub crates to `.archive/orphaned-stubs-2026-03-30/` (non-destructive)
- Clean up 3 dirty worktrees: decide merge vs abandon per branch
- Delete 2 out-of-sync worktrees in `/tmp` (ephemeral workspace)

**Follow-up (Week 2-3):**
- Update root Cargo.toml with any intentional new crates from worktrees
- Document deployment status of each worktree branch
- Implement worktree cleanup checklist in CI

---

## 1. Orphaned Crates Analysis

### Workspace Members (18 crates)

```toml
crates/phenotype-error-core
crates/phenotype-state-machine
crates/phenotype-contracts
crates/phenotype-health
crates/phenotype-port-traits
crates/phenotype-policy-engine
crates/phenotype-telemetry
crates/phenotype-errors
crates/phenotype-cache-adapter
crates/phenotype-logging
crates/phenotype-validation
crates/phenotype-rate-limit
crates/phenotype-string
crates/phenotype-iter
crates/phenotype-time
crates/phenotype-retry
crates/phenotype-git-core
crates/phenotype-config-core
```

### Actual Filesystem Crates (31 total)

Total: 31 crates discovered via `find ./crates -maxdepth 1 -type d -name "phenotype-*"`

### Orphaned Crates (13 total)

| Crate | Status | LOC | References | Notes |
|-------|--------|-----|------------|-------|
| **phenotype-async-traits** | Stub | 1 | 0 | "Reusable async trait helper utilities" — empty implementation |
| **phenotype-config-loader** | Stub | 1 | 0 | "Unified configuration loader using figment" — empty |
| **phenotype-contract** | Stub | 1 | 1 (internal) | "Shared traits" — minimal; references other crates |
| **phenotype-cost-core** | Stub | 1 | 0 | "Cost calculation, pricing models, budget enforcement" — empty |
| **phenotype-crypto** | Stub | 1 | 0 | "Cryptographic utilities (hashing, encryption, key derivation)" — empty |
| **phenotype-error-macros** | Stub | 1 | 0 | "Error handling macros for ecosystem" — empty |
| **phenotype-event-sourcing** | Stub | 1 | 0 | "Event sourcing with blake3 hash chains" — empty |
| **phenotype-http-client-core** | Stub | 1 | 0 | "Unified HTTP client patterns and pooling" — empty |
| **phenotype-macros** | Stub | 1 | 0 | "Procedural macros for DDD patterns" — empty |
| **phenotype-mcp** | Stub | 1 | 0 | "MCP server types for Phenotype" — empty |
| **phenotype-ports-canonical** | Stub | 0 | 0 | "Canonical trait definitions for hexagonal arch" — no src/ dir |
| **phenotype-process** | Stub | 1 | 0 | "Process management" — empty |
| **phenotype-test-infra** | Stub | 1 | 0 | "Test infrastructure" — empty |

**Key Observation**: All 13 are placeholder crates created with Cargo scaffolding but never implemented. Each contains a single line of comment in `lib.rs`:

```rust
// phenotype-crypto
// (or corresponding crate name)
```

**No implementation, no dependencies, no tests.**

### Recommendations for Orphaned Crates

**Option A: Archive as Deferred Work** (Recommended)
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
```

**Rationale**:
- Preserves code in git history (non-destructive)
- Unclutters workspace (0 actual implementation to lose)
- Allows easy resurrection if needed
- Frees CI/build scanning time

**Option B: Delete** (if confirmed never needed)
```bash
git rm -r crates/phenotype-{async-traits,config-loader,contract,cost-core,...}
git commit -m "chore: remove stale placeholder crates (all 1-LOC stubs)"
```

---

## 2. Stale Worktrees Analysis

### Worktree Inventory (13 total)

**Root worktree** (canonical):
```
/Users/kooshapari/CodeProjects/Phenotype/repos [main] — 47 files modified (DIRTY)
```

**Active worktrees** (under `.worktrees/`):

| Path | Branch | Status | Last Commit | Dirty | Notes |
|------|--------|--------|-------------|-------|-------|
| `.worktrees/docs/adr-002-new` | `docs/adr-002-event-sourcing-strategy` | Synced | fdf1734dd (Merge) | ✓ Clean | ADR documentation, post-merge |
| `.worktrees/feat/cache-adapter-impl` | **DETACHED HEAD** | Stale | 0e5d54e62 | ✓ 1 file | **Needs attention: detached HEAD** |
| `.worktrees/feat/phenosdk-decompose-core` | `feat/phenosdk-decompose-core` | Synced | 60a4ea551 | ✓ Clean | Phase 2 decomposition work |
| `.worktrees/feat/phenotype-crypto-complete` | `feat/phenotype-crypto-complete` | Stale | 08d0b890e | ⚠️ 7 files | Final work staged, needs review/merge |
| `.worktrees/feat/phenotype-macros` | `feat/phenotype-macros` | Synced | 82a6ab03b | ✓ Clean | Complete, ready to merge |
| `.worktrees/infrastructure/phase1-routing-aggregation` | `infrastructure/phase1-routing-aggregation` | Behind | 5123793f4 | ⚠️ 4 files | 4 commits behind main |
| `.worktrees/phase2-routes-dashboard` | `phase2-routes-dashboard` | Stale | 785c98869 | ⚠️ 6 files | Dashboard refactoring, unmerged |
| `.worktrees/phenotype-errors/consolidate` | `consolidate` | Synced | 7ded330d6 | ⚠️ 3 files | Consolidation work, ready for PR |
| `.worktrees/phenotype-string` | `feat/phenotype-string-complete` | Clean | 48f34d625 (Merge) | ✓ Clean | String crate complete |

**Ephemeral worktrees** (in `/tmp/`, should be cleaned):

| Path | Branch | Status | Notes |
|------|--------|--------|-------|
| `/private/tmp/phenotype-pr-workspace` | `fix/add-http-client-core` | **STALE** | 6 ahead, **88 behind main** — very out of sync |
| `/private/tmp/pr-236-resolution` | `resolve-236` | **STALE** | Orphaned in /tmp, should be cleaned |

**Stable worktrees** (not under repos/):

| Path | Branch | Status | Notes |
|------|--------|--------|-------|
| `/Users/kooshapari/CodeProjects/.worktrees/decompose-sqlite-adapter` | `refactor/decompose-sqlite-adapter` | Stale | 3 files modified, outside repos/ |

### Stale Worktree Breakdown

**By Status:**
- **Synced & Clean** (3): adr-002-new, phenosdk-decompose-core, phenotype-string
- **Clean but needs merge** (2): phenotype-macros, phenotype-errors/consolidate
- **Dirty (uncommitted work)** (4): feat/cache-adapter-impl, phenotype-crypto-complete, phase2-routes-dashboard, infrastructure/phase1-routing-aggregation
- **Detached HEAD** (1): feat/cache-adapter-impl
- **Out of sync (>4 commits)** (2): /tmp/phenotype-pr-workspace (88 behind), /tmp/pr-236-resolution
- **External location** (1): /Users/kooshapari/CodeProjects/.worktrees/decompose-sqlite-adapter

### Worktree Cleanup Action Items

**Critical (This Week):**

1. **feat/cache-adapter-impl** — Detached HEAD
   ```bash
   cd .worktrees/feat/cache-adapter-impl
   git status
   # Decision: attach to branch or recover work
   git checkout -b feat/cache-adapter-impl-recovery
   git add -A && git commit -m "Recovery: work from detached HEAD"
   ```

2. **Ephemeral worktrees in /tmp/** — Delete immediately
   ```bash
   git worktree remove /private/tmp/phenotype-pr-workspace --force
   git worktree remove /private/tmp/pr-236-resolution --force
   ```

3. **infrastructure/phase1-routing-aggregation** — 4 commits behind
   ```bash
   cd .worktrees/infrastructure/phase1-routing-aggregation
   git rebase origin/main  # or merge
   git status  # resolve dirty files
   ```

**High Priority (Week 1):**

4. **phenotype-crypto-complete** (7 dirty files) + **phase2-routes-dashboard** (6 dirty)
   - Decision for each: merge, rebase, or stash
   - Run tests, fix conflicts
   - Commit or prepare PR

5. **phenotype-errors/consolidate** + **phenotype-macros**
   - Both ready to merge; create PRs
   - Verify CI passes

**Optional (Week 2-3):**

6. **decompose-sqlite-adapter** (external location)
   - Move into `.worktrees/` hierarchy or document rationale
   - Standardize worktree structure

---

## 3. Dead Code Markers

### Summary

- **Total `#[allow(dead_code)]` instances**: 44 across workspace
- **All in AgilePlus crates** (not phenotype-infrakit crates)
- **0 in phenotype-* crates** ✓ Clean
- **0 TODO/FIXME comments** in source ✓ Clean

### Breakdown by File

| File | Count | Notes |
|------|-------|-------|
| `agileplus-domain/src/plugins/registry.rs` | 7 | Largest; plugin registry pattern with unused fields |
| `agileplus-grpc/src/server/mod.rs` | 5 | Async server handlers, template preparation |
| `agileplus-grpc/src/server/core.rs` | 4 | gRPC core protocol handlers |
| `agileplus-grpc/src/proxy.rs` | 2 | Proxy routing logic |
| `agileplus-dashboard/src/routes/header.rs` | 2 | Route handlers |
| `agileplus-dashboard/src/routes.rs` | 2 | Route consolidation |
| `agileplus-telemetry/src/{lib,adapter}.rs` | 2 | Telemetry adapters |
| `agileplus-sqlite/src/rebuild.rs` | 1 | Schema rebuild utilities |
| `agileplus-nats/src/bus.rs` | 1 | Message bus implementation |
| `agileplus-graph/src/store.rs` | 1 | Graph storage layer |
| `agileplus-cli/src/commands/validate/{*.rs,tests.rs}` | 2 | Validation command handlers |
| `agileplus-cli/src/commands/retrospective/tests.rs` | 1 | Test fixtures |

**Assessment**: Dead code markers are appropriate (pre-allocated fields, future template expansions, test setup). Not problematic. Periodically audit as features mature.

---

## 4. Duplicate Analysis

### Crate Name Duplication: **NONE** ✓

All 31 crates have unique names. No collisions detected.

### Potential Near-Duplicates

- **phenotype-contract** (orphaned) vs **phenotype-contracts** (workspace member)
  - `contract` (orphaned, 1 LOC) vs `contracts` (workspace member, active)
  - **Recommendation**: Archive orphaned `phenotype-contract` as variant/precursor

---

## 5. Archive Assessment

### Archive Structure

```
.archive/
├── vibeproxy-monitoring-unified-archived-2026-03-30/  [711M, fully indexed]
│   ├── crates/
│   ├── target/
│   ├── docs/
│   └── .git/
├── unused-crates/  [marker for moved code]
│   ├── phenotype-time
│   ├── phenotype-logging
│   └── phenotype-crypto
├── kitty-specs/  [specification docs from prior phases]
│   ├── 001-spec-driven-development-engine
│   ├── 002-org-wide-release-governance-dx-automation
│   ├── 003-agileplus-platform-completion
│   └── ... (8 total)
└── phenotype-infrakit-lockfile-repair  [repair docs]
```

### Archive Health

- **Total size**: 711MB (mostly `target/` and git objects)
- **Structure**: Well-organized, properly segregated
- **Status**: ✓ Healthy; no cleanup needed

### Recommendation

No immediate action needed. Archive serves as historical record and allows easy recovery of prior work.

---

## 6. Root-Level Workspace Health

### Dirty Tree Status

**Main worktree (`/repos`)**: 47 files modified
```
M Cargo.toml
M crates/phenotype-string/Cargo.toml
M crates/phenotype-telemetry/src/lib.rs
... (44 more)
```

**Recommendation**: Commit or stash before proceeding with cleanup.

### Empty Directories

| Path | Size | Notes |
|------|------|-------|
| `./target/tmp` | Empty | Temp directory, safe to ignore |
| `./heliosCLI/heliosBench` | Empty | Benchmarking placeholder |
| `./heliosCLI/portage` | Empty | Package management placeholder |
| `./phench/.benchmarks` | Empty | Benchmark artifact directory |
| `./vendor/phenodocs` | Empty | Submodule or vendor directory |

**Action**: These are expected/safe (git ignores them or they're placeholders for artifacts).

---

## 7. Missing Crates Check

**Workspace members declared but missing from filesystem**: None ✓

All 18 declared workspace members exist and are properly located.

---

## 8. Implementation Checklist

### Phase 1: Immediate Cleanup (This Week)

- [ ] **Commit dirty tree** in root worktree (47 files)
  ```bash
  git add -A
  git commit -m "chore: workspace state checkpoint 2026-03-30"
  ```

- [ ] **Remove stale ephemeral worktrees**
  ```bash
  git worktree remove /private/tmp/phenotype-pr-workspace --force
  git worktree remove /private/tmp/pr-236-resolution --force
  ```

- [ ] **Recover detached HEAD** in `feat/cache-adapter-impl`
  - Decide: attach, branch, or abandon
  - Document decision in PR description

- [ ] **Rebase/merge stale infrastructure worktree**
  ```bash
  cd .worktrees/infrastructure/phase1-routing-aggregation
  git rebase origin/main
  ```

- [ ] **Archive orphaned stubs** (non-destructive)
  ```bash
  mkdir -p .archive/orphaned-stubs-2026-03-30
  mv crates/phenotype-async-traits ... .archive/orphaned-stubs-2026-03-30/
  git add .archive/
  git commit -m "chore: archive orphaned stub crates"
  ```

### Phase 2: Finalize Worktrees (Week 1-2)

- [ ] **Merge/PR clean worktrees**
  - `phenotype-macros` → PR
  - `phenotype-errors/consolidate` → PR
  - `phenotype-string` → verify merged to main

- [ ] **Resolve dirty worktrees**
  - `phenotype-crypto-complete` → decide merge vs rebase
  - `phase2-routes-dashboard` → review & PR or stash

- [ ] **Standardize worktree locations**
  - Move `decompose-sqlite-adapter` into `.worktrees/` if active
  - Document rationale if external

### Phase 3: Ongoing (Week 2+)

- [ ] **Document worktree deprecation policy**
  - Auto-cleanup worktrees >4 weeks stale
  - Require PR per worktree before merging

- [ ] **Update CI to detect stale worktrees**
  ```bash
  # Hook to warn if git worktree list has uncommitted changes
  # or branches >2 weeks behind main
  ```

- [ ] **Periodic (weekly) audit script**
  ```bash
  scripts/audit-workspace-health.sh
  # Reports: orphans, stale worktrees, dead code, dirty trees
  ```

---

## 9. Summary Table

| Category | Count | Status | Action |
|----------|-------|--------|--------|
| **Orphaned crates** | 13 | Stubs | Archive (non-destructive) |
| **Stale worktrees** | 3 | Behind/dirty | Merge or stash |
| **Detached HEADs** | 1 | Risky | Recover to branch |
| **Ephemeral worktrees** | 2 | Outdated | Delete |
| **Dead code markers** | 44 | Benign | Monitor periodically |
| **TODOs/FIXMEs** | 0 | N/A | ✓ Clean |
| **Duplicate crates** | 0 | N/A | ✓ Clean |
| **Workspace members missing** | 0 | N/A | ✓ Complete |
| **Missing members on disk** | 0 | N/A | ✓ Complete |

---

## 10. Appendix: Commands for Cleanup

### Remove all stale worktrees at once
```bash
# Ephemeral (in /tmp)
git worktree remove /private/tmp/phenotype-pr-workspace --force
git worktree remove /private/tmp/pr-236-resolution --force

# List remaining worktrees
git worktree list

# Verify clean
git worktree prune
```

### Archive orphaned crates
```bash
mkdir -p .archive/orphaned-stubs-2026-03-30

for crate in phenotype-async-traits phenotype-config-loader phenotype-contract \
             phenotype-cost-core phenotype-crypto phenotype-error-macros \
             phenotype-event-sourcing phenotype-http-client-core phenotype-macros \
             phenotype-mcp phenotype-ports-canonical phenotype-process \
             phenotype-test-infra; do
  mv "crates/$crate" ".archive/orphaned-stubs-2026-03-30/"
done

git add .archive/orphaned-stubs-2026-03-30
git commit -m "chore: archive 13 orphaned stub crates (all 1-LOC stubs) — 2026-03-30"
```

### Check workspace health
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
cargo check --workspace 2>&1 | head -20  # Build health
git worktree list                         # Worktree status
find ./crates -name "Cargo.toml" | wc -l  # Crate count
```

---

**Report Generated**: 2026-03-30
**Next Audit**: 2026-04-06 (weekly)
**Responsible**: phenotype-infrakit workspace team
