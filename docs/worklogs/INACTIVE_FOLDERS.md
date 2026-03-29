# Inactive Folders Audit

> Track orphaned, inactive, and non-canonical folders that need cleanup.

---

## 2026-03-29 - Full Git-State Audit of All Non-Canonical Dirs

**Status:** Research complete — action items catalogued
**Updated:** 2026-03-29

### Temp Directories (`~/CodeProjects/Phenotype/*-temp`)

| Dir | Remote | Branch | Dirty | Stashes | Unpushed | Action |
|-----|--------|--------|-------|---------|----------|--------|
| `agent-wave-monorepo-temp` | `KooshaPari/agent-wave` | `main` | 5 untracked docs/ | 0 | 0 | Commit or discard untracked docs files |
| `heliosCLI-monorepo-temp` | — | — | — | — | — | **DELETE** — empty directory |
| `phenotype-gauge-temp` | `KooshaPari/phenotype-gauge` | `chore/rescue-temp-dir-20260329` | 5 untracked docs/ | 1 | 1 commit | Push commit + pop stash + commit/discard untracked |
| `phenotype-go-kit-temp` | `KooshaPari/phenotype-go-kit` | `chore/rescue-temp-dir-20260329` | clean | 1 | 2 commits | Push 2 commits + pop/drop stash → open PR |
| `phenotype-nexus-temp` | `KooshaPari/phenotype-nexus` | `chore/rescue-temp-dir-20260329` | clean | 1 | 3 commits | Push 3 commits + pop/drop stash → open PR |
| `phenotype-shared-temp` | `KooshaPari/phenotype-shared` | `chore/sync-test-artifacts-20260329` | clean | 0 | 0 | **SAFE** — no action needed |
| `template-commons-temp` | `KooshaPari/template-commons` | `main` | `AGENTS.md`, `CLAUDE.md`, `worklog.md` | 0 | 0 | Commit or discard 3 tracked modified files |
| `tokenledger-temp` | `KooshaPari/tokenledger` | `main` | clean | 0 | 0 | **SAFE** — no action needed |

### Worktrees

| Dir | Remote | Branch | Dirty | Unpushed | Action |
|-----|--------|--------|-------|----------|--------|
| `repos/.worktrees/gh-pages-deploy` | none | none | — | 0 | **DELETE** — empty/orphaned, not a git repo |
| `repos/.worktrees/phench-fix` | none | none | — | 0 | **DELETE** — empty/orphaned, not a git repo |
| `repos/.worktrees/thegent` | `KooshaPari/phenotype-infrakit` | `chore/cost-tracking-modules` | 1 modified + 1 untracked | 1 commit | Push commit → open PR → delete after merge |
| `worktrees/phenotypeActions` | none | none | — | 0 | **DELETE** — empty/orphaned |
| `worktrees/portage` | `KooshaPari/portage` | `main` | clean | 0 | **SAFE** — clean canonical worktree |

### Isolated / Backups

| Dir | Type | Status | Action |
|-----|------|--------|--------|
| `isolated/agentapi-plusplus-postmerge-303-20260303-083936` | Post-merge snapshot | 5,324 dirty files (all untracked) | **ARCHIVE** → review, then delete |
| `isolated/agentapi-plusplus-postmerge-303-manual-20260303-084017` | Working copy snapshot | 29 dirty files, no commit history | **DELETE** — no history, snapshot only |
| `backups/4sgm-2` | Non-git backup | Contains Brewfile, cleanup.sh, docker-compose, PRD, ADR | **REVIEW** — may be system backup; keep or archive |

### `~/Repos` Spot-Check

| Repo | Branch | Status | Action |
|------|--------|--------|--------|
| `heliosCLI` | `refactor/decompose-text-manipulation` | 1 uncommitted file, off main | Commit + PR or stash + checkout main |
| `phenotype-shared` | `main` | 1 dirty file | Review + commit or discard |

### Registered Git Worktrees (in `repos/`)

```
/Users/kooshapari/CodeProjects/Phenotype/repos              [main]
/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/phenotype-infrakit/chore/merge-worklogs  [chore/merge-worklogs]
```

The `chore/merge-worklogs` worktree is registered but should be confirmed merged/deleted.

---

### Updated Cleanup Checklist (2026-03-29 v2)

#### IMMEDIATE — Safe Deletes (no unpushed work)

- [ ] DELETE `heliosCLI-monorepo-temp` (empty)
- [ ] DELETE `repos/.worktrees/gh-pages-deploy` (orphaned, not a git repo)
- [ ] DELETE `repos/.worktrees/phench-fix` (orphaned, not a git repo)
- [ ] DELETE `worktrees/phenotypeActions` (empty/orphaned)
- [ ] DELETE `isolated/agentapi-plusplus-postmerge-303-manual-20260303-084017` (no history)
- [ ] DELETE `worktree/` (empty)
- [ ] DELETE `add/` (empty)

#### SHORT-TERM — Push + PR + Delete

- [ ] `phenotype-go-kit-temp`: push 2 commits on `chore/rescue-temp-dir-20260329` → open PR → delete after merge
- [ ] `phenotype-nexus-temp`: push 3 commits + pop stash → open PR → delete after merge
- [ ] `phenotype-gauge-temp`: push 1 commit + pop stash + commit untracked docs → open PR → delete after merge
- [ ] `repos/.worktrees/thegent`: push 1 commit on `chore/cost-tracking-modules` → open PR → delete after merge
- [ ] `agent-wave-monorepo-temp`: commit or discard 5 untracked docs files → delete temp dir
- [ ] `template-commons-temp`: commit or discard `AGENTS.md`, `CLAUDE.md`, `worklog.md` changes

#### REVIEW NEEDED

- [ ] `isolated/agentapi-plusplus-postmerge-303-20260303-083936`: verify 5,324 files are all safely in upstream → delete
- [ ] `backups/4sgm-2`: determine if this is a system backup to preserve → move to archive or delete
- [ ] `~/Repos/heliosCLI`: commit or stash 1 dirty file; return to `main` or continue work
- [ ] `repos/worktrees/phenotype-infrakit/chore/merge-worklogs`: confirm merged → unregister worktree

---

_Last updated: 2026-03-29 (v2 git-state audit)_

---

## 2026-03-29 - Fresh Audit Findings

**Status:** Verified current state
**Updated:** 2026-03-29

### Orphaned Worktrees (`.worktrees/`)

| Directory | Git Status | Contents | Action |
|-----------|------------|----------|--------|
| `.worktrees/gh-pages-deploy/` | NOT GIT REPO | 30 dirs, stale | **DELETE** |
| `.worktrees/phench-fix/` | NOT GIT REPO | 30 dirs, stale | **DELETE** |
| `.worktrees/thegent/` | NOT GIT REPO | 3 dirs | **EVALUATE - contains docs/worklogs** |

### Empty Directories to Delete

| Directory | Status | Action |
|-----------|--------|--------|
| `worktree/` | EMPTY | DELETE |
| `add/` | EMPTY | DELETE |
| `.archive/audit/` | EMPTY | DELETE |
| `.archive/contracts/` | 1 file | REVIEW + DELETE |
| `.archive/kitty-specs/` | 1 file | REVIEW + DELETE |
| `.archive/plans/` | 1 file | REVIEW + DELETE |
| `.archive/schemas/` | 1 file | REVIEW + DELETE |
| `.archive/tests/` | 3 files | REVIEW + DELETE |

### Worktrees Folder (Non-Canonical)

| Directory | Status | Action |
|-----------|--------|--------|
| `worktrees/heliosCLI/` | Inactive | SYNC or DELETE |
| `repos/worktrees/` | EMPTY | DELETE |

---

## Cleanup Checklist (2026-03-29)

### IMMEDIATE (Execute Now)

- [ ] DELETE `.worktrees/gh-pages-deploy/` (30 dirs, stale)
- [ ] DELETE `.worktrees/phench-fix/` (30 dirs, stale)
- [ ] DELETE `worktree/` (empty)
- [ ] DELETE `add/` (empty)
- [ ] DELETE `repos/worktrees/` (empty)

### SHORT-TERM (This Week)

- [ ] EVALUATE `.worktrees/thegent/` - contains worklog changes
- [ ] REVIEW + DELETE `.archive/contracts/`
- [ ] REVIEW + DELETE `.archive/kitty-specs/`
- [ ] REVIEW + DELETE `.archive/plans/`
- [ ] REVIEW + DELETE `.archive/schemas/`
- [ ] REVIEW + DELETE `.archive/tests/`

### Git Cleanup

```bash
# phenotype-infrakit - CLEAN (no stash, clean working dir)
git status  # clean

# phenotype-docs - check for staged changes
cd /Users/kooshapari/CodeProjects/Phenotype/repos/docs
git status --short
```

---

## External Package Research Findings

**Status:** Research complete (2026-03-29)

### Fork/Wrap Opportunities (External 3rd Party)

| Package | Strategy | LOC Savings | Priority | Action |
|---------|----------|-------------|----------|--------|
| `casbin` | WRAP | 2-3k LOC | HIGH | Create `phenotype-policy-engine` wrapper |
| `eventually` | WRAP | 1.5k LOC | HIGH | Create `phenotype-event-sourcing` traits |
| `temporal-sdk` | WRAP | 3k LOC | MEDIUM | Long-running workflows |
| `tauri` | ADOPT | N/A | MEDIUM | Desktop agent UI |
| `zod` | BLACKBOX | 0.5k LOC | HIGH | API validation |
| `pydantic` | INSPIRE | N/A | MEDIUM | Study patterns |
| `xstate` | WRAP | 1k LOC | MEDIUM | Frontend FSM interop |
| `ra2a` | EVALUATE | ~200 LOC | P1 | A2A Protocol SDK |
| `mentisdb` | FORK CANDIDATE | ~400 LOC | P1 | Semantic memory |

### Integration Strategy Definitions

| Level | Description | Example |
|-------|-------------|---------|
| **BLACKBOX** | Direct dependency | `anyhow::Error` |
| **WHITEBOX** | Fork + modify | Custom fork of `eventually` |
| **WRAPPER** | Custom impl wrapping external | `phenotype-event-sourcing` wrapping `eventually` |
| **INSPIRATION** | Study patterns, implement differently | Study `casbin`, implement `phenotype-policy-engine` |
| **ADOPT** | Full adoption | `tauri` for desktop UI |

---

_Last updated: 2026-03-29 (Fresh audit)_

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
