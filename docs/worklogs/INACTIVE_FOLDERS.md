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

| Project | Date | Can Restore? |
|---------|------|--------------|
| (Empty/Check contents) | - | - |

---

## Cleanup Checklist

- [ ] DELETE `.worktrees/gh-pages-deploy`
- [ ] DELETE `.worktrees/phench-fix`
- [ ] PUSH `.worktrees/thegent` to origin
- [ ] CREATE PR for thegent pending changes
- [ ] DELETE after PR merge + review
- [ ] DELETE `add/` (empty)
- [ ] MERGE `worktree/` into `.worktrees/` if needed

---

_Last updated: 2026-03-29_
