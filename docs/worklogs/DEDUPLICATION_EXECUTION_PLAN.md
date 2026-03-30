# Test File Deduplication: Execution Plan

**Date**: 2026-03-30
**Status**: Ready for Implementation
**Branch**: `refactor/deduplicate-tests`
**Owner**: Claude Code Agent
**Priority**: High (removes 9.81 MB of duplicate code)

## Quick Summary

This document outlines the step-by-step execution plan for deduplicating 1,316 test files across the Phenotype workspace.

**Scope**:
- 1,268 duplicate test file names identified
- 1,316 duplicate instances (copies beyond canonical)
- 9.81 MB of redundant code
- ~207,765 LOC of duplicate tests
- 18 files with diverged content (requires manual review)

**Timeline**: ~40-50 minutes wall-clock time
**Reversibility**: Full (all changes in git; can revert if needed)

## Reference Documents

All analysis artifacts stored in `.dedup/`:
1. **TEST_DEDUPLICATION_ANALYSIS.md** - Full analysis + strategy
2. **test_duplication_map.json** - Complete mapping of canonical → duplicates
3. **DIVERGENCE_REPORT.md** - 18 files with content differences (keep both copies)
4. **DEDUP_SUMMARY.txt** - Final statistics (created at end)

## Execution Steps

### Step 1: Verify Current State (5 min)

Check that repository is clean and ready for branch:

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos

# Verify on main branch
git status --short --branch
# Should show: "## main ... origin/main [even]"

# Ensure no uncommitted changes
git diff --quiet && echo "✓ Clean" || echo "✗ Uncommitted changes"

# Pull latest from origin
git pull origin main --ff-only
```

**Expected Output**:
```
## main ... origin/main [even]
On branch main.
nothing to commit, working tree clean

✓ Clean
```

### Step 2: Create Feature Branch (2 min)

```bash
# Create and switch to feature branch
git checkout -b refactor/deduplicate-tests

# Verify branch created
git status --short --branch
# Should show: "## refactor/deduplicate-tests ... no tracking info"
```

**Expected Output**:
```
Switched to a new branch 'refactor/deduplicate-tests'
## refactor/deduplicate-tests ... no tracking info
```

### Step 3: Verify Mapping & Divergences (1 min)

Review the analysis artifacts created in `.dedup/`:

```bash
# Check mapping file exists and is valid
ls -lh /Users/kooshapari/CodeProjects/Phenotype/repos/.dedup/test_duplication_map.json

# Review divergence report
cat /Users/kooshapari/CodeProjects/Phenotype/repos/.dedup/DIVERGENCE_REPORT.md | head -50
```

**Expected Output**:
- Files exist with proper content
- 18 diverged files documented
- ~1,300 identical duplicates ready for removal

### Step 4: Generate Removal Script (5 min)

Create an automated script to safely remove identical duplicates (skipping the 18 diverged copies):

```python
# Script will be auto-generated to:
# 1. Read test_duplication_map.json
# 2. For each file with 0 diverged copies: remove all duplicates
# 3. For files with diverged copies: skip (keep all)
# 4. Generate list of removed files + size savings

# Expected to remove ~1,298 files (1,316 - 18 diverged)
# Expected to save ~9.75 MB
```

### Step 5: Execute Removal (15 min)

Run the removal script which will:

1. **Identify all duplicate files** (non-diverged only)
2. **Remove via git**: `git rm -r <path>`
3. **Batch commits** by directory/worktree (~20-30 commits)
4. **Verify deletions**: `git status` shows only deletions

```bash
# Example commands (automated via script):

# Remove duplicates from .worktrees/*/python/pheno-*/tests/
git rm -r .worktrees/cli-errors/python/pheno-mcp/tests/test_*.py
git commit -m "refactor: remove duplicate tests from cli-errors worktree"

# Remove from platforms/worktrees/thegent/*/tests/
git rm -r platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/

# [Continue for all worktrees...]
```

**After Each Batch**:
```bash
git status
# Should show only deletions (no modifications or new files)

git log --oneline -5
# Should show series of "refactor: remove duplicate tests from <worktree>" commits
```

### Step 6: Verify No Canonical Tests Removed (5 min)

Ensure all canonical test locations are intact:

```bash
# Verify canonical directories still exist
test -d platforms/thegent/tests && echo "✓ thegent tests OK"
test -d python/pheno-mcp/tests && echo "✓ pheno-mcp tests OK"
test -d python/pheno-core/tests && echo "✓ pheno-core tests OK"
test -d heliosCLI/harness/tests && echo "✓ heliosCLI tests OK"

# Count remaining test files (should be 1,281 canonical + 18 diverged = 1,299 minimum)
find /Users/kooshapari/CodeProjects/Phenotype/repos -type f -name "test_*.py" \
  -not -path "*/.venv/*" -not -path "*/.archive/*" 2>/dev/null | wc -l
```

**Expected Output**:
```
✓ thegent tests OK
✓ pheno-mcp tests OK
✓ pheno-core tests OK
✓ heliosCLI tests OK
1299  # or similar (canonical + diverged copies)
```

### Step 7: Generate Deduplication Summary (3 min)

Create final summary document:

```bash
cat > /Users/kooshapari/CodeProjects/Phenotype/repos/.dedup/DEDUP_SUMMARY.txt << 'EOF'
Test File Deduplication Summary
==============================
Date: 2026-03-30
Branch: refactor/deduplicate-tests

Results:
--------
Files removed: 1,298 (identical duplicates)
Files preserved: 1,281 (canonical) + 18 (diverged) = 1,299
Space recovered: 9.75 MB
Estimated LOC removed: ~204,000

Canonical Locations (Preserved):
- platforms/thegent/tests/
- python/pheno-mcp/tests/
- python/pheno-core/tests/
- python/pheno-config/tests/
- heliosCLI/harness/tests/
- phench/tests/
- AgilePlus/*/tests/

Worktrees (Cleaned):
- .worktrees/*/python/*/tests/
- platforms/worktrees/thegent/*/tests/
- heliosCLI/worktrees/*/tests/
- heliosCLI/.worktrees/*/tests/

Diverged Tests (Kept - Manual Review Needed):
- 18 files with content differences between canonical and worktree copies
- See DIVERGENCE_REPORT.md for details

Next Steps:
-----------
1. Review diverged tests: DIVERGENCE_REPORT.md
2. Consider consolidating/merging diverged versions
3. Update CI test discovery (if needed)
4. Create PR for code review
5. Merge to main after approval
EOF

cat /Users/kooshapari/CodeProjects/Phenotype/repos/.dedup/DEDUP_SUMMARY.txt
```

### Step 8: Final Commit (3 min)

Squash or organize commits into a single logical commit:

```bash
# Option A: Squash all removals into single commit
git rebase -i HEAD~20  # Adjust count based on actual commit count
# In rebase editor: keep first commit as "pick", change others to "squash"
# Save and close

# Option B: Keep as-is (if commits are well-organized by worktree)
# [Skip this step]

# Verify final state
git log --oneline -3
git status
```

**Expected Output**:
```
<hash> refactor: deduplicate test files across worktrees
<hash> previous commit on main
...
On branch refactor/deduplicate-tests
nothing to commit, working tree clean
```

### Step 9: Create Commit with Final Details (5 min)

Ensure final commit has comprehensive message:

```bash
# If commits weren't squashed, create a consolidation commit
git commit --allow-empty -m "refactor: deduplicate test files across worktrees

Remove 1,298 duplicate test files from worktrees, saving 9.75 MB.

Changes:
- Removed duplicate tests from all worktrees
- Preserved 1,281 canonical test locations
- Kept 18 diverged test copies (content-different; review needed)

Savings:
- Files removed: 1,298
- Space recovered: 9.75 MB (~204,000 LOC)
- Improvement: Faster git operations, cleaner test discovery

Canonical Locations (Preserved):
- platforms/thegent/tests/
- python/pheno-*/tests/
- heliosCLI/harness/tests/
- phench/tests/
- AgilePlus/*/tests/

Worktree Cleanup Strategy:
- Removed all identical test duplicates
- Kept 18 diverged copies for manual consolidation
- Document: .dedup/DIVERGENCE_REPORT.md
- Reference: .dedup/test_duplication_map.json

See .dedup/ for analysis artifacts and mapping.

Co-Authored-By: Claude Haiku 4.5 <noreply@anthropic.com>"

# Verify commit created
git log --oneline -2
```

### Step 10: Push Branch (2 min)

```bash
# Push feature branch to origin
git push -u origin refactor/deduplicate-tests

# Verify push succeeded
git branch -vv
# Should show: "refactor/deduplicate-tests [origin/refactor/deduplicate-tests] ..."
```

### Step 11: Create Pull Request (5 min)

Create PR on GitHub:

```bash
# Open PR creation page (manual or via CLI)
gh pr create \
  --title "refactor: deduplicate test files (1,298 removed, 9.75 MB saved)" \
  --body "$(cat << 'EOF'
## Summary

Remove 1,298 duplicate test files across worktrees, saving 9.75 MB of redundant code.

### Changes
- Removed duplicate test files from all worktrees
- Preserved canonical test locations in main directories
- Kept 18 diverged copies for manual review (see DIVERGENCE_REPORT.md)

### Statistics
- Files removed: 1,298
- Space recovered: 9.75 MB (~204,000 LOC)
- Canonical test locations: 1,281 (preserved)
- Diverged copies: 18 (kept, require human decision)

### Canonical Locations (Verified Intact)
- platforms/thegent/tests/
- python/pheno-mcp/tests/
- python/pheno-core/tests/
- heliosCLI/harness/tests/
- phench/tests/
- AgilePlus/*/tests/

### Impact
- Faster git clone/fetch (9.75 MB smaller)
- Cleaner test discovery in CI
- Simplified test maintenance (single source of truth)
- No impact on test functionality (canonical tests unchanged)

### Review Notes
- See `.dedup/TEST_DEDUPLICATION_ANALYSIS.md` for full analysis
- See `.dedup/DIVERGENCE_REPORT.md` for files requiring manual consolidation
- See `.dedup/test_duplication_map.json` for complete mapping

### Testing
- Verify canonical test locations intact
- Verify test discovery runs canonical tests
- Verify CI passes with new test paths

🤖 Generated with Claude Code
EOF
)"
```

**Expected Output**:
```
✓ Created pull request #NNN to main
```

### Step 12: Monitor CI & Merge (10 min)

Monitor PR for CI results:

```bash
# Check PR status (via CLI or GitHub)
gh pr checks <pr-number>

# Wait for CI to pass
# Address any test discovery issues if they arise
# Get approval from maintainer
# Merge via GitHub UI or CLI:
gh pr merge <pr-number> --squash --auto
```

## Rollback Plan

If any issues occur, rollback is simple:

```bash
# Local branch (before push)
git reset --hard HEAD~N  # Undo last N commits

# Remote branch (before merge to main)
git push origin -f refactor/deduplicate-tests

# After merge to main
git revert <merge-commit>
git push origin main
```

## Risk Assessment & Mitigation

| Risk | Severity | Mitigation | Status |
|------|----------|-----------|--------|
| Accidental canonical deletion | MEDIUM | Phase 2 verification + hash checking | MITIGATED |
| Diverged tests removed | MEDIUM | DIVERGENCE_REPORT.md + 18 files kept | MITIGATED |
| CI test discovery broken | MEDIUM | Verify canonical paths in CI config | To verify |
| Symlink compatibility | LOW | Keep removed (symlinks optional) | LOW PRIORITY |

## Success Criteria

- [ ] All 1,298 identical duplicates removed
- [ ] All 1,281 canonical tests preserved
- [ ] All 18 diverged copies kept
- [ ] CI passes on canonical test paths
- [ ] No regression in test results
- [ ] PR merged to main
- [ ] `.dedup/` documentation complete

## Timeline Summary

| Step | Task | Time | Cumulative |
|------|------|------|-----------|
| 1 | Verify state | 5 min | 5 min |
| 2 | Create branch | 2 min | 7 min |
| 3 | Review analysis | 1 min | 8 min |
| 4 | Generate script | 5 min | 13 min |
| 5 | Execute removal | 15 min | 28 min |
| 6 | Verify canonical | 5 min | 33 min |
| 7 | Summary doc | 3 min | 36 min |
| 8 | Squash commits | 3 min | 39 min |
| 9 | Final commit | 5 min | 44 min |
| 10 | Push branch | 2 min | 46 min |
| 11 | Create PR | 5 min | 51 min |
| 12 | CI & Merge | 10 min | 61 min |

**Total Wall-Clock Time**: ~60 minutes (mostly automated)

## Next Action

Execute steps 1-12 in sequence per this plan. All analysis artifacts (mapping, divergence report, analysis document) are ready in `.dedup/`.

---

**Generated**: 2026-03-30
**Author**: Claude Code Agent
**Status**: Ready for execution
