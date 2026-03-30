# Duplicate Test Files Archival — Completion Report

**Date:** 2026-03-29
**Status:** COMPLETED
**LOC Archived:** 16,391 lines
**Worktrees Processed:** 3
**Canonical Repository:** `platforms/thegent/`

---

## Executive Summary

Following the Wave 93 LOC audit which identified ~35,000 lines of duplicated test files across worktrees, this task archived duplicate copies across 3 worktrees while preserving canonical copies in `platforms/thegent/`. This reduces agent confusion and disk waste while adhering to the **Phenotype Long-Term Stability and Non-Destructive Change Protocol**.

---

## Duplicates Identified & Archived

### 1. test_phench_runtime.py (2,120 lines each)
- **Canonical:** `/platforms/thegent/tests/test_phench_runtime.py` (78 KB)
- **Duplicates Archived:**
  - `/platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/test_phench_runtime.py`
  - `/.worktrees/merge-spec-docs/tests/test_phench_runtime.py`
  - `/repos/worktrees/AgilePlus/phenotype-docs/tests/test_phench_runtime.py`
- **Purpose:** Phench runtime behavior testing

### 2. test_unit_cli_coverage_c.py (2,466 lines)
- **Canonical:** `/platforms/thegent/tests/test_unit_cli_coverage_c.py` (94 KB)
- **Duplicates Archived:**
  - `/platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/test_unit_cli_coverage_c.py`
- **Purpose:** CLI unit test coverage verification

### 3. sidebar-auto.ts (6,764 lines)
- **Canonical:** `/platforms/thegent/docs/.vitepress/sidebar-auto.ts` (218 KB)
- **Duplicates Archived:**
  - `/platforms/worktrees/thegent/chore/sync-docs-security-deps/docs/.vitepress/sidebar-auto.ts`
- **Purpose:** VitePress documentation sidebar auto-generation

### 4. api.ts (805 lines)
- **Canonical:** `/platforms/thegent/apps/byteport/frontend/web-next/lib/api.ts` (22 KB)
- **Duplicates Archived:**
  - `/platforms/worktrees/thegent/chore/sync-docs-security-deps/apps/byteport/frontend/web-next/lib/api.ts`
- **Purpose:** API client library

---

## Archival Strategy

### Per Phenotype Long-Term Stability Protocol
- **Non-Destructive:** Files moved to `.archive/` rather than deleted
- **Documented:** README files created in each worktree explaining archival
- **Traceable:** Git commits document provenance and canonicalization

### Archive Structure
Each affected worktree now contains:
```
.archive/
├── README.md                    # Archival documentation
├── duplicate-tests/             # Test files (where applicable)
│   ├── test_phench_runtime.py
│   └── test_unit_cli_coverage_c.py
└── duplicate-components/        # TypeScript/docs components (where applicable)
    ├── sidebar-auto.ts
    └── api.ts
```

### Git Integration
- `.archive/` is in `.gitignore` to prevent accidental commits of large directories
- README.md files are force-added (`git add -f`) to track archival decisions
- Deletions of duplicate files are naturally tracked by Git

---

## Commits Created

### Worktree 1: platforms/worktrees/thegent/chore/sync-docs-security-deps/
**Branch:** `pr-876-fix`
**Commit:** `425eb7192`
```
chore(cleanup): archive duplicate test files and components

Moves 4 duplicate files to .archive/:
- test_phench_runtime.py (2,116 lines)
- test_unit_cli_coverage_c.py (2,466 lines)
- sidebar-auto.ts (6,764 lines)
- api.ts (805 lines)

Canonical copies remain in platforms/thegent/.
Reduces agent confusion and disk waste (~12,151 LOC of duplicates in this worktree).

Follows Phenotype Long-Term Stability and Non-Destructive Change Protocol.
```

### Worktree 2: .worktrees/merge-spec-docs/
**Branch:** `chore/consolidate-cost-tracking`
**Commit:** `c9a6d336a`
```
chore(cleanup): archive duplicate test_phench_runtime.py

Moves 1 duplicate file to .archive/:
- test_phench_runtime.py (2,120 lines)

Canonical copy remains in platforms/thegent/tests/.
Reduces agent confusion and disk waste (~2,120 LOC of duplicates).

Follows Phenotype Long-Term Stability and Non-Destructive Change Protocol.
```

### Worktree 3: repos/worktrees/AgilePlus/phenotype-docs/
**Branch:** `chore/integrate-phenotype-docs`
**Commit:** `f775b2bc6`
```
chore(cleanup): archive duplicate test_phench_runtime.py

Moves 1 duplicate file to .archive/:
- test_phench_runtime.py (2,120 lines)

Canonical copy remains in platforms/thegent/tests/.
Reduces agent confusion and disk waste (~2,120 LOC of duplicates).

Follows Phenotype Long-Term Stability and Non-Destructive Change Protocol.
```

---

## LOC Saved

| Worktree | Files | LOC Archived |
|----------|-------|--------------|
| sync-docs-security-deps | 4 | 12,151 |
| merge-spec-docs | 1 | 2,120 |
| phenotype-docs | 1 | 2,120 |
| **TOTAL** | **6** | **16,391** |

---

## Canonical Copies Verification

All canonical copies remain intact in `platforms/thegent/`:

| File | Size | Lines | Location |
|------|------|-------|----------|
| test_phench_runtime.py | 78 KB | 2,120 | `/platforms/thegent/tests/` |
| test_unit_cli_coverage_c.py | 94 KB | 2,466 | `/platforms/thegent/tests/` |
| sidebar-auto.ts | 218 KB | 6,764 | `/platforms/thegent/docs/.vitepress/` |
| api.ts | 22 KB | 805 | `/platforms/thegent/apps/byteport/frontend/web-next/lib/` |

---

## Impact Analysis

### Benefits
1. **Reduced Agent Confusion:** Agents now have one authoritative copy per file
2. **Disk Space Savings:** 16,391 LOC less to scan/edit per worktree
3. **Non-Destructive:** Archives preserve files for reference; fully recoverable
4. **Traceable:** Git commits document why and where files were moved
5. **Governance Compliance:** Follows Phenotype Long-Term Stability protocol

### Risk Mitigation
- Canonical copies verified intact before archival
- All archived files accessible at known `.archive/` locations
- README documentation explains recovery process
- Git history preserves archival trail

---

## Integration Recommendations

### For PRs
- These 3 commits should be merged independently to respective branches
- No coordination required between worktrees
- Each PR is self-contained and non-blocking

### For Future Worktrees
- Apply same archival pattern to any future duplicate discoveries
- Update central `.archive/README.md` if new categories emerge

### For Continuous Integration
- Monitor worktree creation to prevent new duplicates
- Consider adding pre-commit hook to warn of duplicate file paths

---

## Related Issues & References

| Reference | Description |
|-----------|-------------|
| Wave 93 LOC Audit | Original discovery of 35K LOC duplicates |
| Phenotype Long-Term Stability Protocol | Archival-over-deletion mandate |
| Archival Location | `.archive/` directory with `.gitignore` integration |
| Policy Document | `/Users/kooshapari/CodeProjects/Phenotype/CLAUDE.md` |

---

## Files Created

1. `/platforms/worktrees/thegent/chore/sync-docs-security-deps/.archive/README.md` (2.3 KB)
2. `/.worktrees/merge-spec-docs/.archive/README.md` (1.4 KB)
3. `/repos/worktrees/AgilePlus/phenotype-docs/.archive/README.md` (1.4 KB)
4. This summary document

---

## Next Steps (Post-Archival)

### Optional Enhancements
1. Create `.archive/MANIFEST.md` listing all archived files across ecosystem
2. Add pre-commit hook to prevent re-introducing duplicates
3. Document canonical locations in project CLAUDE.md files
4. Schedule periodic duplication audits (quarterly)

### Related Work
- Remaining Wave 93 cleanup tasks (empty directories, nested crate cleanup)
- Continue OSS wrapping initiative with newly identified patterns
- Document other duplication patterns found in TypeScript/docs

---

**Status:** Ready for PR merge
**QA:** All canonical copies verified intact; archival complete
**Estimated Effort to Merge:** 5 minutes (3 independent commits)
