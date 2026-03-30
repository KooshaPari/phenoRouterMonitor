# Phenotype Workspace Scan Index

**Scan Date:** 2026-03-30
**Scope:** Complete cross-repo status audit (phenotype-infrakit + 4 nested repos + 16 worktrees)
**Status:** All work identified and prioritized for execution

---

## Overview

This index provides quick access to all workspace scan deliverables and actionable work items. The scan identified **16-19 pending PRs**, **4 blocking issues**, and **196 unmerged branches** across the Phenotype workspace requiring coordination and execution.

---

## Quick Links to Reports

### 1. Critical Work Items (START HERE)
**File:** `/Users/kooshapari/CodeProjects/Phenotype/repos/docs/reference/WORKSPACE_CRITICAL_WORK_ITEMS.md`
**Type:** Actionable checklist
**Length:** 294 lines
**Purpose:** Priority-ordered execution plan with 4-tier structure (Blocking → Ready → In-Progress → Debt)

**Contents:**
- Tier 1: 4 blocking issues requiring immediate fix (45-60 min)
- Tier 2: 8 ready-to-merge PRs (60-70 min)
- Tier 3: 4 in-progress features to track (180-210 min)
- Tier 4: 4 technical debt items (90-120 min)
- Phase 1-5 execution checklist
- Dependency graph showing critical paths
- Success criteria with acceptance tests

**Best for:** Quick reference during execution; use this to track progress

---

### 2. Comprehensive Status Report
**File:** `/Users/kooshapari/CodeProjects/Phenotype/repos/docs/reports/CROSS_REPO_WORKSPACE_STATUS_2026_03_30.md`
**Type:** Detailed audit
**Length:** 370 lines
**Purpose:** Complete snapshot of all repos, branches, and dependencies

**Contents:**
- Executive summary with 5 critical issues
- Detailed status for each repo:
  - phenotype-infrakit (monorepo): 17 changes, 196 unmerged branches
  - heliosApp: 1 change, 2 branches
  - heliosCLI: clean, 5 branches
  - platforms/thegent: 7 changes, 8 branches
  - phenotype-bootstrap: 1 change (submodule)
  - phenotype-replication-engine: clean (submodule)
- Worktree status: 11 active, 5 prunable
- Cross-repo dependencies and integration points
- Complete unmerged branches list with categorization
- Build/test status notes

**Best for:** Understanding the full workspace state; reference during PR reviews

---

## Key Metrics at a Glance

| Metric | Value | Notes |
|--------|-------|-------|
| **Total Repos** | 6 | 1 monorepo + 5 nested repos |
| **Repos on `main`** | 4 | phenotype-infrakit, thegent, phenotype-bootstrap, phenotype-replication-engine |
| **Repos on feature branches** | 2 | heliosApp (Vite federation), heliosCLI (governance) |
| **Build Status** | 1 failing | phenotype-infrakit (phenotype-crypto compilation errors) |
| **Unmerged Branches** | 196 | phenotype-infrakit only |
| **Uncommitted Changes** | 19 files | Across main repos (mostly infrakit) |
| **Pending PRs** | 16-19 | Ready to create + merge |
| **Worktrees Active** | 11 | Feature work in progress |
| **Worktrees Prunable** | 5 | Detached heads + stale |
| **Total Timeline** | 5-7 hours | Estimated (can parallelize to 3-4 hours) |

---

## Blocking Issues Requiring Immediate Action

### Issue 1: phenotype-crypto Build Failures (T1-1)
**Repo:** phenotype-infrakit
**Branch:** feat/phenotype-crypto-complete
**Blocker:** Prevents all other work on phenotype-infrakit
**Est. Time:** 30-45 minutes
**Action:** Read `CI_FAILURE_DIAGNOSTIC_REPORT.md` and `CI_REMEDIATION_PLAN.md`; fix compilation errors
**Success:** `cargo build --workspace` succeeds

### Issue 2: phenotype-bootstrap Submodule Uncommitted (T1-2)
**Repo:** phenotype-infrakit (submodule: repos/phenotype-bootstrap)
**Problem:** 1 file modified; content unknown
**Est. Time:** 10 minutes
**Action:** Review changes with `git diff`; commit or revert in main
**Success:** Zero submodule warnings in `git status`

### Issue 3: heliosApp vite.config.ts Uncommitted (T1-3)
**Repo:** heliosApp
**Branch:** feat/fix-typescript-vite-federation
**Problem:** 1 change uncommitted; blocks PR merge
**Est. Time:** 5 minutes
**Action:** Commit vite.config.ts; create PR → main
**Success:** PR created and reviewable

### Issue 4: Branch Cleanup (T1-4) - Optional
**Repo:** phenotype-infrakit
**Problem:** 196 unmerged branches creating clutter
**Est. Time:** 20-30 minutes (optional)
**Action:** Archive 100+ old branches; keep 20-30 active
**Success:** `git branch | wc -l` < 50

---

## Ready-to-Merge PRs (16-19 Total)

### heliosCLI: 4 PRs
1. `chore/gitattributes-proper` → main (5 min)
2. `chore/fix-dep-drift-python` → main (5 min)
3. `chore/governance-migration-hc` → main (5 min)
4. `chore/integrate-phenotype-docs` → main (5 min)

**Total: 20 minutes**

### heliosApp: 1 PR (after T1-3)
1. `feat/fix-typescript-vite-federation` → main (5 min)

**Total: 5 minutes (prerequisite: T1-3)**

### platforms/thegent: 4 PRs
1. `chore/add-thegent-specs-and-fixes` → main (10 min)
2. `chore/consolidate-dotfiles` → main (15 min)
3. `chore/governance-agents-consolidation` → main (10 min)
4. `docs/add-architecture-overview` → main (5 min)

**Total: 40 minutes**

### phenotype-infrakit: 8-10 PRs (queued, depends on T1-1 fix)
- feat/phenotype-crypto-complete
- feat/phenosdk-sanitize-atoms* (multiple variants)
- feat/loc-reduction-workspace-deps
- chore/merge-worklogs
- chore/consolidate-dotfiles
- Others (see WORKSPACE_CRITICAL_WORK_ITEMS.md)

**Total: 180-210 minutes (after T1-1)**

---

## In-Progress Features (Track & Unblock)

| Feature | Branch | Blocker | Status | Est. Time |
|---------|--------|---------|--------|-----------|
| phenotype-crypto | feat/phenotype-crypto-complete | Build errors (T1-1) | BLOCKED | 60-90 min |
| phenosdk-sanitize-atoms | feat/phenosdk-sanitize-atoms | phenotype-crypto | QUEUED | 45-60 min |
| Workspace deps reduction | feat/loc-reduction-workspace-deps | None | READY | 30-45 min |
| Merge worklogs | chore/merge-worklogs | Cleanup | READY | 20-30 min |

---

## Technical Debt (Cleanup After Phase 3)

1. **Worktree Cleanup** — Prune 5 stale worktrees (5 min)
2. **Branch Archival** — Move 100+ old branches to tracking doc (20-30 min)
3. **Dependency Consolidation** — Finalize refine-*deps PRs (45-60 min)
4. **Documentation Sync** — Merge all docs/* branches (60-90 min)

**Total: 90-120 minutes (optional but recommended)**

---

## Execution Phases (Reference)

### Phase 1: Unblock (45-60 min)
Fix the 4 blocking issues that prevent further progress

### Phase 2: Merge Ready (60-70 min)
Create and merge all ready-to-merge PRs (16 PRs across 3 repos)

### Phase 3: Finalize Features (180-210 min)
Complete in-progress major features and merge to main

### Phase 4: Cleanup (90-120 min)
Archive branches, prune worktrees, consolidate dependencies

### Phase 5: Verify (15-30 min)
Run acceptance tests and verify all builds green

---

## Files Created by This Scan

| File | Type | Size | Purpose |
|------|------|------|---------|
| WORKSPACE_CRITICAL_WORK_ITEMS.md | Checklist | 10 KB | Priority-ordered execution plan |
| CROSS_REPO_WORKSPACE_STATUS_2026_03_30.md | Report | 15 KB | Complete status audit |
| WORKSPACE_SCAN_INDEX.md | Index | This file | Navigation + quick reference |

---

## How to Use These Documents

### For Quick Status
1. Read this file (WORKSPACE_SCAN_INDEX.md) — 5 minutes
2. Check "Blocking Issues" section above
3. Check "Ready-to-Merge PRs" section above

### For Execution
1. Open WORKSPACE_CRITICAL_WORK_ITEMS.md
2. Follow the "Execution Checklist" in Phase order
3. Mark items as you complete them
4. Use success criteria to verify completion

### For Understanding
1. Read CROSS_REPO_WORKSPACE_STATUS_2026_03_30.md for full details
2. Review "Repository Details" section for per-repo status
3. Review "Cross-Repo Dependencies" section for integration points
4. Review "Worktree Status" section for branch-to-worktree mapping

---

## Critical Path (What Must Happen in Order)

```
START: T1-1 (fix phenotype-crypto) — 30-45 min
  ↓
THEN: T3-1 (complete phenotype-crypto) — 60-90 min
  ↓
THEN: T3-2 (phenosdk-sanitize-atoms) — 45-60 min
  ↓
PARALLEL:
  • T2-1-4 (heliosCLI 4 PRs) — 20 min
  • T2-6-8 (thegent 4 PRs) — 40 min
  • T3-3 (workspace deps) — 30-45 min
  ↓
THEN: T3-4 (merge worklogs) — 20-30 min
  ↓
OPTIONAL: T4-1-4 (cleanup) — 90-120 min

TOTAL: 5-7 hours (optimized with parallelization to 3-4 hours)
```

---

## Success Criteria (When Done)

- [ ] All repos on `main` branch (4/4 main repos)
- [ ] phenotype-infrakit builds green (`cargo build --workspace` ✓)
- [ ] All 16-19 PRs merged
- [ ] 0 uncommitted changes in any main repo
- [ ] 5 worktrees pruned
- [ ] 100+ stale branches archived
- [ ] All tests passing locally

---

## Contact & Questions

For questions about this scan or the work items:
1. Check the "Critical Path" section above
2. Review WORKSPACE_CRITICAL_WORK_ITEMS.md for detailed acceptance criteria
3. Review CROSS_REPO_WORKSPACE_STATUS_2026_03_30.md for full repo status

---

**Last Updated:** 2026-03-30 05:40 MST
**Report Generator:** Phenotype Workspace Scan Tool
**Next Scan:** Recommend 1 week after completing Phase 3-5
