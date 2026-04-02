# Crate Registry Consolidation Plan

**Date**: 2026-04-01
**Status**: AUDITED - Ready for PR Execution

---

## Executive Summary

Audit reveals **51 prefixed crates** across three product lines with fragmentation issues. Proposed consolidation into monoregistries with clear boundaries.

---

## Current State Audit

### Git Repositories in Scope

| Repo | Remote | Type | Status |
|------|--------|------|--------|
| `repos/` | `phenotype-infrakit` | Main workspace | Dirty (39 stashes) |
| `repos/phenotype-infrakit/` | `phenotype-infrakit` | Submodule | Orphaned stubs |
| `repos/platforms/thegent/` | N/A (local) | Agent platform | Dirty |
| `repos/AgilePlus/` | `AgilePlus` | Product | Separate git repo |

### Crate Distribution

| Prefix | Count | Location | Implementation |
|--------|-------|----------|----------------|
| `phenotype-*` | 33 | `repos/crates/` | 100% implemented |
| `phenotype-*` | 8 | `repos/phenotype-infrakit/crates/` | 1 implemented, 7 stubs |
| `agileplus-*` | 22 | `repos/crates/` | Mixed |
| `thegent-*` | 30 | `repos/platforms/thegent/crates/` | Most implemented |

### Duplicate Crates

| Crate | Root Location | phenotype-infrakit | Resolution |
|-------|---------------|-------------------|------------|
| `phenotype-error-core` | Implemented | STUB | Archive in infrakit |
| `phenotype-casbin-wrapper` | Implemented | STUB | Archive in infrakit |
| `phenotype-config-core` | Implemented | STUB | Archive in infrakit |
| `phenotype-config-loader` | Implemented | STUB | Archive in infrakit |
| `phenotype-cost-core` | Implemented | STUB | Archive in infrakit |
| `phenotype-git-core` | Implemented | STUB | Archive in infrakit |

### Worktrees

```
repos/                          [fix/http-client-core-simplify]
├── .worktrees/feat/cache-adapter-impl     [detached HEAD]
└── .worktrees/feat/phenotype-crypto-complete  [feat/phenotype-crypto-complete-v2]

platforms/thegent/               [feat/plugin-host-adapter]
├── .worktrees/thegent/pr-876-fix   [pr-876-fix]
└── platforms/thegent-pr882      [detached HEAD] ← PRUNABLE
```

### Stash Inventory

**repos/**: 39 stashes (phenotype-infrakit related work)
**platforms/thegent/**: 2 stashes

---

## Recommended Structure

```
repos/ (phenotype-infrakit monoregistry)
├── Cargo.toml              ← Unified workspace root
├── crates/phenotype-*/     ← All phenotype shared crates
└── phenotype-infrakit/     ← DEPRECATED - archive after merge

platforms/ (future parent)
└── thegent/               ← SEPARATE workspace (agent platform)

AgilePlus/                 ← SEPARATE git repo
```

---

## PR Roadmap

### PR 1: Archive phenotype-infrakit duplicates
**Branch**: `chore/archive-infrakit-duplicates`
**Action**: Archive `phenotype-infrakit/crates/` duplicates to `.archive/`
**Affected**: 7 stub crates

```bash
mkdir -p phenotype-infrakit/crates/.archive
git -C phenotype-infrakit mv crates/phenotype-error-core crates/.archive/
git -C phenotype-infrakit mv crates/phenotype-casbin-wrapper crates/.archive/
git -C phenotype-infrakit mv crates/phenotype-config-core crates/.archive/
git -C phenotype-infrakit mv crates/phenotype-config-loader crates/.archive/
git -C phenotype-infrakit mv crates/phenotype-cost-core crates/.archive/
git -C phenotype-infrakit mv crates/phenotype-git-core crates/.archive/
git -C phenotype-infrakit mv crates/phenotype-capital crates/.archive/
```

### PR 2: Add missing phenotype-* crates to workspace
**Branch**: `feat/add-missing-phenotype-crates`
**Action**: Add newly discovered phenotype crates to root workspace

### PR 3: Prune orphaned stashes
**Branch**: `chore/prune-stashes`
**Action**: Review and prune 39 stashes (move to worktrees if needed)

### PR 4: Cleanup prunable worktrees
**Branch**: `chore/cleanup-worktrees`
**Action**: Remove `platforms/thegent-pr882` (detached, prunable)

---

## Execution Commands

```bash
# 1. Archive phenotype-infrakit stubs
cd /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit
mkdir -p crates/.archive
for crate in phenotype-error-core phenotype-casbin-wrapper phenotype-config-core phenotype-config-loader phenotype-cost-core phenotype-git-core phenotype-capital; do
    [ -d "crates/$crate" ] && git mv crates/$crate crates/.archive/
done
git add -A && git commit -m "chore(infrakit): archive duplicate stubs to .archive/"

# 2. Create PR for phenotype-infrakit
gh pr create --base main --title "chore(infrakit): archive duplicate crates" --body "Archive 7 stub crates that duplicate root workspace implementations"

# 3. Switch to repos root for workspace cleanup
cd /Users/kooshapari/CodeProjects/Phenotype/repos

# 4. Prune prunable worktree
git -C platforms/thegent worktree remove platforms/thegent-pr882 --force 2>/dev/null || echo "Already cleaned"
```

---

## Verification Checklist

- [ ] phenotype-infrakit has no duplicate stubs
- [ ] All phenotype-* crates in repos/crates/ are in workspace
- [ ] No prunable worktrees remain
- [ ] Stashes reviewed and pruned or promoted
- [ ] All PRs created and linked

---

## Future Considerations

1. **Generic parent `platforms/` workspace** - Can be added when cross-platform dependencies emerge
2. **thegent-* crates** - Remain separate; different product line
3. **agileplus-* crates** - Remain in AgilePlus repo; separate release cadence
