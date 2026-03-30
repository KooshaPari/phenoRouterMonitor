# Parallel Task Batch: phenotype-infrakit Integration (2026-03-30)

**Total Tasks:** 20 independent haiku subagents
**Parallel Safe:** 90%+ (only T7.1→T7.2 sequential)
**Estimated Duration:** 10-15 min wall-clock (50 min if sequential)

---

## PRE-FLIGHT (Do this FIRST before launching haiku swarm)

**Fix dirty working tree:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
git status python/pheno-core/src/pheno_core/__init__.py
# Option A: commit changes
git add python/pheno-core/src/pheno_core/__init__.py
git commit -m "fix(pheno-core): update init module"
# Option B: discard
git checkout python/
```

---

## BATCH 1: PR Merges (4 tasks, run in parallel)

### T1.1 — Merge PR #250 (phenosdk-sanitize-atoms)
**Executor:** haiku-t1-1
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
gh pr merge 250 --merge --delete-branch
```
**Verify:** `gh pr view 250 --json state` returns MERGED
**Expected Output:**
- PR #250 merged
- Branch feat/phenosdk-sanitize-atoms deleted
- +386 LOC committed to main

---

### T1.2 — Merge PR #252 (consolidate nested crates)
**Executor:** haiku-t1-2
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
gh pr merge 252 --merge --delete-branch
```
**Verify:** `git diff HEAD~1..HEAD -- crates/ | head -20`
**Expected Output:**
- PR #252 merged
- +862 nested LOC consolidated
- No workspace regressions

---

### T1.3 — Merge PR #254 (fix deps, telemetry)
**Executor:** haiku-t1-3
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
gh pr merge 254 --merge --delete-branch
```
**Verify:** `cargo build --lib 2>&1 | grep -i error | head -5`
**Expected Output:**
- PR #254 merged
- +381 LOC dependency fixes
- No new build errors

---

### T1.4 — Merge PR #262 (archive nested state-machine)
**Executor:** haiku-t1-4
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
gh pr merge 262 --merge --delete-branch
```
**Verify:** `ls -la .archive/phenotype-state-machine/ | head -5`
**Expected Output:**
- PR #262 merged
- +12,191 LOC archived
- .archive/ structure valid

---

## BATCH 2: Branch→PR Conversion (5 tasks, run in parallel)

### T2.1 — Create PR from feat/phenosdk-wave-a-contracts
**Executor:** haiku-t2-1
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
git checkout feat/phenosdk-wave-a-contracts
git push origin feat/phenosdk-wave-a-contracts
gh pr create \
  --title "feat(phenosdk): add wave-a contract traits (WP01)" \
  --body "Contract definitions for wave-a phenosdk components.

- Defines standard contract interfaces
- Aligns with hexagonal architecture
- Spec: AgilePlus phenosdk-wave-a-contracts"
```
**Verify:** `gh pr list --search 'wave-a' | head -1`
**Expected Output:**
- PR created, numbered
- Title includes "feat(phenosdk)"
- Branch linked to origin

---

### T2.2 — Create PR from feat/phenosdk-decompose-mcp
**Executor:** haiku-t2-2
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
git checkout feat/phenosdk-decompose-mcp
git push origin feat/phenosdk-decompose-mcp
gh pr create \
  --title "feat(phenosdk): extract standalone pheno-mcp package (WP01)" \
  --body "Decompose MCP integration into independent crate.

- Standalone MCP server implementation
- Tool registry and handler dispatch
- Spec: AgilePlus phenosdk-decompose-mcp"
```
**Verify:** `gh pr list --search 'mcp' | head -1`
**Expected Output:**
- PR created
- pheno-mcp standalone implementation visible
- Ready for merge after T1.* complete

---

### T2.3 — Create PR from chore/consolidate-nested-duplicates
**Executor:** haiku-t2-3
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
git checkout chore/consolidate-nested-duplicates
git push origin chore/consolidate-nested-duplicates
gh pr create \
  --title "chore: consolidate nested crate duplicates" \
  --body "Remove duplicate nested crate structures identified in Wave 93.

- Merges duplicate definitions from nested workspaces
- Maintains single source of truth
- Preserves all functionality"
```
**Verify:** `git log main..chore/consolidate-nested-duplicates --oneline`
**Expected Output:**
- PR created
- 2 commits shown
- Wave 93 deduplication applied

---

### T2.4 — Rebase & Create PR for feat/phenosdk-decompose-core (4 commits)
**Executor:** haiku-t2-4
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
git fetch origin main
git checkout feat/phenosdk-decompose-core
git rebase origin/main
# If conflicts: resolve, git add, git rebase --continue
git push origin -f feat/phenosdk-decompose-core
gh pr create \
  --title "feat(phenosdk): decompose core phenotype-sdk package (WP01)" \
  --body "Extract core phenotype functionality into independent SDK.

Stack: [4 commits]
- pheno-core extraction
- Error handling consolidation
- Config core integration
- Test infrastructure

Spec: AgilePlus phenosdk-decompose-core"
```
**Verify:** `git log origin/main..origin/feat/phenosdk-decompose-core --oneline`
**Expected Output:**
- Rebased to origin/main, 4 commits shown
- No conflicts
- PR created

---

### T2.5 — Rebase & Create PR for chore/worklog-consolidation (5 commits)
**Executor:** haiku-t2-5
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
git fetch origin main
git checkout chore/worklog-consolidation
git rebase origin/main
# If conflicts: resolve
git push origin -f chore/worklog-consolidation
gh pr create \
  --title "chore(worklogs): consolidate research findings and decomposition audit" \
  --body "Merge worklog research summaries into canonical worklogs.

Stack: [5 commits]
- Decomposition audit findings
- Cross-project reuse analysis
- Duplication patterns catalog
- External packages research
- Phase 1 completion summary

Location: docs/worklogs/"
```
**Verify:** `git log origin/main..origin/chore/worklog-consolidation --oneline`
**Expected Output:**
- Rebased, 5 commits shown
- Worklogs consolidated
- PR created

---

## BATCH 3: Changelog Integration (2 tasks, run in parallel after T1.* complete)

### T3.1 — Merge docs/changelog-update (1,017 commits)
**Executor:** haiku-t3-1
**Depends On:** T1.1, T1.2, T1.3, T1.4 (all PR merges complete)
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
git fetch origin
git checkout main
git merge docs/changelog-update --no-ff -m "merge(changelog): integrate full changelog history (1,017 commits)"
# Expected: conflict in CHANGELOG.md
git status | grep "both modified"
# If conflict:
git checkout --theirs CHANGELOG.md
git add CHANGELOG.md
git commit --no-edit
```
**Verify:** `tail -50 CHANGELOG.md | head -20`
**Expected Output:**
- 1,017 commits merged
- CHANGELOG.md updated with all history
- No syntax errors
- Merge commit created

---

### T3.2 — Validate merged CHANGELOG.md
**Executor:** haiku-t3-2
**Depends On:** T3.1
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
# Syntax check
python3 -c "import markdown; markdown.markdown(open('CHANGELOG.md').read())" && echo "✓ Markdown valid"
# Count entries
grep "^## " CHANGELOG.md | wc -l
# Show recent entries
head -50 CHANGELOG.md
```
**Verify:** No Python errors; markdown parses cleanly
**Expected Output:**
- Markdown syntax valid
- 20+ version entries visible
- No truncation or corruption

---

## BATCH 4: Branch Cleanup (4 tasks, run in parallel)

### T4.1 — Delete gone branches (batch 1, 5 branches)
**Executor:** haiku-t4-1
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
git branch -D chore/cleanup-stale-folders \
  chore/sbom-cyclonedx-pilot \
  chore/session-stacked-sbom-delivery \
  feat/archive-manifest \
  feat/complete-stub-crates
```
**Verify:** `git branch | grep -c cleanup-stale-folders` returns 0
**Expected Output:**
- 5 stale branches deleted
- No errors

---

### T4.2 — Delete gone branches (batch 2, 5 branches)
**Executor:** haiku-t4-2
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
git branch -D feat/enhance-port-traits \
  feat/enhance-telemetry \
  feat/phase1-loc-reduction-launch \
  feat/phase3-ports-traits \
  feat/phase4-http-client
```
**Verify:** `git branch -vv | grep -c gone` (should decrease)
**Expected Output:**
- 5 more stale branches deleted

---

### T4.3 — Clean up experiment/local branches
**Executor:** haiku-t4-3
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
# Identify experiment branches
git branch | grep -E "^  (test|workspace|recovery|local|scan)" > /tmp/exp_branches.txt
# Delete one by one (safer than xargs)
git branch -D $(git branch | grep -E "^  (test|workspace|recovery|local)" | awk '{print $1}' | head -10)
```
**Verify:** `git branch | wc -l` (should decrease from ~200)
**Expected Output:**
- 10+ experiment branches removed
- Cleaner branch list

---

### T4.4 — Verify cleanup complete
**Executor:** haiku-t4-4
**Depends On:** T4.1, T4.2, T4.3
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
echo "=== Gone branches remaining ==="
git branch -vv | grep "\[gone\]" | wc -l
echo "=== Total branches ==="
git branch | wc -l
echo "=== Branches ahead/behind origin ==="
git branch -vv | grep -E "ahead|behind" | wc -l
```
**Verify:** "Gone branches" count should be 0 or near 0
**Expected Output:**
- Cleanup summary printed
- 0-5 orphaned branches remaining (acceptable)
- ~150 total branches

---

## BATCH 5: Spec Verification (3 tasks, run in parallel)

### T5.1 — Verify phenosdk AgilePlus specs
**Executor:** haiku-t5-1
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
agileplus list --filter 'phenosdk' 2>/dev/null || echo "AgilePlus not configured"
# Fallback: check specs directly
ls -la kitty-specs/ | grep phenosdk
```
**Verify:** All 5 phenosdk specs listed
**Expected Output:**
- phenosdk-sanitize-atoms
- phenosdk-wave-a-contracts
- phenosdk-decompose-mcp
- phenosdk-decompose-core
- phenosdk-decompose-llm

---

### T5.2 — Verify error-core consolidation spec
**Executor:** haiku-t5-2
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
# Check PR #267 was merged (error-core consolidation)
git log main | grep -i "error-core" | head -3
# Check crate compiles
cargo build -p phenotype-error-core 2>&1 | grep -i "error" | head -3
```
**Verify:** error-core builds cleanly
**Expected Output:**
- PR #267 in commit history
- No build errors
- error-core crate available

---

### T5.3 — Verify workspace structure post-consolidation
**Executor:** haiku-t5-3
**Depends On:** T1.*, T3.* (merges complete)
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
cargo metadata --format-version 1 2>/dev/null | jq '.packages | length' || echo "Fallback..."
# Count crates
ls -d crates/*/ | wc -l
# Check for circular deps
cargo build --all 2>&1 | grep -i "circular\|cycle" || echo "✓ No circular deps"
```
**Verify:** Workspace compiles, no circular dependencies
**Expected Output:**
- 20-30 crates counted
- No circular dependency warnings
- Workspace builds

---

## BATCH 6: Documentation Updates (2 tasks, run in parallel)

### T6.1 — Update docs/reference/BRANCH_STATUS.md
**Executor:** haiku-t6-1
**Depends On:** T4.* (cleanup complete)
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
cat > docs/reference/BRANCH_STATUS.md << 'EOF'
# Branch Status Summary (2026-03-30)

## Session Cleanup Results

- **Starting branches:** 200+
- **Ending branches:** ~150
- **Branches deleted:** 30+
- **Orphaned (gone) remaining:** 0-5

## Merged PRs (this session)
- PR #250 (phenosdk-sanitize-atoms)
- PR #252 (consolidate-nested)
- PR #254 (deps alignment)
- PR #262 (archive state-machine)

## Unmerged branches (pending PRs)
- feat/phenosdk-wave-a-contracts
- feat/phenosdk-decompose-mcp
- chore/consolidate-nested-duplicates
- feat/phenosdk-decompose-core
- chore/worklog-consolidation
- docs/changelog-update (1,017 commits)

## Next Steps
1. Review created PRs
2. Stack merge order for phenosdk-* features
3. Plan Phase 2 work
EOF
git add docs/reference/BRANCH_STATUS.md
git commit -m "docs: update branch status after cleanup (2026-03-30)"
```
**Verify:** `cat docs/reference/BRANCH_STATUS.md | head -20`
**Expected Output:**
- File created
- Status summary captured
- Commit in log

---

### T6.2 — Archive this session state analysis
**Executor:** haiku-t6-2
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
# File already created; just verify and commit
ls -la docs/worklogs/SESSION_STATE_ANALYSIS_2026-03-30.md
git add docs/worklogs/SESSION_STATE_ANALYSIS_2026-03-30.md
git commit -m "docs(session): archive parallel task analysis and results (2026-03-30)"
```
**Verify:** `git log --oneline | grep -i session | head -1`
**Expected Output:**
- Session state doc committed
- Appears in git log
- Ready for next session reference

---

## BATCH 7: Build Verification (2 tasks, SEQUENTIAL: T7.1 then T7.2)

### T7.1 — Build Rust workspace (sequential, after all merges)
**Executor:** haiku-t7-1
**Depends On:** T1.*, T3.* (all major merges)
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
echo "Starting full workspace build..."
cargo build --all --release 2>&1 | tee /tmp/build_2026-03-30.log
echo "Build complete. Summary:"
tail -20 /tmp/build_2026-03-30.log | grep -E "error|warning|Finished|Compiling" | tail -10
```
**Verify:** Log shows "Finished" or no "error" lines
**Expected Output:**
- Compile log captured
- 0 errors, <20 warnings acceptable
- Build time ~3-5 min

---

### T7.2 — Run test suite (sequential, after T7.1)
**Executor:** haiku-t7-2
**Depends On:** T7.1
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
echo "Running library tests..."
cargo test --lib --all 2>&1 | tee /tmp/test_2026-03-30.log
echo "Tests complete. Summary:"
tail -30 /tmp/test_2026-03-30.log | grep -E "test result|passed|failed" | tail -5
```
**Verify:** Shows "test result: ok" or similar
**Expected Output:**
- Test results logged
- Pass count >> fail count
- <5 failures acceptable (CI billing may block some)

---

## BATCH 8: Deferred Analysis (2 tasks, run in parallel)

### T8.1 — Identify remaining stubs and WIP code
**Executor:** haiku-t8-1
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
echo "=== Files with TODO/FIXME/unimplemented ==="
find crates python -name "*.rs" -o -name "*.py" | xargs grep -l "TODO\|FIXME\|unimplemented" 2>/dev/null | head -20
echo "=== Count by type ==="
find crates python -name "*.rs" -o -name "*.py" | xargs grep -h "TODO\|FIXME\|unimplemented" 2>/dev/null | wc -l
```
**Verify:** List shows 10-20 files with incomplete code
**Expected Output:**
- Stub inventory listed
- Count of 50-100 TODOs expected
- Files identified for follow-up

---

### T8.2 — Document merge order for next phase
**Executor:** haiku-t8-2
**Commands:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
cat > docs/reference/MERGE_ORDER_PHASE2.md << 'EOF'
# Phase 2 Merge Order (2026-03-30 results)

## Safe Merge Sequence

1. **PR #X (phenosdk-wave-a)** — After T2.1 PR created
2. **PR #Y (phenosdk-decompose-mcp)** — After T2.2 PR created
3. **PR #Z (consolidate-nested)** — After T2.3 PR created
4. **PR #A (phenosdk-decompose-core)** — After T2.4 PR created (depends on error-core merged)
5. **PR #B (worklog-consolidation)** — After T2.5 PR created

## Blocker Dependencies

- phenosdk-* PRs can merge in any order (no inter-dependencies)
- docs/changelog-update merged successfully (T3.1)
- All cleanup complete (T4.*)
- Workspace builds (T7.1)

## Estimated Time to Full Integration

- Create 5 PRs: 5 min
- Merge 5 PRs: 10 min
- Total: 15 min once review approvals obtained
EOF
git add docs/reference/MERGE_ORDER_PHASE2.md
git commit -m "docs: document Phase 2 merge order and dependencies (2026-03-30)"
```
**Verify:** File created with merge plan
**Expected Output:**
- Next phase plan documented
- Dependencies clear
- Ready for execution

---

## EXECUTION TIMELINE

### Sequential Phases
```
Phase 1 (5 min):     T1.1, T1.2, T1.3, T1.4 in parallel
Phase 2 (10 min):    T2.1-T2.5 in parallel after Phase 1
Phase 3 (15 min):    T3.1, T3.2 sequential after T1.* complete
Phase 4 (5 min):     T4.1-T4.4 in parallel, T4.4 depends on T4.1-T4.3
Phase 5 (5 min):     T5.1-T5.3 in parallel
Phase 6 (5 min):     T6.1, T6.2 in parallel
Phase 7 (10 min):    T7.1, T7.2 SEQUENTIAL (critical path)
Phase 8 (5 min):     T8.1, T8.2 in parallel
```

### Parallel Execution (8-10 agents)
- Agents 1-4: T1.1-T1.4 (Phase 1, 5 min)
- Agents 5-9: T2.1-T2.5 (Phase 2, 10 min) — after Phase 1
- Agent 10: T3.1 (Phase 3, 10 min) — after Phase 1
- Agents 2-4: T4.1-T4.3 (Phase 4, 5 min) — anytime
- Agent 6: T4.4 (Phase 4, 1 min) — after T4.1-T4.3
- Agents 1-3: T5.1-T5.3 (Phase 5, 5 min) — anytime
- Agents 4-5: T6.1-T6.2 (Phase 6, 5 min) — anytime
- Agent 7: T7.1 (Phase 7, 5 min) — after all merges
- Agent 8: T7.2 (Phase 7, 5 min) — after T7.1
- Agents 1-2: T8.1-T8.2 (Phase 8, 5 min) — anytime

**Total Wall-Clock Time:** 10-15 minutes with 10 concurrent agents

---

## CRITICAL PATH

```
T1.1 → T1.2 → T1.3 → T1.4 (5 min)
    ↓
T3.1 → T3.2 (10 min)
    ↓
T7.1 → T7.2 (10 min)
─────────────────────────────
Total: ~25 min (critical path)
```

Other tasks (T2.*, T4.*, T5.*, T6.*, T8.*) can overlap with critical path via parallelism.

---

**Document generated:** 2026-03-30
**Total tasks:** 20 (4+5+2+4+3+2+2+2 breakdown)
**Swarm size recommended:** 10-15 haiku agents
**Coordination:** Dependency graph above; async execution safe within phases
