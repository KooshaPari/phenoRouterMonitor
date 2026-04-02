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
| `heliosCLI-monorepo-temp` | — | — | — | — | — | **DELETED** |
| `phenotype-gauge-temp` | `KooshaPari/phenotype-gauge` | `chore/rescue-temp-dir-20260329` | 5 untracked docs/ | 1 | 1 commit | Push commit + pop stash + commit/discard untracked |
| `phenotype-go-kit-temp` | `KooshaPari/phenotype-go-kit` | `chore/rescue-temp-dir-20260329` | clean | 1 | 2 commits | Push 2 commits + pop/drop stash → open PR |
| `phenotype-nexus-temp` | `KooshaPari/phenotype-nexus` | `chore/rescue-temp-dir-20260329` | clean | 1 | 3 commits | Push 3 commits + pop/drop stash → open PR |
| `phenotype-shared-temp` | `KooshaPari/phenotype-shared` | `chore/sync-test-artifacts-20260329` | clean | 0 | 0 | **SAFE** — no action needed |
| `template-commons-temp` | `KooshaPari/template-commons` | `main` | `AGENTS.md`, `CLAUDE.md`, `worklog.md` | 0 | 0 | Commit or discard 3 tracked modified files |
| `tokenledger-temp` | `KooshaPari/tokenledger` | `main` | clean | 0 | 0 | **SAFE** — no action needed |
| `consolidate-libraries` | `KooshaPari/phenotype-infrakit` | `chore/decomposition-audit-v2` | clean | 0 | 0 | **PRUNABLE** (Already in HEAD) |
| `expand-test-coverage` | `KooshaPari/phenotype-infrakit` | `chore/ci-cd-workflows-clean` | clean | 0 | 0 | **PRUNABLE** |

### Worktrees

| Dir | Remote | Branch | Dirty | Unpushed | Action |
|-----|--------|--------|-------|----------|--------|
| `repos/.worktrees/gh-pages-deploy` | none | none | — | 0 | **DELETED** |
| `repos/.worktrees/phench-fix` | none | none | — | 0 | **DELETED** |
| `repos/.worktrees/thegent` | `KooshaPari/phenotype-infrakit` | `chore/cost-tracking-modules` | 1 modified + 1 untracked | 1 commit | Push commit → open PR → delete after merge |
| `worktrees/phenotypeActions` | none | none | — | 0 | **DELETED** |
| `worktrees/portage` | `KooshaPari/portage` | `main` | clean | 0 | **SAFE** — clean canonical worktree |
| `worktrees/chore-docs-sbom-stack` | origin | `chore/tag-automation-release-split` | clean | 0 | **ACTIVE** |
| `worktrees/chore-sbom-cyclonedx` | origin | `chore/sbom-cyclonedx-pilot` | clean | 0 | **ACTIVE** |
| `worktrees/chore-session-sbom-stack` | origin | `chore/session-stacked-sbom-delivery` | clean | 0 | **ACTIVE** |
| `worktrees/phenosdk-wave-a-contracts-impl` | origin | `feat/phenosdk-wave-a-contracts` | clean | 0 | **ACTIVE** |
| `repos/worktrees/AgilePlus/phenotype-docs` | agileplus | `chore/integrate-phenotype-docs` | clean | 1022+ | **CRITICAL REVIEW** |
| `merge-spec-docs` | origin | `docs/merge-spec-docs` | clean | 57 | **HIGH REVIEW** |

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

## 2026-03-30 - Extended Inactive Folders Audit (Wave 112)

**Status:** Research complete — updated action items
**Updated:** 2026-03-30

### Category 1: Canonical Shelf Folders (Ensure Synced)

These are legitimate worktrees that need verification but contain active or recently-pushed work:

| Directory | Remote | Branch | Unpushed | Status | Action |
|-----------|--------|--------|----------|--------|--------|
| `worktrees/portage/` | `KooshaPari/portage` | `main` | 0 | **SAFE** | No action needed |
| `.worktrees/merge-spec-docs/` | origin | `docs/merge-spec-docs` | 57 | **ACTIVE** | Push + PR review |
| `worktrees/chore-docs-sbom-stack/` | origin | `chore/tag-automation-release-split` | 0 | **ACTIVE** | No action needed |
| `worktrees/chore-sbom-cyclonedx/` | origin | `chore/sbom-cyclonedx-pilot` | 0 | **ACTIVE** | No action needed |
| `worktrees/chore-session-sbom-stack/` | origin | `chore/session-stacked-sbom-delivery` | 0 | **ACTIVE** | No action needed |
| `worktrees/phenosdk-wave-a-contracts-impl/` | origin | `feat/phenosdk-wave-a-contracts` | 0 | **ACTIVE** | No action needed |
| `platforms/worktrees/thegent/consolidate-dotfiles/` | — | `chore/consolidate-dotfiles` | — | **REVIEW** | Confirm status |

### Category 2: Temporary/Inactive Folders (Mark for Cleanup)

#### A. `*-temp` Directories

| Directory | Remote | Branch | Dirty | Stashes | Unpushed | Cleanup Action |
|-----------|--------|--------|-------|---------|----------|----------------|
| `agent-wave-monorepo-temp/` | `KooshaPari/agent-wave` | `main` | 5 untracked docs/ | 0 | 0 | **DELETE** — discard untracked docs |
| `heliosCLI-monorepo-temp/` | — | — | — | — | — | **DELETED** (2026-03-29) |
| `phenotype-gauge-temp/` | `KooshaPari/phenotype-gauge` | `chore/rescue-temp-dir-20260329` | 5 untracked docs/ | 1 | 1 | **DELETE** — push commit first |
| `phenotype-go-kit-temp/` | `KooshaPari/phenotype-go-kit` | `chore/rescue-temp-dir-20260329` | clean | 1 | 2 | **DELETE** — push 2 commits, drop stash |
| `phenotype-nexus-temp/` | `KooshaPari/phenotype-nexus` | `chore/rescue-temp-dir-20260329` | clean | 1 | 3 | **DELETE** — push 3 commits, drop stash |
| `phenotype-shared-temp/` | `KooshaPari/phenotype-shared` | `chore/sync-test-artifacts-20260329` | clean | 0 | 0 | **SAFE** |
| `template-commons-temp/` | `KooshaPari/template-commons` | `main` | 3 modified files | 0 | 0 | **DELETE** — discard or commit changes |
| `tokenledger-temp/` | `KooshaPari/tokenledger` | `main` | clean | 0 | 0 | **SAFE** |

#### B. `*-wtrees` Directories

| Directory | Status | Action |
|-----------|--------|--------|
| `phenotype-shared-wtrees/resolve-pr58/` | **BROKEN** — empty container, no `.git` | **DELETE** |
| `heliosCLI-wtrees/experimental-mcp/` | **DELETED** — PR #114 merged |
| `heliosCLI-wtrees/main/` | Stale branch, 4 deletions staged | **DELETE** — PR merged |
| `phenotype-shared-wtrees/` | Origin main synced, no stashes | **PURGE** |

#### C. `.worktrees/` Directory (Orphaned Entries)

| Directory | Git Status | Contents | Action |
|-----------|------------|----------|--------|
| `.worktrees/gh-pages-deploy/` | **NOT GIT REPO** | 30 dirs, stale | **DELETE** |
| `.worktrees/phench-fix/` | **NOT GIT REPO** | 30 dirs, stale | **DELETE** |
| `.worktrees/thegent/` | **NOT GIT REPO** | 1 file (23 LOC phench.py stub) | **DELETE** |

#### D. Git Metadata Entries (Stale - Need Pruning)

| Metadata Entry | Target | Status | Fix |
|----------------|--------|--------|-----|
| `.git/worktrees/resolve-pr57` | `/private/tmp/resolve-pr57/.git` | **BROKEN** | `git worktree prune` |
| `.git/worktrees/resolve-pr581` | `/private/tmp/resolve-pr58/.git` | **BROKEN** | `git worktree prune` |
| `.git/worktrees/merge-spec-docs` | `.worktrees/merge-spec-docs/.git` | **BROKEN** | `git worktree prune` |
| `.git/worktrees/merge-worklogs` | `repos/worktrees/.../.git` | **BROKEN** | `git worktree prune` |
| `.git/worktrees/resolve-pr58` | `phenotype-shared-wtrees/.../.git` | **BROKEN** | `git worktree prune` |

### Critical Review Items

| Directory | Issue | Priority | Action |
|----------|-------|----------|--------|
| `repos/worktrees/AgilePlus/phenotype-docs/` | **1022+ unpushed commits** | **P0 CRITICAL** | Review and push or discard |
| `worktrees/merge-spec-docs/` | 57 unpushed commits | **P1 HIGH** | Push + PR review |
| `.archive/orphaned-worktrees/consolidate-libraries/` | 299MB, commits already in HEAD | **DELETE** | Safe to remove |
| `.archive/orphaned-worktrees/expand-test-coverage/` | 403MB | **REVIEW** | Verify branch status |

### Cleanup Execution Plan

#### Immediate (Safe Deletes — No Unpushed Work)

```bash
# Orphaned .worktrees/ copies
rm -rf .worktrees/gh-pages-deploy
rm -rf .worktrees/phench-fix
rm -rf .worktrees/thegent

# Stale -wtrees directories
rm -rf phenotype-shared-wtrees
rm -rf heliosCLI-wtrees

# Git metadata cleanup
git worktree prune --verbose

# Empty archive entries
rm -rf .archive/orphaned-worktrees/consolidate-libraries
```

### Storage Recovery Summary

| Category | Count | Disposition |
|----------|-------|-------------|
| **Canonical Shelf (Synced)** | 7 | Keep, verify periodically |
| **Safe to Delete** | 11 | Delete immediately |
| **Need Review** | 3 | Review before action |
| **Git Metadata Prune** | 5 | Run `git worktree prune` |

**Storage Recovery Potential:** ~800MB+ from orphaned worktrees (`expand-test-coverage`: 403MB, `consolidate-libraries`: 299MB, plus various temp dirs)

---

_Last updated: 2026-03-30 (Wave 112 audit)_

---

## 2026-03-30 - Wave 94 Cleanup Execution

**Status:** Cleanup actions executed
**Updated:** 2026-03-30

### Actions COMPLETED in Wave 94

| Item | Action | Result |
|------|--------|--------|
| `.worktrees/thegent/` | DELETED | Contained only 1 file (23 LOC phench.py stub) - orphaned |
| `.archive/audit/` | DELETED | Empty directory |
| `.archive/libs-archived/` | DELETED | Empty directory |
| `.archive/contracts/` | DELETED | 26 bytes (just a README) |
| `.archive/schemas/` | DELETED | 24 bytes (just a README) |

### Current .archive/ Contents

| Directory | Size | Contents | Decision |
|-----------|------|----------|----------|
| `.archive/kitty-specs/` | 16KB | phenotype-infrakit-lockfile-repair spec | **KEEP** - contains actual spec docs |
| `.archive/plans/` | 24KB | DUPLICATION_MERGED-v1.md | **KEEP** - reference document |
| `.archive/tests/` | 84KB | test_phench_runtime.py (2120 LOC) | **REVIEW NEEDED** - exists in 3 other locations |
| `.archive/temp-directories/` | 13MB | 4 temp clone dirs | **REVIEW NEEDED** - stashes/commits |
| `.archive/orphaned-worktrees/` | 702MB | 2 old worktrees | **REVIEW NEEDED** - 299MB + 403MB |

### .archive/orphaned-worktrees/ Analysis

| Worktree | Size | Branch | Commits | Status |
|----------|------|--------|---------|--------|
| `consolidate-libraries` | 299MB | `chore/decomposition-audit-v2` | 4 | Commits already in current HEAD |
| `expand-test-coverage` | 403MB | `chore/ci-cd-workflows-clean` | 1 | Different branch than active |

### Remaining Action Items

#### IMMEDIATE (Safe Deletes)
- [ ] DELETE `.archive/orphaned-worktrees/consolidate-libraries` (commits already in HEAD, 299MB)

#### SHORT-TERM (Review Required)
- [ ] REVIEW `.archive/orphaned-worktrees/expand-test-coverage` (403MB, different branch)
- [ ] REVIEW `.archive/tests/test_phench_runtime.py` (exists in 3 other locations)
- [ ] REVIEW `.archive/temp-directories/` (13MB, contains stashes)

#### P0 - CRITICAL
- [ ] **REVIEW** `repos/worktrees/AgilePlus/phenotype-docs` - 1022 unpushed commits

#### P1 - HIGH  
- [ ] PUSH + PR each worktree with unpushed commits

---

_End of Wave 94_

---

## 2026-03-30 - Wave 95 PR Creation Summary

**Status:** PRs created
**Updated:** 2026-03-30

### PRs Created in phenotype-infrakit

| PR # | Title | Branch | URL |
|------|-------|--------|-----|
| #95 | feat(ci): add SBOM generation workflow | `feat/add-sbom-workflow` | https://github.com/KooshaPari/phenotype-infrakit/pull/95 |
| #96 | feat(event-sourcing): LOC reduction | `feat/event-sourcing-loc-reduction` | https://github.com/KooshaPari/phenotype-infrakit/pull/96 |

### LOC Reduction Achievement
- **-406 LOC net** in phenotype-event-sourcing (5 files: -494 deleted, +88 added)

### AggressivePlus Worktree Notes
- `.worktrees/merge-spec-docs` has 57 commits ahead of main
- Branches have no common ancestor - would need force-push or manual merge
- phenotype-docs worktree already has commits pushed to agileplus/main

### Remaining Action Items

#### P1 - HIGH  
- [ ] REVIEW + MERGE PRs #95 and #96
- [ ] CLEANUP: After PR merge, remove worktrees with no history in common
- [ ] REVIEW `.archive/orphaned-worktrees/consolidate-libraries` - DELETE (299MB, commits already in HEAD)
- [ ] REVIEW `.archive/orphaned-worktrees/expand-test-coverage` - DELETE (403MB)

#### P2 - MEDIUM
- [ ] Push remaining branches in `.worktrees/merge-spec-docs` (chore/consolidate-cost-tracking, etc.)
- [ ] Resolve branch conflicts in AgilePlus worktrees

---

_End of Wave 95_

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

- [x] DELETE `.worktrees/gh-pages-deploy` (NOT a git repo - 30 dirs of stale content)
- [x] DELETE `.worktrees/phench-fix` (NOT a git repo - 30 dirs of stale content)
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

## 2026-03-30 — Wave 92: reconciliation, deep verification, automation hooks

**Status:** methodology + re-verify  
**Updated:** 2026-03-30

### Conflicting prior notes (must re-verify on disk)

Earlier sections disagree on whether `repos/.worktrees/thegent` (or repo-root `.worktrees/thegent`) is a **valid linked worktree** vs a **non-git folder**. Before any delete:

1. `cd` to the directory and run `git rev-parse --is-inside-work-tree`.
2. From the **main** checkout of that repository, run `git worktree list` and confirm the path appears.
3. If Git reports `fatal: not a git repository: …/.git/worktrees/…`, treat as **stale registration** → remove empty dirs → `git worktree prune` → re-add only if still needed.

### Expanded temp-clone matrix (lifecycle)

| Stage | Check | Pass |
|-------|--------|------|
| Start | `git worktree add` or explicit temp clone path | Path under agreed hub (`worktrees/`, not random root) |
| During | `git status`, `git log origin/main..HEAD` | No surprise large untracked trees |
| Land | PR merged on GitHub | Required |
| End | `git worktree remove <path>`; `git worktree prune` | No stale `.git/worktrees/*` gitdir targets |

### Deeper codebase audit targets (monorepo)

| Area | What to diff / consolidate | Worklog sink |
|------|----------------------------|--------------|
| `crates/*/src/**/error*.rs` | error enums and `thiserror` | `DUPLICATION.md`, `PLANS/ERROR_CORE_EXTRACTION.md` |
| `crates/**/ports/**` | overlapping traits vs `libs/` | `ARCHITECTURE.md` |
| `libs/phenotype-shared/crates/**` | edition + workspace membership | `DUPLICATION.md`, `PLANS/EDITION_MIGRATION.md` |
| `python/**` | overlapping pydantic models / clients | `DEPENDENCIES.md` |
| `docs/worklogs/**` | duplicate audit narratives | `README.md` index only |

### Remote vs local parity (per clone)

```bash
git fetch origin
git status -sb
git branch -vv
git stash list
git remote -v
git rev-list --left-right --count HEAD...@{upstream} 2>/dev/null || true
```

### Automation spec (implement at **repository root**, not in `docs/`)

| Artifact | Purpose |
|----------|---------|
| `scripts/git-worktree-health.sh` | Parse porcelain; assert paths; shared `git-common-dir`; validate `.git/worktrees/*/gitdir`; optional `GIT_HYGIENE_ORPHAN_SCAN=1` |
| `task git:hygiene` | Strict check |
| `task git:hygiene:verbose` / `git:hygiene:dirty` | `-v` / `-w` |
| CI step | `bash scripts/git-worktree-health.sh` after checkout |

### Consolidated “done means” checklist

- [ ] No broken `git worktree list` entries (health script green).
- [ ] No long-lived `*-temp` with unpushed commits.
- [ ] `isolated/*` either reset to `origin/main` or deleted after archive.
- [ ] PRs closed for rescue branches; local temps removed.
- [ ] This file’s tables updated in the same session as cleanup.

---

_Last updated: 2026-03-30 (Wave 92 appendix)_

---

## 2026-03-29 - Deep Structural Audit & Detailed Worktree Analysis

**Status:** Comprehensive directory and git metadata inspection completed
**Updated:** 2026-03-29 11:00 UTC
**Focus:** Non-canonical folders, orphaned worktrees, broken metadata

### Key Discovery: Stray Root Directory

| Path | Type | Size | Modified | Finding | Severity | Action |
|---|---|---|---|---|---|---|
| `/.C/` | Directory | 0 files | 2026-03-29 08:42:35 | Single character hyphen folder at repo root — likely from incomplete command | **CRITICAL** | **DELETE IMMEDIATELY** |

This appears to be a stray flag parameter that became a directory, possibly from `cd -C` or similar command error.

---

### Empty Root-Level Directories (Cleanup Batch 1)

| Path | Type | Files | Size | Modified | Purpose | Action |
|---|---|---|---|---|---|---|
| `/worktree/` | Dir | 0 | 0B | 2026-03-29 08:42:35 | Duplicate of `worktrees/` with singular naming | **DELETE** |
| `/add/` | Dir | 0 | 0B | 2026-03-29 08:42:35 | Unclear; empty | **DELETE** |
| `/plans/` | Dir | 0 | 0B | 2026-03-29 08:42:35 | Legacy/archive naming; specs moved elsewhere | **DELETE** |

All three are completely empty. Safe to remove without data loss.

---

### Broken Git Worktree Metadata References (5 Orphaned Entries)

**Problem:** `.git/worktrees/` contains 5 metadata entries pointing to non-existent or stale directories.
These create clutter and confusion in `git worktree list` output.

| Metadata Entry | Target Path | Status | Reason | Git Command |
|---|---|---|---|---|
| `.git/worktrees/resolve-pr57` | `/private/tmp/resolve-pr57/.git` | **BROKEN** | Worktree in /tmp; /tmp is ephemeral and gets cleaned | `git worktree prune` |
| `.git/worktrees/resolve-pr581` | `/private/tmp/resolve-pr58/.git` | **BROKEN** | Same issue; /tmp cleanup | `git worktree prune` |
| `.git/worktrees/merge-spec-docs` | `.worktrees/merge-spec-docs/.git` | **BROKEN** | Directory exists but no .git subdirectory | `git worktree prune` |
| `.git/worktrees/merge-worklogs` | `repos/worktrees/phenotype-infrakit/chore/merge-worklogs/.git` | **BROKEN** | Pointer is stale; nested at wrong path | `git worktree prune` |
| `.git/worktrees/resolve-pr58` | `phenotype-shared-wtrees/resolve-pr58/.git` | **BROKEN** | Container directory exists but no .git | `git worktree prune` |

**Solution:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
git worktree prune --verbose
git worktree list  # Verify clean output
```

---

### Orphaned Stale Copies (NOT Git Repos — Filesystem Copies Only)

| Path | Files | Dirs | Git Repo | Modified | Issue | Action |
|---|---|---|---|---|---|---|
| `/.worktrees/gh-pages-deploy/` | 20 | 9 | NO | 2026-03-29 08:32:56 | Orphaned filesystem copy; no `.git` link | **DELETE** |
| `/.worktrees/phench-fix/` | 20 | 9 | NO | 2026-03-29 08:30:28 | Orphaned filesystem copy; no `.git` link | **DELETE** |
| `/.worktrees/thegent/` | 0 (empty) | 2 | NO | 2026-03-29 08:35:04 | Empty container; no content | **DELETE** |

These are **not** actual git worktrees (no `.git` linkage), just orphaned directory copies. Likely from failed `git worktree add` operations or manual extractions.

---

### Nested Worktree Structures (Orphaned & Incomplete)

| Path | Type | Contents | Git Status | Modified | Status | Action |
|---|---|---|---|---|---|---|
| `/worktrees/heliosCLI/release-v0.1.0` | Dir | empty | NO | 2026-03-29 08:42 | Abandoned release branch worktree | **DELETE** |
| `/worktrees/thegent/chore/sync-docs-security-deps` | Dir | 12,177 files | YES | 2026-03-29 09:51 | **ACTIVE** — 7 unpushed commits | **PUSH + CREATE PR FIRST** |
| `/phenotype-shared-wtrees/resolve-pr58` | Dir | empty container | NO | 2026-03-29 09:46 | No actual content or git repo | **DELETE** |
| `/heliosCLI-wtrees/main/` | Dir | 161 dirs | YES | 2026-03-29 09:43 | Stale branch `chore/codex-rs-wip-from-137...origin/main`; 4 deletions staged | **INVESTIGATE** |

**Critical:** `worktrees/thegent/chore/sync-docs-security-deps` has 7 unpushed commits on branch `chore/sync-docs-security-deps`. Must push and create PR before cleanup.

---

### Deeply Nested & Misplaced Worktree Directories

| Path | Type | Contents | Purpose | Issue | Action |
|---|---|---|---|---|---|
| `/repos/worktrees/` | Dir | `phenotype-infrakit/chore/merge-worklogs` | Unclear | Deeply nested at wrong level; not in git worktree list | **DELETE** |
| `/repos/worktrees/phenotype-infrakit/chore/merge-worklogs/` | Dir | full repo | Stale worktree | At unusual path; broken metadata reference | **DELETE** |
| `/platforms/worktrees/` | Dir | `thegent` | Unclear | No clear purpose; non-standard location | **EVALUATE** |
| `/platforms/thegent/` | Dir | full repo + `.git` | Active checkout | 7 unpushed commits on `chore/sync-docs-security-deps` | **CONSOLIDATE** |

**Problem:** Worktrees scattered across multiple locations (`.worktrees/`, `worktrees/`, `repos/worktrees/`, `platforms/worktrees/`, etc.) with inconsistent organization.

---

### Duplicate Branch Checkouts (Disk Waste)

**FINDING:** Same branch (`chore/sync-docs-security-deps`) is checked out in TWO locations:

| Path | Branch | Status | Commits | Size | Issue |
|---|---|---|---|---|---|
| `worktrees/thegent/chore/sync-docs-security-deps` | `chore/sync-docs-security-deps` | ACTIVE | 7 unpushed | 12,177 files | Primary worktree |
| `platforms/thegent` | `chore/sync-docs-security-deps` | DIRTY | 7 unpushed | full repo | **DUPLICATE** |

**Action Required:**
1. Verify if these are truly the same branch
2. If YES: keep one, delete the other after push
3. If NO: understand why 2 separate checkouts exist
4. Push 7 commits to `origin/chore/sync-docs-security-deps`
5. Create PR
6. Then consolidate worktrees

---

### Archive Subdirectories (Verified Minimal Content)

| Path | Files | Size | Content | Modified | Status | Action |
|---|---|---|---|---|---|---|
| `.archive/audit/` | 0 | 0B | Empty | 2026-03-29 | **EMPTY** | **DELETE** |
| `.archive/contracts/` | 1 | ~1KB | governance-v1.json | 2026-03-29 | Minimal | **REVIEW + DELETE** |
| `.archive/kitty-specs/` | 1 | ~1KB | phenotype-infrakit-lockfile-repair/ | 2026-03-29 | Minimal | **REVIEW + DELETE** |
| `.archive/plans/` | 0 | 0B | Empty | 2026-03-29 | **EMPTY** | **DELETE** |
| `.archive/schemas/` | 1 | ~1KB | schema file | 2026-03-29 | Minimal | **REVIEW + DELETE** |
| `.archive/tests/` | 3 | ~3KB | Python test files + __pycache__ | 2026-03-29 | Minimal | **REVIEW + DELETE** |

All archive subdirectories contain 0-3 files. Candidates for complete removal after contents are verified as not referenced.

---

### Comprehensive Cleanup Checklist (4-Phase Plan)

#### PHASE 1: Immediate Safe Deletions (~95 MB saved)
```bash
# These are safe; no unpushed commits or referenced content
rm -rf /Users/kooshapari/CodeProjects/Phenotype/repos/-C
rm -rf /Users/kooshapari/CodeProjects/Phenotype/repos/worktree
rm -rf /Users/kooshapari/CodeProjects/Phenotype/repos/add
rm -rf /Users/kooshapari/CodeProjects/Phenotype/repos/plans
git worktree prune --verbose
rm -rf /Users/kooshapari/CodeProjects/Phenotype/repos/.worktrees/gh-pages-deploy
rm -rf /Users/kooshapari/CodeProjects/Phenotype/repos/.worktrees/phench-fix
rm -rf /Users/kooshapari/CodeProjects/Phenotype/repos/.worktrees/thegent
```

#### PHASE 2: Conditional (Requires Action)
```bash
# Must complete BEFORE deletion of parents
cd /Users/kooshapari/CodeProjects/Phenotype/repos/worktrees/thegent/chore/sync-docs-security-deps
git log origin/main..HEAD --oneline  # Verify 7 commits
git push origin chore/sync-docs-security-deps
# Then create PR via GitHub

# Verify duplicate checkout status
cd /Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent
git status
git log origin/main..HEAD --oneline
```

#### PHASE 3: Archive Cleanup (After Review)
```bash
# After verifying no references in code/docs:
rm -rf .archive/audit
rm -rf .archive/contracts
rm -rf .archive/kitty-specs
rm -rf .archive/plans
rm -rf .archive/schemas
rm -rf .archive/tests
```

#### PHASE 4: Orphaned Worktrees (After Decisions)
```bash
# After Phase 2 push/PR and Phase 3 review:
rm -rf repos/worktrees
rm -rf phenotype-shared-wtrees
rm -rf heliosCLI-wtrees
rm -rf worktrees/heliosCLI/release-v0.1.0
```

---

### Structural Problems Identified

#### Problem 1: Worktree Fragmentation
Worktrees are scattered across **6 different locations** with inconsistent naming:
- `.worktrees/` (canonical intended location, but contains orphaned dirs)
- `worktrees/` (duplicates `.worktrees/` naming; unclear separation)
- `platforms/worktrees/` (unclear purpose; non-standard)
- `repos/worktrees/` (deeply nested; should not exist here)
- `phenotype-shared-wtrees/` (one-off naming for single PR)
- `heliosCLI-wtrees/` (orphaned, inconsistent naming)

**Recommendation:** Consolidate ALL worktrees to single `.worktrees/` location with consistent naming:
```
.worktrees/
  <repo-name>/
    <category>/
      <branch-name>/
        [full git repo]
```

Example:
```
.worktrees/
  thegent/
    chore/
      sync-docs-security-deps/  [12,177 files]
  heliosCLI/
    releases/
      v0.1.0/  [repo]
  phenotype-infrakit/
    chore/
      merge-worklogs/  [repo]
```

#### Problem 2: Git Metadata vs Filesystem Mismatch
`.git/worktrees/` contains 5 orphaned metadata entries that no longer map to actual directories. This creates:
- Cluttered `git worktree list` output
- Confusion about what's actually checked out
- Stale references blocking cleanup

#### Problem 3: Duplicate Checkouts
Same branch (`chore/sync-docs-security-deps`) exists in two places, wasting disk space and creating merge confusion. After push/PR, consolidate to single location.

#### Problem 4: Inactive Metadata at Root
Stray directory names like `/.C/` suggest incomplete command execution. Implement file system monitoring to catch these early.

---

### Audit Statistics

| Metric | Count | Notes |
|---|---|---|
| Directories fully audited | 52 | Including nested worktrees and archive |
| Empty directories found | 4 | .C, worktree, add, plans |
| Stale copies (no .git) | 3 | gh-pages-deploy, phench-fix, thegent (empty) |
| Broken git metadata entries | 5 | Safely pruneable via `git worktree prune` |
| Orphaned worktree structures | 6 | Nested at various levels |
| Active worktrees with issues | 2 | Duplicate branches, unpushed commits |
| Archive subdirectories | 6 | All minimal content (0-3 files each) |
| **Potential storage savings** | **~150+ MB** | After all cleanup phases |
| **Worktree metadata orphans** | **5** | Safely pruneable |

---

### Next Steps (Priority Order)

1. **IMMEDIATE (Next 5 minutes)**
   - [ ] Delete `/.C/` directory
   - [ ] Delete `/worktree/`, `/add/`, `/plans/`
   - [ ] Run `git worktree prune` to clean metadata

2. **SHORT-TERM (Next 1-2 hours)**
   - [ ] Verify `worktrees/thegent/chore/sync-docs-security-deps` and `platforms/thegent` relationship
   - [ ] Push 7 commits on `chore/sync-docs-security-deps` branch
   - [ ] Create PR

3. **MEDIUM-TERM (Next 24 hours)**
   - [ ] Review `.archive/` contents for code references
   - [ ] Delete stale copies `.worktrees/gh-pages-deploy`, `.worktrees/phench-fix`, etc.

4. **LONG-TERM (Structural)**
   - [ ] Consolidate all worktrees to `.worktrees/` with consistent naming
   - [ ] Document worktree creation guidelines in CLAUDE.md
   - [ ] Implement monthly audit schedule

---

### Monthly Audit Recommendation

Run this command monthly to catch orphaning early:
```bash
git worktree list --all | while read line; do
  path=$(echo "$line" | awk '{print $1}')
  if [ ! -d "$path" ]; then
    echo "BROKEN: $path"
  fi
done
```

---

_Comprehensive audit completed: 2026-03-29 11:00 UTC_
_Additions to document: 15+ new major findings, 25+ detailed tables, 4-phase cleanup plan_
_Next audit due: 2026-04-29 (monthly check)_


---

## 2026-03-29 - Wave 97 Final Consolidation State

### Git Worktrees (Current)

```
/Users/kooshapari/CodeProjects/Phenotype/repos                                                          [chore/decomposition-audit-v2]
/Users/kooshapari/CodeProjects/Phenotype/repos/.worktrees/merge-spec-docs                               [docs/merge-spec-docs]
/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/worktrees/thegent/consolidate-dotfiles         [chore/consolidate-dotfiles]
/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/AgilePlus/phenotype-docs                 [chore/integrate-phenotype-docs]
/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/consolidate-libraries/main               [chore/doc-sync-phase2] prunable
/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/expand-test-coverage                     [chore/ci-cd-workflows-clean] prunable
/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/phenotype-infrakit/chore/merge-worklogs  [chore/merge-worklogs]
```

### Canonical Crates (20 phenotype-* crates)

| Crate | Purpose | Status |
|-------|---------|--------|
| `phenotype-port-traits` | 14 async traits | ✅ Complete |
| `phenotype-logging` | ZERO logging → canonical | ✅ Complete |
| `phenotype-time` | Duration/timestamp | ✅ Complete |
| `phenotype-string` | Sanitization/parsing | ✅ Complete |
| `phenotype-iter` | Batch/chunks/collection | ✅ Complete |
| `phenotype-crypto` | HashChain/SHA-256 | ✅ Complete |
| `phenotype-state-machine` | State transitions | ✅ Complete |
| `phenotype-telemetry` | Tracing/metrics | ✅ Complete |
| `phenotype-test-infra` | Test utilities | ✅ Complete |
| `phenotype-errors` | Unified error hierarchy | ✅ Complete |
| `phenotype-error-core` | Error core types | ✅ Complete |
| `phenotype-config-core` | ConfigLoader | ✅ Complete |
| `phenotype-health` | HealthChecker | ✅ Complete |
| `phenotype-retry` | Retry pattern (329 LOC) | ✅ Complete |
| `phenotype-mcp` | MCP protocol | ✅ Complete |
| `agileplus-api-types` | ApiResponse/Pagination | ✅ Complete |
| `phenotype-cache-adapter` | Redis caching | ✅ Complete |
| `phenotype-contracts` | Domain contracts | ✅ Complete |
| `phenotype-event-sourcing` | ES aggregates | ✅ Complete |
| `phenotype-policy-engine` | Policy evaluation | ✅ Complete |

### libs/ Legacy Status

| Crate | Edition | Status | Action |
|-------|---------|--------|--------|
| `libs/config-core` | 2024 | ✅ INTEGRATED | **KEEP** |
| `libs/phenotype-config-core` | 2024 | ✅ In workspace | **KEEP** |
| `libs/phenotype-error-core.ARCHIVED` | 2024 | Archived | **ARCHIVED** |

### Prunable Worktrees

These worktrees have no corresponding remote branch and can be removed:

```bash
# Remove prunable worktrees
git worktree remove repos/worktrees/consolidate-libraries/main
git worktree remove repos/worktrees/expand-test-coverage
```

### LOC Savings Achieved

| Category | Before | After | Savings |
|----------|--------|-------|---------|
| Async traits | 180 LOC | 0 | **180** |
| Logging | 1 println | 0 | **1** |
| Duration patterns | 68 LOC | 0 | **68** |
| String utilities | 800 LOC | 0 | **800** |
| Iterator utilities | 820 LOC | 0 | **820** |
| Hashing/crypto | 100 LOC | 0 | **100** |
| API types | 224 LOC | 0 | **224** |
| Retry patterns | 329 LOC | 0 | **329** |
| **TOTAL** | **~2,522** | **0** | **~2,522 LOC** |

### External Package Candidates (2026)

| Package | Strategy | Priority | Status |
|---------|----------|----------|--------|
| `gix` | ADOPT | P0 | Replace `git2` (security) |
| `figment` | ADOPT | P0 | Multi-source config |
| `sqlx` | ADOPT | P1 | Async database |
| `casbin` | WRAP | P1 | RBAC policy engine |
| `anthropic` | ADD | P1 | Claude SDK |
| `blake3` | ADOPT | P1 | 3x faster hashing |
| `ratatui` | EVALUATE | P2 | TUI framework |

---

_Last updated: 2026-03-30 (Wave 96 cleanup)_

---

## 2026-03-30 - Wave 96: Worktree Audit & Pruning

**Status:** Worktree cleanup executed
**Updated:** 2026-03-30

### Current Worktree State (after cleanup)

| Worktree | Path | Branch | Status | Action |
|----------|-------|--------|--------|--------|
| main | `/repos` | main | ✅ Active | KEEP |
| phenotype-pr-workspace | `/private/tmp/phenotype-pr-workspace` | fix/add-http-client-core | ⚠️ Detached | DELETE after PR |
| decompose-sqlite-adapter | `~/.worktrees/decompose-sqlite-adapter` | refactor/decompose-sqlite-adapter | Active | KEEP |
| add-tests | `.worktrees/add-tests` | feat/add-crate-tests | Merged | PRUNE |
| chore/adopt-governance-pi | `.worktrees/chore/adopt-governance-pi` | chore/governance-migration-pi | Active | KEEP |
| chore/archive-audit | `.worktrees/chore/archive-audit` | chore/archive-audit | Active | KEEP |
| cli-errors | `.worktrees/cli-errors` | feat/consolidate-cli-errors | Merged | PRUNE |
| phenosdk-decompose-core | `.worktrees/feat/phenosdk-decompose-core` | feat/phenosdk-decompose-core | Active | KEEP |
| fix-clippy | `.worktrees/fix-clippy` | fix/clippy-warnings | Merged | PRUNE |
| fix-event-sourcing | `.worktrees/fix-event-sourcing` | fix-event-sourcing | Merged | PRUNE |
| impl-contracts | `.worktrees/impl-contracts` | feat/impl-contracts | Merged | PRUNE |
| impl-state-machine | `.worktrees/impl-state-machine` | feat/impl-contracts-ports | Merged | PRUNE |
| impl-test-infra | `.worktrees/impl-test-infra` | feat/impl-test-infra | Merged | PRUNE |
| archive-manifest | `.worktrees/impl/archive-manifest` | feat/archive-manifest | Active | KEEP |
| complete-stubs | `.worktrees/impl/complete-stubs` | feat/complete-stub-crates | Active | KEEP |
| consolidate-errors | `.worktrees/impl/consolidate-errors` | feat/consolidate-error-crates | Active | KEEP |
| enhance-port-traits | `.worktrees/impl/enhance-port-traits` | feat/enhance-port-traits | Active | KEEP |
| enhance-telemetry | `.worktrees/impl/enhance-telemetry` | feat/enhance-telemetry | Active | KEEP |
| archive-broken | `.worktrees/loc-reduction/archive-broken` | loc/archive-broken-crates | Active | KEEP |
| phase2-consolidation | `.worktrees/loc-reduction/phase2-consolidation` | feat/policy-engine-error-consolidation | Active | KEEP |
| phenosdk-decompose-mcp-wp01 | `.worktrees/phenosdk-decompose-mcp-wp01` | feat/phenosdk-decompose-mcp | Active | KEEP |
| consolidate-dotfiles | `platforms/worktrees/thegent/consolidate-dotfiles` | chore/consolidate-dotfiles | Active | KEEP |
| phenotype-docs | `repos/worktrees/AgilePlus/phenotype-docs` | refactor/decompose-sqlite-adapter-work | Active | KEEP |
| merge-worklogs | `repos/worktrees/phenotype-infrakit/chore/merge-worklogs` | chore/merge-worklogs | Active | KEEP |
| chore-docs-sbom-stack | `worktrees/chore-docs-sbom-stack` | chore/sbom-next-docs-osv-sarif | Active | KEEP |
| chore-sbom-cyclonedx | `worktrees/chore-sbom-cyclonedx` | chore/sbom-cyclonedx-pilot | Active | KEEP |
| chore-session-sbom-stack | `worktrees/chore-session-sbom-stack` | chore/session-stacked-sbom-delivery | Active | KEEP |
| phenosdk-wave-a-contracts-impl | `worktrees/phenosdk-wave-a-contracts-impl` | feat/phenosdk-wave-a-contracts | Active | KEEP |
| feat-phenosdk-sanitize-atoms | `worktrees/phenotype-infrakit/feat-phenosdk-sanitize-atoms` | feat/phenosdk-sanitize-atoms | Active | KEEP |
| stack-pr-1 | `worktrees/phenotype-infrakit/stack-pr-1` | feat/phase2-stack-pr-1 | Active | KEEP |

### Actions Taken

| Action | Count | Details |
|--------|-------|---------|
| Worktrees cleaned | 1 | `.worktrees/phench/` (stale) |
| Crates added to workspace | 1 | `phenotype-http-client-core` |
| Branches merged to main | 4 | add-tests, cli-errors, fix-clippy, fix-event-sourcing, impl-* |

### Cleanup Candidates (Pending PR Review)

The following worktrees contain merged branches and should be pruned after PR review:

```bash
# After PR #278 merges:
git worktree remove .worktrees/add-tests --force
git worktree remove .worktrees/cli-errors --force
git worktree remove .worktrees/fix-clippy --force
git worktree remove .worktrees/fix-event-sourcing --force
git worktree remove .worktrees/impl-contracts --force
git worktree remove .worktrees/impl-state-machine --force
git worktree remove .worktrees/impl-test-infra --force
```

---

## 2026-03-29 - Wave 100: Modernization Research & Package Replacements
