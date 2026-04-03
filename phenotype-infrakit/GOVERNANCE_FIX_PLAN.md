# Governance Fix Plan: phenotype-infrakit Canonical Repository
**Generated**: 2026-03-30T21:00 UTC
**Status**: Diagnostic complete, awaiting user decision on merge strategy

---

## Current State Summary

### Repository Divergence
```
Local main:      31fdce55c (10 commits ahead of origin/main)
Origin/main:     ab9b0b9d0 (3 commits ahead of local main)
Merge base:      b1692d8ef (commit #481: stabilize workspace)
Commits diverged: 10 (local) + 3 (remote) = 13 total
```

### Local Working Tree State
- **Dirty files**: 21 modified/deleted
- **Untracked directories**: `KaskMan/`, `forgecode-fork/`, `.agileplus/.agileplus/`
- **Untracked files**: Various crate Cargo.toml files, new modules
- **Git stashes**: 6 (converting to commits per user preference)

### Changes Summary
**Local main (ahead of origin/main)**:
- 10 commits refactoring workspace members (bifrost-routing cleanup, phenotype-error-core enhancements, stub crate exclusion)
- State-machine type aliases
- Cargo.toml consolidation

**Origin/main (ahead of local main)**:
- ADR-015: Crate organization + PR size guidelines
- Comprehensive phenotype crate consolidation (PR #482)
- Workspace stabilization + clippy fixes (PR #481)

---

## Governance Violation Analysis

Per `/Users/kooshapari/CodeProjects/Phenotype/CLAUDE.md` (Worktree Rule):

> **Violation**: Canonical repository has 10 local commits ahead of upstream
> **Root cause**: Feature work committed directly to local main instead of feature branches + worktrees
> **Impact**: Cannot pull origin/main without merge conflict; canonical is out of sync with upstream

**Correct workflow** (what should happen):
1. Canonical repo stays on `origin/main` (pull-only)
2. Feature work goes to `.worktrees/<topic>/` (separate checkout)
3. Feature branches merge via PR (reviewed, tested)
4. Canonical pulls latest after PR merge

**Current workflow** (what happened):
1. Feature work committed directly to local main
2. 10 commits accumulated locally without pushing
3. origin/main moved ahead (3 new commits from other work)
4. Local main now diverges from origin/main

---

## Options for Remediation

### Option A: Merge Upstream (Recommended)
**Action**: Merge origin/main into local main, resolve conflicts, push.

**Pros**:
- ✅ Preserves all local work
- ✅ Fast-forwards canonical to latest upstream
- ✅ Creates clean merge commit with full history
- ✅ Allows reviewing what upstream changed (ADR-015, consolidation)

**Cons**:
- ⚠️ Creates merge commit (not linear history)
- ⚠️ Requires resolving conflicts (likely in Cargo.toml, crate members)
- ⚠️ Merge commit is "noisy" (10 local + 3 upstream = complex DAG)

**Steps**:
```bash
git fetch origin
git merge origin/main --no-ff  # Creates merge commit
# Resolve conflicts in Cargo.toml, crate configs
git add -A
git commit
git push origin main
```

---

### Option B: Rebase Local Work (Clean History)
**Action**: Rebase 10 local commits onto origin/main, force-push.

**Pros**:
- ✅ Linear history (no merge commit)
- ✅ Local commits appear "after" upstream changes
- ✅ Cleaner Git log

**Cons**:
- ❌ Rewrites history (force-push required)
- ❌ **Per global governance**: Force-push to main is discouraged (can overwrite upstream work)
- ❌ Harder to review what changed

**Risk**: Medium-High. If upstream has concurrent work, force-push can cause issues.

---

### Option C: Discard Local Work, Sync to Upstream
**Action**: Reset to origin/main, discard 10 local commits, start fresh.

**Pros**:
- ✅ Canonical immediately synced to upstream
- ✅ Clean state
- ✅ No merge conflicts

**Cons**:
- ❌ **Loses 10 commits of local work** (state-machine fixes, bifrost cleanup, error-core enhancements)
- ❌ Work must be manually recreated or cherry-picked
- ❌ Not recommended unless work is duplicated or invalid

**Risk**: High. Potential data loss.

---

### Option D: Create Feature Branch for Local Work
**Action**: Create feature branch from local main, reset main to origin/main, push feature branch.

**Pros**:
- ✅ Preserves all local work in named feature branch
- ✅ Canonical syncs to origin/main
- ✅ Feature branch can be reviewed and merged via PR
- ✅ Follows proper Git workflow

**Cons**:
- ⚠️ Requires PR review before merging back (proper governance)
- ⚠️ Extra step (branch creation, PR, merge)

**Steps**:
```bash
git branch feature/local-workspace-consolidation  # Preserve current work
git reset --hard origin/main                       # Sync canonical to upstream
git push origin feature/local-workspace-consolidation  # Push feature branch
# Then: Open PR to merge feature branch back to main
```

---

## Recommended Path Forward

### Step 1: Preserve Current State ✅ (DONE)
Current working tree archived to `.archive/pre-sync-state-2026-03-30/`:
- `working-tree.patch` — All unstaged changes
- `stashes.txt` — All 6 stash entries

### Step 2: Choose Remediation Strategy (USER DECISION)

**Recommendation**: **Option B (Rebase) OR Option D (Feature Branch)**

- **If local 10 commits are ready to merge**: Use Option B (rebase + push)
  - Cost: Resolve conflicts, force-push (medium risk)
  - Outcome: Linear history, all work integrated

- **If local 10 commits need review**: Use Option D (feature branch)
  - Cost: Create branch, open PR, merge via review
  - Outcome: Proper governance, no force-push, all work preserved

**NOT recommended**: Option A (merge) — creates noisy history with 13 diverged commits.
**NOT recommended**: Option C (discard) — loses work without justification.

---

## Immediate Next Actions (for User Decision)

1. **Review local 10 commits** — Are they ready to merge?
   ```bash
   git log origin/main..HEAD --oneline
   ```

2. **Review origin/main 3 commits** — What changed upstream?
   ```bash
   git log origin/main -3 --format="%h %s"
   # ab9b0b9d0 docs(adr): add ADR-015 for crate organization and PR guidelines
   # 753e72646 chore(workspace): comprehensive phenotype crate consolidation
   # b1692d8ef chore(phenotype-infrakit): stabilize workspace + fix clippy warnings
   ```

3. **Decide**: Rebase + push (Option B) or Feature branch + PR (Option D)?

---

## Risk Assessment

| Step | Risk | Mitigation |
|------|------|-----------|
| **Merge origin/main (Option A)** | Medium (conflicts, merge commit) | Already archived pre-state |
| **Rebase (Option B)** | Medium (force-push, history rewrite) | Limited to main; no concurrent pushes expected |
| **Feature branch (Option D)** | Low (no force-push, normal PR workflow) | Standard Git process; safe |
| **Discard (Option C)** | High (data loss) | ❌ NOT recommended without justification |

---

## Timeline

**Option B (Rebase + Push)**: ~5-10 min (resolve conflicts, push)
**Option D (Feature Branch + PR)**: ~15-20 min (branch, PR creation, review, merge)

---

## Decision Checklist

- [ ] User reviews local 10 commits (are they valid?)
- [ ] User reviews origin/main 3 commits (what changed upstream?)
- [ ] User chooses: Option B (rebase) OR Option D (feature branch)
- [ ] Execute chosen option
- [ ] Verify: `git status` shows "On branch main, nothing to commit"
- [ ] Verify: `git log origin/main..HEAD` shows 0 commits (in sync)

---

## Files Created

- `.archive/pre-sync-state-2026-03-30/working-tree.patch` — All uncommitted changes
- `.archive/pre-sync-state-2026-03-30/stashes.txt` — All 6 git stash entries

**To restore if needed**:
```bash
cd .archive/pre-sync-state-2026-03-30
git apply working-tree.patch
# OR review specific stash
git stash apply stash@{N}
```
