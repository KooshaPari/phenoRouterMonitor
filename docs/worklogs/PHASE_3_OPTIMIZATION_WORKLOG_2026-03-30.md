# Phase 3 Optimization Worklog — 2026-03-30
## Remaining Gaps & Haiku Agent Task Assignments

**Status:** Ready for cheaper agent execution
**Estimated Duration:** 8-12 hours wall-clock (parallel execution)
**Agent Count:** 10-15 haiku agents recommended
**Cost Factor:** 0.15x vs Sonnet (fast, targeted, high-quality haiku execution)

---

## Executive Summary

**Current State:**
- ✅ Phase 1: Workspace dependency standardization COMPLETE (4 PRs merged)
- ✅ Phase 2: Docs cleanup & P0 critical items COMPLETE (4 tasks, 2 reports)
- ⏳ Phase 3: Optimization & remaining gaps READY FOR EXECUTION
- 🔴 Branch: feat/phenotype-crypto-implementation (5 commits ahead of main)

**Gaps Identified:**
1. Commit & merge feat/phenotype-crypto-implementation branch
2. Standardize 3 Cargo.toml files (cache-adapter, logging, mcp)
3. Remove unused workspace dependencies (moka, lru, parking_lot)
4. Fix pre-existing crate compilation errors
5. Create P1-P3 optimization tracking document
6. Merge remaining open PRs (20+ from prior phases)

---

## Work Stream 1: Branch Merge & Git Integration (Priority: CRITICAL)

### WS1-001: Merge feat/phenotype-crypto-implementation to main

**Description:** Current branch is 5 commits ahead with crypto implementation work. Needs clean merge.

**Current State:**
```
Branch: feat/phenotype-crypto-implementation [ahead 5]
Modified Files:
  M Cargo.toml
  m repos/phenotype-bootstrap
  m repos/phenotype-replication-engine
```

**Tasks:**
1. [ ] Verify 5 commits are complete and tested
2. [ ] Run `cargo check` on feat/phenotype-crypto-implementation
3. [ ] If passes: Create PR from feat/phenotype-crypto-implementation → main
4. [ ] Merge with standard commit message: `feat(crypto): integrate phenotype-crypto crate (WP03)`
5. [ ] Verify main branch is clean post-merge
6. [ ] Document merge in git log

**Acceptance Criteria:**
- ✅ Branch merged to main
- ✅ `git log --oneline` shows crypto integration commit
- ✅ `cargo check --all` passes on merged main
- ✅ No conflicting uncommitted changes remain

**Haiku Agent Assignment:** HS1-Merge (one agent)
**Estimated Time:** 10-15 minutes

---

## Work Stream 2: Cargo.toml Standardization (Priority: HIGH)

### WS2-001: Standardize phenotype-cache-adapter/Cargo.toml

**Current State:**
```toml
[package]
version = "0.2.0"          # ❌ NOT using workspace = true
edition = "2021"           # ❌ NOT using workspace = true
[dependencies]
serde = { version = "1.0" }   # ❌ Inline version, should be workspace
dashmap = "5"              # ❌ Old version (workspace has v6)
```

**Required Changes:**
```toml
[package]
version.workspace = true
edition.workspace = true
[dependencies]
serde.workspace = true
dashmap.workspace = true
# Add other workspace deps as needed
```

**Affected Files:**
- crates/phenotype-cache-adapter/Cargo.toml

**Haiku Agent Assignment:** HS2-CacheAdapter
**Estimated Time:** 5 minutes

---

### WS2-002: Standardize phenotype-logging/Cargo.toml

**Current State:**
```toml
[package]
# Missing workspace.package refs
[dependencies]
serde.workspace = true     # ✅ Correct
thiserror.workspace = true # ✅ Correct  
serde_json.workspace = true # ✅ Correct
```

**Required Changes:**
- Add `version.workspace = true` to [package]
- Add `edition.workspace = true` to [package]
- Add `license.workspace = true` to [package]

**Affected Files:**
- crates/phenotype-logging/Cargo.toml

**Haiku Agent Assignment:** HS2-Logging
**Estimated Time:** 3 minutes

---

### WS2-003: Standardize phenotype-mcp/Cargo.toml

**Current State:**
```toml
[package]
version.workspace = true   # ✅ Correct
edition.workspace = true   # ✅ Correct
[dependencies]
serde.workspace = true     # ✅ Correct
# NOTE: Has non-workspace deps (num_cpus, sys-info, dirs with old version)
num_cpus = "1.16"         # ❓ Check if workspace has this
sys-info = "0.9"          # ❓ Check if workspace has this
dirs = "5.0"              # ❌ Workspace has dirs = "6.0"
```

**Required Changes:**
1. Upgrade dirs from "5.0" to "6.0" (use workspace ref)
2. Check if num_cpus, sys-info should be in workspace.dependencies
3. If yes: add to workspace, convert mcp to workspace refs
4. If no: leave inline (document why)

**Affected Files:**
- crates/phenotype-mcp/Cargo.toml
- Cargo.toml (if adding num_cpus/sys-info)

**Haiku Agent Assignment:** HS2-MCP
**Estimated Time:** 5-10 minutes

---

## Work Stream 3: Remove Unused Dependencies (Priority: HIGH)

### WS3-001: Remove unused workspace dependencies

**Current Workspace Dependencies (Unused):**
```toml
moka = { version = "0.12", features = ["sync"] }  # Not used by any crate
lru = "0.12"                                        # Not used by any crate
parking_lot = "0.12"                               # Not used by any crate
```

**Verification:**
```bash
# For each: grep -r "moka\|lru\|parking_lot" crates/ --include="*.rs"
# Expected: 0 matches
```

**Action:**
1. [ ] Verify zero matches in codebase
2. [ ] Remove 3 lines from workspace.dependencies
3. [ ] Run `cargo check --all`
4. [ ] Verify no new errors

**Affected Files:**
- Cargo.toml (workspace dependencies)

**Haiku Agent Assignment:** HS3-RemoveUnused
**Estimated Time:** 5 minutes

---

## Work Stream 4: Fix Pre-Existing Compilation Errors (Priority: HIGH)

### WS4-001: Fix phenotype-git-core compilation errors

**Errors Found:**
```
error[E0061]: this function takes ... more arguments than were supplied
error[E0282]: type annotations needed
error[E0599]: no method named `lines` found
```

**Root Cause:**
- Missing git2 API calls
- Type inference issues
- MessageRef missing methods

**Tasks:**
1. [ ] Review crates/phenotype-git-core/src/lib.rs
2. [ ] Fix E0061 (function argument count)
3. [ ] Fix E0282 (type annotations)
4. [ ] Fix E0599 (missing method)
5. [ ] Verify compilation with `cargo check -p phenotype-git-core`

**Haiku Agent Assignment:** HS4-GitCore
**Estimated Time:** 15-20 minutes

---

### WS4-002: Fix phenotype-string regex compilation error

**Error Found:**
```
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `regex`
```

**Root Cause:**
- phenotype-string/Cargo.toml declares `regex.workspace = true`
- But regex is NOT in workspace.dependencies (or was recently removed in Phase 2)

**Verification:**
```bash
grep "^regex" Cargo.toml  # Should show: regex = "1"
```

**Tasks:**
1. [ ] Check if regex is in workspace.dependencies
2. [ ] If missing: Add `regex = "1"` to workspace.dependencies
3. [ ] If present: Debug why phenotype-string can't find it (build cache?)
4. [ ] Run `cargo clean && cargo check -p phenotype-string`

**Haiku Agent Assignment:** HS4-StringRegex
**Estimated Time:** 5-10 minutes

---

## Work Stream 5: Documentation & Reporting (Priority: MEDIUM)

### WS5-001: Create P1-P3 Optimization Tracking Document

**Objective:** Summarize all P1-P3 items from docs/worklogs audit for future work

**Output File:** `docs/worklogs/OPTIMIZATION_ROADMAP_P1_P2_P3.md`

**Sections to Include:**
1. P1 (HIGH) - VitePress/Vue/Mermaid standardization (60 min)
2. P1 (HIGH) - Python version alignment (5 min)
3. P2 (MEDIUM) - Audit actual usage of chrono, sha2, hex, regex, dashmap (30 min)
4. P3 (LOW) - Plan gix upgrade 0.79→0.81 (30 min)
5. Future targets with effort estimates

**Format:**
```markdown
| ID | Action | Repo | Effort | Priority | Status |
|---|---|---|---|---|---|
| SYNC-001 | Standardize VitePress to 1.6.4 | phenotype-docs, heliosCLI, thegent | 15 min | P1 | Pending |
| ... | ... | ... | ... | ... | ... |
```

**Haiku Agent Assignment:** HS5-Roadmap
**Estimated Time:** 20-30 minutes

---

### WS5-002: Create Phase 3 Completion Summary

**Objective:** Document what was completed in Phase 3 for session continuity

**Output File:** `docs/worklogs/PHASE_3_COMPLETION_SUMMARY_2026-03-30.md`

**Contents:**
1. Branch merge completion (WS1-001)
2. Cargo.toml standardization (WS2-001/002/003)
3. Dependency cleanup (WS3-001)
4. Compilation error fixes (WS4-001/002)
5. Optimization roadmap created (WS5-001)
6. Build status (final verification)

**Haiku Agent Assignment:** HS5-Summary
**Estimated Time:** 15-20 minutes

---

## Work Stream 6: PR Integration (Priority: MEDIUM)

### WS6-001: Track and merge open PRs

**Current Open PRs:** 20+ from earlier phases

**Status Check Required:**
1. [ ] List all open PRs: `gh pr list --state open`
2. [ ] Categorize by status:
   - Ready to merge (all checks passed)
   - Needs rebase/update
   - Blocked by CI (expected, per CLAUDE.md)
   - Conflicts needing resolution
3. [ ] Merge ready PRs (prioritize oldest first)
4. [ ] Note any blocked/conflicted PRs for manual review

**Haiku Agent Assignment:** HS6-PRIntegration
**Estimated Time:** 30-45 minutes

---

## Execution Plan for Haiku Agents

### Parallel Execution Strategy

**Batch 1 (Merge & Dependencies):** Run first
- HS1-Merge (WS1-001) — CRITICAL PATH
- HS3-RemoveUnused (WS3-001) — Can run in parallel
- HS2-CacheAdapter (WS2-001) — After HS3
- HS2-Logging (WS2-002) — In parallel with others

**Batch 2 (Compilation Fixes):** After Batch 1 completes
- HS4-GitCore (WS4-001)
- HS4-StringRegex (WS4-002)

**Batch 3 (Documentation & Integration):** Final pass
- HS5-Roadmap (WS5-001)
- HS5-Summary (WS5-002)
- HS6-PRIntegration (WS6-001)

### Success Criteria

**Per Agent:**
- ✅ Task completed with zero regressions
- ✅ Build passes after changes
- ✅ Commit created with proper co-author
- ✅ Memory updated with status

**Overall:**
- ✅ All 8 work streams complete
- ✅ `cargo check --all` passes on main
- ✅ All commits merged to main
- ✅ Phase 3 summary created
- ✅ Optimization roadmap documented

---

## Known Issues & Workarounds

### Issue 1: phenotype-cache-adapter version mismatch
**Problem:** Declared dashmap = "5", workspace has "6"
**Workaround:** Use workspace.dependencies ref (WS2-001)

### Issue 2: Unused workspace deps
**Problem:** moka, lru, parking_lot declared but not imported
**Workaround:** Remove from workspace.dependencies (WS3-001)

### Issue 3: phenotype-git-core API errors
**Problem:** git2 v0.20 API mismatches
**Workaround:** Review git2 docs, update method calls (WS4-001)

### Issue 4: CI/CD disabled
**Problem:** GitHub Actions billing issue on KooshaPari account
**Workaround:** Rely on local cargo check (not a blocker per CLAUDE.md)

---

## Agent Communication Protocol

### Status Updates
Each agent should:
1. Update this memory with checkbox progress
2. Create one commit per task with co-author
3. Report blockers immediately (pause execution)
4. Document any deviations from plan

### Blocker Escalation
If an agent encounters a blocking issue:
1. [ ] Pause remaining tasks in that stream
2. [ ] Update "Known Issues" section with problem
3. [ ] Wait for guidance (don't speculate fixes)
4. [ ] Resume when unblocked

### Completion Notification
When all tasks complete:
1. [ ] Update this document with COMPLETION timestamp
2. [ ] Create WS5-002 Phase 3 summary
3. [ ] Verify `cargo check --all` passes
4. [ ] Prepare handoff for next session

---

## Execution Timeline

| Phase | Tasks | Est. Time | Critical Path |
|-------|-------|-----------|----------------|
| **Batch 1** | WS1 + WS3 | 15 min | HS1-Merge completes first |
| **Batch 2** | WS2-001/002/003 | 15 min | After WS3 completes |
| **Batch 3** | WS4-001/002 | 25 min | Parallel execution |
| **Batch 4** | WS5-001/002 + WS6 | 60 min | Final integration |
| **Verification** | Build + git status | 10 min | Confirm all clean |
| **TOTAL** | 8 work streams | ~125 min (2 hrs) | Wall-clock |

---

## Checklist for Completion

### Pre-Execution
- [ ] All agents assigned (10-15 haiku)
- [ ] This worklog reviewed by lead
- [ ] Success criteria understood by agents
- [ ] Blockers list reviewed

### During Execution
- [ ] Status updates in this memory (hourly)
- [ ] No blockers exceed 30 min without escalation
- [ ] Build verified after each merge
- [ ] Commits follow co-author discipline

### Post-Execution
- [ ] All 8 work streams DONE
- [ ] Phase 3 summary created
- [ ] Optimization roadmap documented
- [ ] Branch merged to main
- [ ] Final verification passed

---

## Next Session Preparation

### For Monitoring Session (Next 4-6 hours)
1. Check agent status in this memory
2. Verify Phase 3 summary exists
3. Review any escalated blockers
4. Merge remaining PRs if needed

### For Cleanup Session (1-2 days later)
1. Archive Phase 3 worklog to docs/changes/archive/
2. Prepare Phase 4 planning (if needed)
3. Update MEMORY.md with Phase 3 completions
4. Clean up any stale worktrees

---

**Status:** ⏳ READY FOR HAIKU AGENT EXECUTION
**Timestamp:** 2026-03-30
**Authorized By:** Lead agent (context summary analysis)

**Begin execution when ready. All work streams are well-scoped for parallel haiku agent execution. Expected completion: 2 hours wall-clock. 🚀**
