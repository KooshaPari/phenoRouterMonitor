# Active Worktrees Status - 2026-04-03

**Project:** [cross-repo]
**Category:** governance, worktrees
**Status:** completed
**Priority:** P1

---

## Summary

Audit of active git worktrees across heliosApp, portage, and heliosCLI projects. Identifies stalled work, consolidation opportunities, and cleanup candidates.

---

## heliosApp Worktrees

### In heliosApp-wtrees/ (Active)

| Worktree | Branch | Status | Files Changed | Action Required |
|----------|--------|--------|---------------|-----------------|
| **stabilize** | `fix/stabilize` | Uncommitted | 200+ | ⚠️ Review large changeset |
| **claude-md-standardize** | `docs/claude-md-standardize` | Ahead 9, Behind 60 | 80+ | Merge or close |
| **phase2-decompose** | `cleanup/local-work` | Ahead 7, Behind 642 | Many | Highly diverged |
| **launch-20260314** | `codex/launch-20260314` | Uncommitted | 60+ | Review |
| **code-reduction** | `feat/code-reduction-heliosapp` | Uncommitted | AGENTS.md, bun.lock | Review |
| **ci-workflow-fix** | `fix/ci-workflow-billing` | Staged | AGENTS.md | Review |
| **tech-debt-wave** | `tech-debt/wave-20260310` | Ahead 1 | Gate reports | OK |

### Canonical heliosApp (repos/heliosApp/)
- `main` branch: Synced with origin/main (0 ahead/0 behind)
- Local dirt: `.oxlintrc.json`, `docs/.vitepress/config.ts`

---

## portage Worktrees

### In portage-wtrees/

| Worktree | Status | Issue |
|----------|--------|-------|
| `hexagonal` | Dirty | 1 modified file: `src/harbor/llms/base.py` |
| `oxc-migration-20260303` | Ahead 4, Behind 2 | Needs sync |

### Canonical portage (repos/portage/)
- `main`: CLEAN (0 ahead/0 behind)

### Open PRs
- **PR #250**: Viewer lockfile cleanup - ✅ Ready to merge

---

## heliosCLI Worktrees

### In heliosCLI-wtrees/

| Worktree | Status | Issue |
|----------|--------|-------|
| **review-orchestrator** | Ahead 4038 commits | ⚠️ Massive divergence - probably cannot merge |
| **l2-memory-state** | Uncommitted | 100+ files modified |
| `modular-arch` | Clean | OK |
| `patch-superset-v2` | Ahead 3 | OK |
| `bazel-llvm-modules-fix` | Uncommitted | Modified MODULE.bazel |
| **release-v0.1.0** | Abandoned | Empty directory - DELETE |

### Canonical heliosCLI (repos/heliosCLI/)
- `main`: Synced (0/0)
- Local branches:
  - `chore/add-agileplus-governance`: ahead 4, behind 12
  - `fix/skip-billable-runs`: ahead 3, behind 20

---

## Cleanup Candidates

### High Priority (Delete)

| Path | Reason |
|------|--------|
| `/worktrees/heliosCLI/release-v0.1.0` | Abandoned release branch, empty |
| `heliosCLI-wtrees/main/` | Stale branch with staged deletions |

### Medium Priority (Investigate)

| Path | Issue | Action |
|------|-------|--------|
| `heliosCLI-wtrees/review-orchestrator` | 4038 ahead | Decide: merge, close, or archive |
| `heliosApp-wtrees/claude-md-standardize` | 80+ files, 60 behind | Merge or close |
| `heliosApp-wtrees/phase2-decompose` | 642 behind | Likely cannot merge cleanly |

---

## Consolidation Opportunities

| Duplicate | Locations | Issue |
|-----------|-----------|-------|
| `stabilize` branch | heliosApp-wtrees, heliosCLI-wtrees, phench-wtrees, parpour-wtrees | Overlapping development |
| Same branch checked out twice | `worktrees/thegent/chore/sync-docs-security-deps` AND `platforms/thegent` | Both have 7 unpushed commits |

---

## Recommendations

### Immediate Actions
1. Delete `/worktrees/heliosCLI/release-v0.1.0` (empty, abandoned)
2. Verify PR #250 merge (portage viewer lockfile)
3. Push `hexagonal` worktree changes in portage

### This Week
1. Decide fate of `heliosCLI/review-orchestrator` (4038 ahead)
2. Review `heliosApp/stabilize` (200+ modified files)
3. Merge or close `claude-md-standardize`

### This Quarter
1. Consolidate worktrees to single `.worktrees/` per ADR-0008
2. Standardize lifecycle: create → develop → push → PR → delete
3. Document worktree governance policy

---

## Worktree Governance Script

Recommended: Run `worktree_governance.sh oldest-first` to prioritize merging oldest worktrees.

Reference: `scripts/worktree_governance.sh` in thegent or agileplus

---

_Last updated: 2026-04-03_