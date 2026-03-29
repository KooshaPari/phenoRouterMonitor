# Inactive Folders Audit

> Track orphaned, inactive, and non-canonical folders that need cleanup.

---

## Orphaned Worktrees (`.worktrees/`)

**Status:** Need cleanup

### `.worktrees/gh-pages-deploy`

| Property | Value |
|----------|-------|
| Status | ORPHANED - Not a git repository |
| Git Worktree | No |
| Last Activity | Unknown |
| Action | **DELETE** |

```bash
rm -rf /Users/kooshapari/CodeProjects/Phenotype/repos/.worktrees/gh-pages-deploy
```

### `.worktrees/phench-fix`

| Property | Value |
|----------|-------|
| Status | ORPHANED - Not a git repository |
| Git Worktree | No |
| Last Activity | Unknown |
| Action | **DELETE** |

```bash
rm -rf /Users/kooshapari/CodeProjects/Phenotype/repos/.worktrees/phench-fix
```

### `.worktrees/thegent`

| Property | Value |
|----------|-------|
| Status | ACTIVE - 1 commit ahead of origin |
| Git Worktree | No (manual copy) |
| Last Commit | 72418c5c9 |
| Action | **PUSH + CREATE PR** |

```bash
# From thegent worktree
git status  # Shows 1 commit ahead
git push origin main
# Then create PR
```

**Changes staged:**
- `docs/worklogs/DEPENDENCIES.md`
- `src/thegent/__init__.py` (new)
- `src/thegent/adapters/*` (new)
- `src/thegent/agents/*` (new)

---

## Canonical vs Non-Canonical Folders

### Confirmed Canonical Folders

| Path | Purpose | Status |
|------|---------|--------|
| `crates/` | Rust workspace crates | CANONICAL |
| `libs/` | Phenotype shared libraries | CANONICAL |
| `src/` | Main source code | CANONICAL |
| `docs/` | Documentation | CANONICAL |
| `worklogs/` | Work tracking | CANONICAL |
| `sessions/` | Session logs | CANONICAL |

### Non-Canonical Folders (Review)

| Path | Purpose | Status | Action |
|------|---------|--------|--------|
| `.worktrees/` | Stray worktree copies | REVIEW | Clean orphaned |
| `.benchmarks/` | Benchmark artifacts | OK | Keep |
| `.archive/` | Archived projects | OK | Keep |
| `add/` | Empty directory | DELETE | Empty |
| `worktree/` | Duplicate worktree | MERGE | Merge into `.worktrees/` |

---

## Archive Status

### `.archive/` Contents

Projects moved to archive:

| Subdirectory | Files | Status | Action |
|--------------|-------|--------|--------|
| `audit/` | 0 | EMPTY | DELETE |
| `contracts/` | 1 | Minimal | REVIEW + DELETE |
| `kitty-specs/` | 1 | Minimal | REVIEW + DELETE |
| `plans/` | 1 | Minimal | REVIEW + DELETE |
| `schemas/` | 1 | Minimal | REVIEW + DELETE |
| `tests/` | 3 | Minimal | REVIEW + DELETE |

### `.worktrees/` Contents

| Directory | Git Status | Files | Action |
|-----------|------------|-------|--------|
| `gh-pages-deploy/` | NOT A GIT REPO | 30 dirs | DELETE |
| `phench-fix/` | Unknown | 30 dirs | DELETE |
| `thegent/` | NOT A GIT REPO | 3 dirs | PUSH + PR |

### `worktrees/` Contents

| Directory | Status | Files | Action |
|-----------|--------|-------|--------|
| `heliosCLI/` | Inactive worktree | 3 dirs | SYNC or DELETE |

### `worktree/` Contents

| Directory | Status | Action |
|-----------|--------|--------|
| `worktree/` | EMPTY | DELETE |

---

## 2026-03-29 Updated Cleanup Checklist

### IMMEDIATE (This Session)

- [ ] DELETE `.worktrees/gh-pages-deploy` (NOT a git repo - 30 dirs of stale content)
- [ ] DELETE `.worktrees/phench-fix` (NOT a git repo - 30 dirs of stale content)
- [ ] DELETE `worktree/` (empty)
- [ ] DELETE `add/` (empty)

### SHORT-TERM (This Week)

- [ ] PUSH `.worktrees/thegent` to origin/main
- [ ] CREATE PR for thegent pending changes
- [ ] REVIEW `worktrees/heliosCLI/` - determine canonical location
- [ ] REVIEW + DELETE `.archive/contracts/`
- [ ] REVIEW + DELETE `.archive/kitty-specs/`
- [ ] REVIEW + DELETE `.archive/plans/`
- [ ] REVIEW + DELETE `.archive/schemas/`
- [ ] REVIEW + DELETE `.archive/tests/`

### MEDIUM-TERM (This Month)

- [ ] Verify deleted items don't break any references
- [ ] Update `.gitignore` if needed
- [ ] Clean up merged git branches

---

## Git Branch Cleanup

### Local Branches to Delete

```bash
git branch -d fix/phench-tests-1
git branch -d chore/worklog-consolidation
```

### Remote Branches to Delete

```bash
git push origin --delete chore/spec-docs
git push origin --delete chore/vitepress-docs
git push origin --delete chore/worklog-*
git push origin --delete docs/consolidate-worklog-notes
```

### Stashed Changes to Review

```bash
# Review before dropping
git stash show -p stash@{0}
git stash drop stash@{0}  # After review
```

---

_Last updated: 2026-03-29_
