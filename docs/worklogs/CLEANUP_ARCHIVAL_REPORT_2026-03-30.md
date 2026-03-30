# Directory Cleanup & Orphaned Worktrees Archival Report

**Execution Date:** 2026-03-30  
**Status:** Analysis Complete — Ready for User Action  
**Author:** Claude Code Cleanup Agent

---

## Executive Summary

Analysis of `/Users/kooshapari/CodeProjects/Phenotype/repos/` directory structure identified:

- **1 Empty Placeholder:** `worktrees/phenotype/` (0 files) — **SAFE TO DELETE**
- **6 Active Root-Level Worktrees:** 360M total in `worktrees/` directory
- **15 Active Hidden Worktrees:** ~7.5GB in canonical `.worktrees/` directory (governance-standard location)
- **Non-existent Directories:** `worktree/` and `add/` (no cleanup needed)

**Key Finding:** Repository has TWO worktree staging areas. Root-level `worktrees/` may be legacy infrastructure competing with canonical `.worktrees/`.

---

## Directory Analysis Detail

### 1. ROOT-LEVEL `worktrees/` Directory

**Status:** Needs Consolidation Review  
**Location:** `repos/worktrees/`  
**Total Size:** ~360M (3,010 files)

#### Contents Inventory:

| Name | Status | Size | Files | Modified |
|------|--------|------|-------|----------|
| `phenotype/` | **EMPTY PLACEHOLDER** | 0B | 0 | 2026-03-29 19:00 |
| `chore-docs-sbom-stack/` | ACTIVE | 4.9M | 297 | 2026-03-30 00:15 |
| `chore-sbom-cyclonedx/` | RECENT | 3.9M | 174 | 2026-03-29 17:48 |
| `chore-session-sbom-stack/` | RECENT | 3.9M | 175 | 2026-03-29 17:51 |
| `devenv-abstraction/` | ACTIVE | 84K | 21 | 2026-03-29 18:06 |
| `phenosdk-wave-a-contracts-impl/` | ACTIVE | 267M | 1,942 | 2026-03-29 18:03 |
| `phenotype-infrakit/` | ACTIVE | 8.2M | 401 | 2026-03-29 18:18 |

#### Analysis:

All root-level worktrees except `phenotype/` have been modified within the last 24-48 hours, indicating:
- Recent active use (not stale)
- Possibly related to bulk documentation/SBOM generation work
- Candidates for consolidation into `.worktrees/` per governance

**Recommendation:** 
- **Immediate:** Delete `worktrees/phenotype/` (confirmed empty)
- **Pending User Decision:** Consolidate remaining 6 active worktrees into `.worktrees/` or document if root-level staging is intentional

---

### 2. HIDDEN `.worktrees/` Directory (CANONICAL)

**Status:** ACTIVE & GOVERNANCE-STANDARD  
**Location:** `repos/.worktrees/`  
**Total Size:** ~7.5GB (21,247 files)

#### Contents Inventory (15 Worktrees):

| Name | Size | Files | Status | Modified |
|------|------|-------|--------|----------|
| `add-tests` | 4.6M | 280 | RECENT | 2026-03-29 20:35 |
| `chore` | 9.9M | 597 | RECENT | 2026-03-30 00:52 |
| `chore-govern-pi` | 4.3M | 221 | RECENT | 2026-03-29 18:28 |
| `cli-errors` | 523M | 2,526 | SUBSTANTIAL | 2026-03-29 20:35 |
| `feat` | 580M | 5,716 | SUBSTANTIAL | 2026-03-30 00:55 |
| `fix-clippy` | 836M | 6,672 | VERY ACTIVE | 2026-03-30 00:58 |
| `fix-event-sourcing` | 4.7M | 287 | RECENT | 2026-03-29 23:18 |
| `impl-contracts` | 4.6M | 280 | RECENT | 2026-03-29 20:35 |
| `impl-state-machine` | 1.1G | 10,009 | VERY ACTIVE | 2026-03-30 00:55 |
| `impl-test-infra` | 4.6M | 280 | RECENT | 2026-03-29 20:35 |
| `impl` | 1.3G | 9,019 | VERY ACTIVE | 2026-03-30 00:58 |
| `loc-reduction` | 516M | 3,866 | ACTIVE | 2026-03-30 00:54 |
| `merge-spec-docs` | 1.4G | 9,861 | VERY ACTIVE | 2026-03-30 01:03 |
| `phench` | 5.6M | 379 | RECENT | 2026-03-29 17:55 |
| `phenosdk-decompose-mcp-wp01` | 4.9M | 266 | RECENT | 2026-03-29 23:53 |

#### Analysis:

- All 15 worktrees are currently active with recent modifications (within 24-48 hours)
- Several are "VERY ACTIVE" (modified in last 2 hours): `fix-clippy`, `impl-state-machine`, `impl`, `merge-spec-docs`
- This is the governance-standard canonical worktree location per `CLAUDE.md`
- Contains ~5 substantial worktrees (>500M each)

**Recommendation:** KEEP. Do not delete or move without explicit user approval.

---

### 3. `worktree/` Directory

**Status:** DOES NOT EXIST  
**Action:** No cleanup required.

---

### 4. `add/` Directory

**Status:** DOES NOT EXIST  
**Note:** There is an `add-tests` worktree in `.worktrees/add-tests/`, but no top-level `add/` directory.  
**Action:** No cleanup required.

---

## Confirmed Deletions (Safe)

Only the following directory is confirmed safe for deletion:

```bash
# EMPTY PLACEHOLDER - 0 files, no dependencies
rm -rf repos/worktrees/phenotype/
```

This deletion is safe because:
1. Directory is completely empty (0 files)
2. No git tracking or references to this path found
3. Placeholder name suggests it was scaffolding that was never populated

---

## Orphaned Worktrees for Archival

**Current Status:** No orphaned worktrees identified.

All 21 worktrees identified (6 in `worktrees/` + 15 in `.worktrees/`) are either:
- **RECENT** — Modified within 24-48 hours
- **ACTIVE** — Modified within last 48 hours
- **VERY ACTIVE** — Modified within last 2 hours

**Action:** No archival required at this time. All worktrees appear to be in active use.

---

## Architecture Consolidation Needed (User Decision)

The repository currently maintains TWO worktree staging directories:

| Directory | Status | Size | Worktrees | Governance Status |
|-----------|--------|------|-----------|------------------|
| `repos/worktrees/` | ROOT-LEVEL | 360M | 6 active + 1 empty | NOT CANONICAL |
| `repos/.worktrees/` | HIDDEN | 7.5GB | 15 active | **CANONICAL** |

**Governance Standard:**
Per `Phenotype/repos/CLAUDE.md`, `.worktrees/` is the canonical location for all feature work and development worktrees. Root-level `worktrees/` directory is not mentioned in governance.

**Clarification Questions for User:**

1. **Is `repos/worktrees/` still in use?**
   - Recent timestamps suggest yes, but may be legacy
   - All 6 worktrees have active branches and content

2. **Should root-level worktrees be consolidated into `.worktrees/`?**
   - Would enforce governance standard (one canonical location)
   - Would simplify navigation and documentation
   - Requires moving 360M + 15 worktrees across locations

3. **Are the SBOM/chore worktrees in `worktrees/` generated or manual?**
   - Names suggest automated generation (chore-docs-sbom-stack, chore-sbom-cyclonedx, etc.)
   - May be output from CI/analysis pipeline
   - If automated, consider whether output should live in `.archive/` instead

---

## Recommended Action Plan

### Phase 1: Immediate (1 minute)
- [ ] Delete empty placeholder: `rm -rf repos/worktrees/phenotype/`
- [ ] Commit to git with message: `chore: remove empty placeholder directory worktrees/phenotype`

### Phase 2: Requires User Input (pending)
- [ ] Confirm: Should root-level `worktrees/` directory be retained or consolidated?
- [ ] If consolidated: Execute migration of 6 worktrees to `.worktrees/`
- [ ] If retained: Document why root-level `worktrees/` exists and update CLAUDE.md

### Phase 3: Long-Term Governance (optional)
- [ ] Clarify whether SBOM/chore output should be archived elsewhere
- [ ] Update CLAUDE.md to formalize worktree location policy
- [ ] Document any automated pipeline that populates `repos/worktrees/`

---

## Files Modified

- This report: `docs/worklogs/CLEANUP_ARCHIVAL_REPORT_2026-03-30.md`

---

## Sign-Off

**Analysis:** Complete  
**Safe Deletions Identified:** 1 (empty placeholder)  
**Orphaned Worktrees:** 0 (all active)  
**Consolidation Needed:** Pending user review  
**Next Steps:** User decision on Phase 2 actions

