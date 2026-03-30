# Phenotype Workspace: Critical Work Items (Priority-Ordered)

**Last Updated:** 2026-03-30
**Status:** Ready to execute (16-19 pending PRs, 4 blocking issues)

---

## TIER 1: BLOCKING ISSUES (Must Fix First)

### T1-1: Fix phenotype-infrakit Build Failures
**Status:** CRITICAL BLOCKER
**Repo:** phenotype-infrakit
**Issue:** phenotype-crypto crate has compilation errors preventing workspace build
**Details:**
- Branch: `feat/phenotype-crypto-complete`
- Error source: See `CI_FAILURE_DIAGNOSTIC_REPORT.md` + `CI_REMEDIATION_PLAN.md`
- Blocking: T3-1, T3-2, T3-3
- **Action:** Debug CI diagnostics and apply fixes in feat/phenotype-crypto-complete
- **Est. Time:** 30-45 minutes
- **Acceptance:** `cargo build --workspace` succeeds with zero warnings

### T1-2: Resolve phenotype-bootstrap Submodule Changes
**Status:** BLOCKING REVIEW
**Repo:** phenotype-infrakit (submodule: repos/phenotype-bootstrap)
**Issue:** Submodule has 1 uncommitted file; unknown change content
**Details:**
- Current branch: main
- Commit: 6eb34dd ("Initial commit: phenotype-bootstrap system bootstrap")
- Change: 1 file modified
- **Action:** `cd repos/phenotype-bootstrap && git diff` to review; then commit or revert in main
- **Est. Time:** 10 minutes
- **Acceptance:** phenotype-infrakit main has no submodule warnings

### T1-3: Finalize heliosApp vite.config.ts
**Status:** READY
**Repo:** heliosApp
**Issue:** 1 uncommitted change (vite.config.ts) blocks PR merge
**Details:**
- Branch: `feat/fix-typescript-vite-federation` (1 commit ahead of origin)
- Change: vite.config.ts (Vite + TypeScript federation setup)
- **Action:** `git add vite.config.ts && git commit -m "chore: finalize vite federation config"` → create PR
- **Est. Time:** 5 minutes
- **Acceptance:** PR created and ready for review

### T1-4: Clean Up 196 Unmerged Branches
**Status:** TECH DEBT (low priority but recommended)
**Repo:** phenotype-infrakit
**Issue:** 196 local + remote branches creating clutter and confusion
**Details:**
- **To DELETE (70+):** All `remotes/origin/*` branches (already on GitHub; safe to delete)
- **To ARCHIVE (50+):** Old `chore/refine-*`, `chore/cleanup-*`, test branches
- **To KEEP (20):** Active feat/* and in-progress branches
- **Action:**
  ```bash
  # Delete remote tracking branches no longer on origin
  git branch -r | grep "remotes/origin/" | head -70 | xargs git branch -dr
  # Archive old local branches locally (no deletion)
  git branch -v | grep "cleanup\|refine" | awk '{print $1}' | wc -l
  ```
- **Est. Time:** 20-30 minutes
- **Acceptance:** `git branch | wc -l` shows <50 local branches

---

## TIER 2: READY FOR MERGE (Execute After T1)

### heliosCLI: 4 PRs Ready to Create & Merge
**Repo:** heliosCLI
**Status:** All branches clean; all ready

| Branch | PR Title | Target | Est. Time |
|--------|----------|--------|-----------|
| chore/gitattributes-proper | "chore: apply proper gitattributes for polyglot diffs" | main | 5 min |
| chore/fix-dep-drift-python | "chore: resolve Python dependency version drift" | main | 5 min |
| chore/governance-migration-hc | "chore: migrate to shared governance standards" | main | 5 min |
| chore/integrate-phenotype-docs | "chore: integrate @phenotype/docs package" | main | 5 min |

**Total Est. Time:** 20 minutes for all 4 PRs

### heliosApp: 1 PR Ready
**Repo:** heliosApp
**Status:** Blocked on T1-3

| Branch | PR Title | Target | Prerequisite |
|--------|----------|--------|--------------|
| feat/fix-typescript-vite-federation | "feat: add TypeScript + Vite module federation" | main | T1-3 complete |

**Est. Time:** 5 minutes (after T1-3)

### platforms/thegent: 3-4 PRs Ready to Create & Merge
**Repo:** platforms/thegent
**Status:** All branches ready; 7 files staged for merge

| Branch | PR Title | Target | Est. Time |
|--------|----------|--------|-----------|
| chore/add-thegent-specs-and-fixes | "chore: add thegent specs + fixes" | main | 10 min |
| chore/consolidate-dotfiles | "chore: consolidate dotfiles across workspace" | main | 15 min |
| chore/governance-agents-consolidation | "chore: consolidate agent governance patterns" | main | 10 min |
| docs/add-architecture-overview | "docs: add comprehensive thegent architecture overview" | main | 5 min |

**Total Est. Time:** 40 minutes for all 4 PRs

---

## TIER 3: IN PROGRESS (Track & Unblock)

### T3-1: Complete phenotype-crypto Implementation
**Status:** IN PROGRESS
**Repo:** phenotype-infrakit
**Branch:** feat/phenotype-crypto-complete
**Goal:** Full crypto library with signatures, encryption, key management
**Blocker:** T1-1 (build errors)
**Est. Remaining:** 60-90 minutes
**Acceptance Criteria:**
- All crypto modules compile
- Unit tests pass (signatures, encryption, key derivation)
- Security audit passes (no unsafe code warnings)
- Integration with phenotype-error-core complete

### T3-2: SDK Sanitize Atoms Implementation
**Status:** PLANNED
**Repo:** phenotype-infrakit
**Branch:** feat/phenosdk-sanitize-atoms*
**Goal:** Sanitize and normalize phenotype SDK atomic types
**Blocker:** T3-1 (depends on phenotype-crypto)
**Est. Remaining:** 45-60 minutes
**Acceptance Criteria:**
- All atom types have proper validation
- Tests for all sanitization rules pass
- AgilePlus spec WP01-WP04 complete

### T3-3: Workspace Dependencies & LOC Reduction
**Status:** READY (no blockers)
**Repo:** phenotype-infrakit
**Branch:** feat/loc-reduction-workspace-deps
**Goal:** Reduce workspace dependencies and consolidate crates
**Blocker:** None
**Est. Remaining:** 30-45 minutes
**Acceptance Criteria:**
- Cargo.toml dependencies reduced by 15-20%
- Duplicate crates consolidated into shared libs
- Workspace members optimized
- Zero new CVEs introduced

### T3-4: Merge All Worklogs into Main Documentation
**Status:** WAITING MERGE
**Repo:** phenotype-infrakit
**Branch:** chore/merge-worklogs
**Goal:** Consolidate all worklog entries into docs/worklogs/
**Blocker:** Cleanup of old log files
**Est. Remaining:** 20-30 minutes
**Acceptance Criteria:**
- All worklogs consolidated into single versioned file
- Old log files archived to `.archive/`
- No duplicate entries
- docs/worklogs/WORK_LOG.md updated

---

## TIER 4: TECHNICAL DEBT (Schedule After T1-3)

### T4-1: Clean Up Prunable Worktrees
**Status:** TECH DEBT
**Repo:** phenotype-infrakit
**Issue:** 5 worktrees marked prunable (detached or stale)
**Worktrees to prune:**
- `.worktrees/feat/cache-adapter-impl` (detached HEAD)
- `repos/worktrees/phenotype-infrakit/chore/merge-worklogs`
- `repos/worktrees/phenotype-infrakit/feat-phenosdk-sanitize-atoms`
- `repos/worktrees/phenotype-infrakit/stack-pr-1`
- `platforms/worktrees/thegent/consolidate-dotfiles`
- `repos/worktrees/AgilePlus/phenotype-docs`

**Action:** `git worktree prune`
**Est. Time:** 5 minutes
**Acceptance:** All worktrees listed in `git worktree list` are active

### T4-2: Archive Old & Stale Branches
**Status:** TECH DEBT
**Repo:** phenotype-infrakit
**Issue:** 100+ stale local branches create noise
**Branches to archive (move to `.archive/` tracking doc):**
- All test-* branches
- All recovery/* branches
- All refine-* branches older than 2 weeks
- All chore/cleanup-* branches completed

**Action:** Create `.archive/ARCHIVED_BRANCHES.txt` with list; delete from git
**Est. Time:** 20-30 minutes
**Acceptance:** `git branch | wc -l` < 50

### T4-3: Finalize Dependency Consolidation
**Status:** WAITING
**Repos:** phenotype-infrakit + helios repos
**Issue:** 50+ `refine-*` branches need final merge
**Branches to consolidate:**
- chore/refine-workspace-deps* (5 variants)
- chore/refine-errors-deps
- chore/refine-cache-* and chore/refine-retry-*

**Action:** Create single consolidation PR combining all refine-* work
**Est. Time:** 45-60 minutes
**Acceptance:** All workspace deps pinned to latest stable versions

### T4-4: Finalize Documentation Sync (Phase 2)
**Status:** IN PROGRESS
**Repos:** phenotype-infrakit + all nested repos
**Issue:** 20+ `docs/*` branches need merge
**Branches to consolidate:**
- docs/adr-002-event-sourcing-strategy
- docs/merge-spec-docs
- docs/phase1-2-arch-docs
- doc-sync-phase2

**Action:** Create consolidated docs PR with all spec + ADR + plan updates
**Est. Time:** 60-90 minutes
**Acceptance:**
- All ADRs have decision records
- All specs (PRD, FR, PLAN) are complete
- Cross-repo traceability map created

---

## EXECUTION CHECKLIST

### Phase 1: Unblock (45-50 min)
- [ ] **T1-1** Debug and fix phenotype-crypto build errors (30-45 min)
- [ ] **T1-2** Review + commit/revert phenotype-bootstrap submodule (10 min)
- [ ] **T1-3** Commit heliosApp vite.config.ts + create PR (5 min)

### Phase 2: Merge Ready Branches (60-70 min)
- [ ] **T2-1-4** Create & merge 4 heliosCLI PRs (20 min)
- [ ] **T2-5** Create & merge heliosApp PR (5 min after T1-3)
- [ ] **T2-6-8** Create & merge 3-4 thegent PRs (40 min)

### Phase 3: Finalize Major Features (180-210 min)
- [ ] **T3-1** Complete phenotype-crypto implementation (60-90 min)
- [ ] **T3-2** Implement phenosdk-sanitize-atoms (45-60 min after T3-1)
- [ ] **T3-3** Finalize workspace deps & LOC reduction (30-45 min)
- [ ] **T3-4** Merge all worklogs (20-30 min)

### Phase 4: Technical Debt (90-120 min)
- [ ] **T4-1** Prune worktrees (5 min)
- [ ] **T4-2** Archive old branches (20-30 min)
- [ ] **T4-3** Finalize dependency consolidation (45-60 min)
- [ ] **T4-4** Finalize documentation sync (60-90 min)

### Phase 5: Verification (15-30 min)
- [ ] **Verify:** All main branches building green
- [ ] **Verify:** All PRs merged and conflicts resolved
- [ ] **Verify:** Workspace deps audited (cargo audit)
- [ ] **Verify:** All tests passing locally

---

## CRITICAL PATHS & DEPENDENCIES

```
T1-1 (phenotype-crypto fix)
 ↓
T3-1 (phenotype-crypto complete)
 ↓
T3-2 (phenosdk-sanitize-atoms)

T1-2 (phenotype-bootstrap resolve)
 ↓
T1-3 (heliosApp vite.config)
 ↓
T2-5 (heliosApp PR)

T2-1-4 (heliosCLI PRs) — independent
T2-6-8 (thegent PRs) — independent

T3-3 (workspace deps) — can run parallel with T3-1, T3-2
T3-4 (merge worklogs) — independent

T4-1, T4-2 (cleanup) — can run after Phase 3
T4-3 (refine deps) — should wait for T3-3
T4-4 (docs sync) — can run parallel with others
```

**Critical Path Estimate:** 5-7 hours total (optimized with parallelization)

---

## SUCCESS CRITERIA

- [ ] All main branches on `main` (phenotype-infrakit, heliosApp, heliosCLI, thegent)
- [ ] phenotype-infrakit builds green: `cargo build --workspace` succeeds
- [ ] 0 uncommitted changes in any main repo
- [ ] 0 failing CI checks (except GitHub Actions billing issues)
- [ ] All 16-19 pending PRs merged
- [ ] Worktrees pruned; stale branches archived
- [ ] Documentation complete and in-sync across repos
