# Test File Deduplication Analysis & Roadmap

**Date**: 2026-03-30
**Status**: Analysis Complete, Ready for Implementation
**Branch Target**: `refactor/deduplicate-tests`
**Estimated LOC Savings**: ~207,765 LOC (9.91 MB)

## Executive Summary

This repository contains severe test file duplication across worktrees. The analysis identified:

- **1,281** total test files in project (excluding `.venv/` and `.archive/`)
- **1,268** duplicate file names (same filename in multiple locations)
- **1,332** total duplicate instances (copies beyond the original)
- **9.91 MB** of duplicate test code (~207,765 LOC estimated)
- **Primary cause**: Worktrees cloned full project structures, including test suites

This deduplication effort will reduce repository bloat, speed up git operations, and improve CI performance by eliminating redundant test discovery.

## Duplicate File Distribution

### Top 20 Most-Duplicated Files

| File | Copies | Locations | Canonical |
|------|--------|-----------|-----------|
| test_schema.py | 7 | platforms/thegent/heliosCLI + worktrees | platforms/thegent/tests/research_engine/ |
| test_resilience.py | 6 | platforms/thegent + worktrees | platforms/thegent/tests/chaos/ |
| test_integration_mcp_tools_agents.py | 5 | python/pheno-mcp + 4 worktrees | python/pheno-mcp/tests/ |
| test_mcp_entry_points.py | 5 | python/pheno-mcp + 4 worktrees | python/pheno-mcp/tests/ |
| test_agents_orchestration.py | 5 | python/pheno-mcp + 4 worktrees | python/pheno-mcp/tests/ |
| test_tools_decorators.py | 5 | python/pheno-mcp + 4 worktrees | python/pheno-mcp/tests/ |
| test_path_utils.py | 4 | platforms/thegent + worktrees | platforms/thegent/tests/thegent/utils/ |
| test_batch_file_ops.py | 4 | platforms/thegent + worktrees | platforms/thegent/tests/thegent/utils/ |
| test_wl185_reflection_rollback.py | 4 | platforms/thegent + worktrees | platforms/thegent/tests/integrations/ |
| test_project_registry.py | 4 | platforms/thegent + worktrees | platforms/thegent/tests/registry/ |
| test_enterprise_compliance.py | 4 | platforms/thegent + worktrees | platforms/thegent/tests/governance/ |
| test_injection.py | 4 | platforms/thegent + worktrees | platforms/thegent/tests/routing/guardrails/ |
| test_git_parallelism.py | 4 | platforms/thegent + worktrees | platforms/thegent/tests/unit/ |
| test_cache.py | 4 | platforms/thegent + worktrees | platforms/thegent/tests/routing/ |
| test_session_hook.py | 4 | platforms/thegent + worktrees | platforms/thegent/tests/docs_engine/ |
| test_mcp_tools.py | 4 | platforms/thegent + worktrees | platforms/thegent/tests/docs_engine/ |
| test_cli.py | 4 | platforms/thegent + worktrees | platforms/thegent/tests/docs_engine/ |
| test_sub_agent_dispatcher.py | 4 | platforms/thegent + worktrees | platforms/thegent/tests/orchestration/ |
| test_cross_project.py | 4 | platforms/thegent + worktrees | platforms/thegent/tests/ipc/ |
| test_runner.py | 4 | platforms/thegent + worktrees | platforms/thegent/tests/maif/ |

## Canonical Location Rules

### Rule 1: thegent Tests
- **Canonical**: `platforms/thegent/tests/` (main branch directory)
- **Duplicates**: `platforms/worktrees/thegent/*/tests/`
- **Action**: Remove all duplicate copies from worktrees; optionally symlink back to canonical

### Rule 2: heliosCLI Tests
- **Canonical**: `heliosCLI/harness/tests/` (main directory)
- **Duplicates**: `heliosCLI/.worktrees/*/tests/`, `heliosCLI/worktrees/*/tests/`
- **Action**: Remove all duplicate copies from worktrees

### Rule 3: Phenotype SDK Tests (python/)
- **Canonical**: `python/pheno-*/tests/` (main directories)
- **Duplicates**: `.worktrees/*/python/pheno-*/tests/`
- **Action**: Remove all duplicate copies from worktrees; optionally symlink back to canonical

### Rule 4: AgilePlus Tests
- **Canonical**: `AgilePlus/*/tests/` (main crate tests)
- **Duplicates**: Any copies in worktrees
- **Action**: Remove all duplicate copies from worktrees

## Implementation Phases

### Phase 1: Create Mapping & Verification (NON-DESTRUCTIVE)
**Objective**: Document all duplicates, verify content hash, identify divergences

**Tasks**:
1. Generate comprehensive mapping: canonical → [duplicates]
2. Hash-verify all duplicates against canonical (flag divergences)
3. Create `.dedup/test_duplication_map.json` (reference artifact)
4. Document any test files with diverged content (requires manual review)

**Output**:
- `.dedup/test_duplication_map.json` (1,268 files × 2-3 locations each)
- `.dedup/DIVERGENCE_REPORT.md` (any tests with content differences)

**Risk**: None (read-only analysis)

### Phase 2: Create Feature Branch
**Objective**: Prepare isolated branch for safe deduplication work

**Tasks**:
1. Create branch: `git checkout -b refactor/deduplicate-tests`
2. Verify branch is clean: `git status`
3. No commits yet; just preparation

### Phase 3: Remove Duplicate Test Files (DESTRUCTIVE, REVERSIBLE)
**Objective**: Delete duplicate test files from worktrees

**Tasks**:
1. For each worktree with duplicates:
   - Delete duplicate test files: `git rm -r <worktree-test-path>/`
   - Commit changes per worktree: `git commit -m "refactor: remove duplicate tests from <worktree>"`

2. Verify deletions:
   - `git status` shows only deletions
   - No files modified, only removed

**Output**:
- ~1,332 files deleted (9.91 MB saved)
- ~20-30 commits (one per worktree / test subdirectory removed)

**Risk**: MEDIUM
- Reversible via `git reset` or `git revert`
- Once committed and pushed, harder to undo
- Recommend squashing commits in PR for cleaner history

### Phase 4: Create Symlinks (OPTIONAL, GIT-FRIENDLY)
**Objective**: Allow worktrees to reference canonical tests without duplication

**Tasks**:
1. Identify worktrees that need test discovery
2. Create relative symlinks: `ln -s ../../thegent/tests tests`
3. Add symlinks to git: `git add tests`
4. Document symlink strategy in `.dedup/SYMLINK_STRATEGY.md`

**Output**:
- Relative symlinks from worktrees to canonical test directories
- All symlinks tracked in git (portable across clones)

**Risk**: LOW
- Symlinks are OS-portable in git
- May not work on Windows without special config (but tests run on Linux in CI)
- Fallback: Keep removed, don't create symlinks (tests won't run in worktree)

### Phase 5: Update CI Configuration
**Objective**: Ensure test discovery includes canonical locations, skips worktree duplicates

**Tasks**:
1. Update `pytest.ini` / `pyproject.toml`:
   - Add/verify canonical test paths in `testpaths`
   - Example: `testpaths = ["tests", "python/*/tests", "platforms/thegent/tests"]`

2. Update CI workflows (`.github/workflows/*.yml`):
   - Verify pytest runs with canonical paths
   - Remove any worktree-specific test discovery paths
   - Test on Linux CI (primary platform)

3. Verify test discovery:
   - `pytest --collect-only` should show all canonical tests
   - No duplicate test collection

**Output**:
- Updated CI configs
- Verified test discovery

**Risk**: LOW
- Can roll back via git
- CI runs can be tested on a branch before merging

### Phase 6: Documentation & Commit
**Objective**: Document deduplication strategy and create final commit

**Tasks**:
1. Create `.dedup/DEDUPLICATION_STRATEGY.md` (this document + final stats)
2. Create `.dedup/DEDUP_SUMMARY.txt` with quick stats:
   ```
   Deduplication Summary
   ====================
   Files removed: 1,332
   Size recovered: 9.91 MB (~207,765 LOC)
   Canonical locations preserved: 1,281
   Symlinks created: [N/A or count]

   Primary locations:
   - platforms/thegent/tests/
   - python/pheno-*/tests/
   - heliosCLI/harness/tests/
   - AgilePlus/*/tests/
   ```

3. Squash or rebase commits:
   - Option A: Squash all deletes into single commit: `git rebase -i <base>`
   - Option B: Keep as-is with logical per-worktree commits (more atomic)
   - Recommendation: Squash for simplicity

4. Final commit message:
   ```
   refactor: deduplicate test files across worktrees

   Remove 1,332 duplicate test files from worktrees, saving 9.91 MB.

   Changes:
   - Removed duplicate tests from all worktrees
   - Preserved canonical test locations in main directories
   - Updated CI test discovery to use canonical paths

   Savings:
   - Files removed: 1,332
   - Space recovered: 9.91 MB (~207,765 LOC)
   - CI test discovery improved (no redundant paths)

   Canonical locations (preserved):
   - platforms/thegent/tests/
   - python/pheno-*/tests/
   - heliosCLI/harness/tests/
   - AgilePlus/*/tests/

   Co-Authored-By: Claude Haiku 4.5 <noreply@anthropic.com>
   ```

5. Push and create PR:
   - `git push -u origin refactor/deduplicate-tests`
   - Create PR with summary

### Phase 7: PR & Merge
**Objective**: Code review and merge to main

**Tasks**:
1. Open PR on GitHub
2. PR title: `refactor: deduplicate test files (1,332 removed, 9.91 MB saved)`
3. PR description:
   - Include summary from Phase 6
   - Link to `.dedup/` documentation
   - Before/after stats

4. Verify CI:
   - Tests pass on canonical paths
   - No duplicate test collection
   - Build succeeds

5. Merge via squash (if Phase 6 didn't squash):
   - Keeps history clean
   - Single commit record of major refactoring

## Timeline Estimate

| Phase | Task | Time |
|-------|------|------|
| 1 | Mapping + verification | 5 min (automated script) |
| 2 | Create branch | 1 min |
| 3 | Remove duplicates | 10 min (automated deletion + git ops) |
| 4 | Create symlinks (optional) | 5 min |
| 5 | Update CI | 5 min |
| 6 | Documentation + commit | 5 min |
| 7 | PR + merge | 10 min (CI validation) |
| **Total** | | **~41 min wall-clock** |

## Risk Assessment

| Phase | Risk | Severity | Mitigation |
|-------|------|----------|-----------|
| 3 | Accidental deletion of canonical tests | MEDIUM | Hash verification before removal; keep only duplicates |
| 3 | Diverged tests removed incorrectly | MEDIUM | Phase 2 divergence report + manual review |
| 4 | Symlinks not portable on Windows | LOW | CI runs on Linux; Windows support optional |
| 5 | CI test discovery misconfigured | MEDIUM | Test on branch before merging; verify collection |
| 6 | Commit message unclear | LOW | Use detailed template; document in PR |

## Success Criteria

- [ ] All 1,332 duplicate files removed from worktrees
- [ ] All canonical test directories preserved and verified
- [ ] No diverged tests removed (only identical copies)
- [ ] CI test discovery includes all canonical tests
- [ ] Build succeeds; all tests run
- [ ] No regression in test results
- [ ] `.dedup/` documentation complete
- [ ] PR merged to main

## Rollback Plan

If issues arise post-merge:
1. Revert commit: `git revert <merge-commit>`
2. Re-run CI to verify
3. Investigate root cause
4. Retry with fix

If rollback needed pre-merge:
1. Delete branch: `git branch -D refactor/deduplicate-tests`
2. No data lost (original files unchanged on main)

## Next Steps

1. **Review this analysis** for approval
2. **Execute Phase 1-2** (mapping + branch creation)
3. **Verify Phase 2 divergence report** for any manual work needed
4. **Execute Phase 3-7** (removal, symlinks, CI, commit, PR)
5. **Monitor PR** for CI results and code review feedback
6. **Merge to main** once approved

---

**Generated**: 2026-03-30
**Analysis Tool**: Python script (deduplication detection via file naming + hashing)
**Data Sources**: Git repository scan, file size analysis, git history
