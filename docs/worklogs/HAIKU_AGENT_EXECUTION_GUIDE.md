# Haiku Agent Execution Guide — Phase 3 Optimization
## Quick-Reference for Cheaper Agent Execution (15-minute onboarding)

---

## 🎯 Your Mission

Complete 8 work streams to finalize Phase 3 optimization. Each work stream = 1 haiku agent assigned.

**Total Wall-Clock:** ~2 hours (parallel execution)
**Total Haiku Agents:** 10-15
**Budget-Friendly:** 0.15x Sonnet cost

---

## 📋 Work Streams (Pick One, Complete It)

### Stream 1: HS1-Merge
**Task:** Merge feat/phenotype-crypto-implementation to main
**Effort:** 10-15 min
**Steps:**
1. Run: `cargo check` on current branch (verify no errors)
2. Create PR: `gh pr create --title "feat(crypto): integrate phenotype-crypto"` → main
3. Merge when ready
4. Verify: `cargo check --all` on main after merge

### Stream 2: HS2-CacheAdapter
**Task:** Standardize crates/phenotype-cache-adapter/Cargo.toml
**Effort:** 5 min
**Changes:**
```toml
# Before:
version = "0.2.0"
edition = "2021"
serde = { version = "1.0" }
dashmap = "5"

# After:
version.workspace = true
edition.workspace = true
serde.workspace = true
dashmap.workspace = true
```

### Stream 3: HS2-Logging
**Task:** Standardize crates/phenotype-logging/Cargo.toml
**Effort:** 3 min
**Changes:**
```toml
# Add to [package] section:
version.workspace = true
edition.workspace = true
license.workspace = true
```

### Stream 4: HS2-MCP
**Task:** Fix crates/phenotype-mcp/Cargo.toml
**Effort:** 5-10 min
**Changes:**
```toml
# Change:
dirs = "5.0"
# To:
dirs.workspace = true
# (workspace has dirs = "6.0")
```

### Stream 5: HS3-RemoveUnused
**Task:** Remove unused workspace dependencies
**Effort:** 5 min
**Steps:**
1. Open: Cargo.toml
2. Find: `moka = { version = "0.12", features = ["sync"] }`
3. Find: `lru = "0.12"`
4. Find: `parking_lot = "0.12"`
5. Delete all 3 lines
6. Run: `cargo check --all`
7. Verify: No new errors

### Stream 6: HS4-GitCore
**Task:** Fix crates/phenotype-git-core/src/lib.rs errors
**Effort:** 15-20 min
**Errors to Fix:**
- E0061: function takes more arguments
- E0282: type annotations needed
- E0599: method not found

**Action:** Review git2 v0.20 docs, update API calls

### Stream 7: HS4-StringRegex
**Task:** Fix phenotype-string regex compilation
**Effort:** 5-10 min
**Error:** `error[E0433]: failed to resolve: use of unresolved module or unlinked crate regex`
**Fix:**
1. Check: Does Cargo.toml have `regex = "1"` in workspace.dependencies? 
2. If no: Add it
3. Run: `cargo clean && cargo check -p phenotype-string`

### Stream 8: HS5-Roadmap
**Task:** Create docs/worklogs/OPTIMIZATION_ROADMAP_P1_P2_P3.md
**Effort:** 20-30 min
**Content:** Table of all P1-P3 optimization items (see PHASE_3_OPTIMIZATION_WORKLOG for format)

### Stream 9: HS5-Summary
**Task:** Create docs/worklogs/PHASE_3_COMPLETION_SUMMARY_2026-03-30.md
**Effort:** 15-20 min
**Contents:** Summarize all 8 work streams completed

### Stream 10: HS6-PRIntegration
**Task:** Track and merge open PRs
**Effort:** 30-45 min
**Steps:**
1. Run: `gh pr list --state open`
2. For each PR: Check if ready to merge (CI status, no conflicts)
3. Merge ready ones: `gh pr merge <PR#>`
4. Document any blocked/conflicted PRs

---

## ✅ Success Criteria Per Agent

- [ ] Task completed without errors
- [ ] Code changes verified with `cargo check` (if applicable)
- [ ] One commit created with proper message
- [ ] Memory updated with completion status
- [ ] No regressions introduced

---

## 📝 Standard Commit Format

```bash
git add <files>
git commit -m "$(cat <<'EOF'
<short description>

Co-Authored-By: Claude Haiku 4.5 <noreply@anthropic.com>
EOF
)"
```

**Examples:**
- `fix(cache-adapter): standardize to workspace dependencies`
- `refactor(logging): use workspace.package metadata`
- `chore(workspace): remove unused moka, lru, parking_lot`

---

## 🔧 Troubleshooting

### Problem: `cargo check` fails after my changes
**Solution:** Check that all `workspace = true` references match actual workspace.dependencies entries

### Problem: Build takes too long
**Solution:** Use `cargo check` (not `cargo build`) - it's 3-5x faster

### Problem: PR merge conflicts
**Solution:** Run `git pull origin main --rebase` before merging

### Problem: "Type annotations needed" error
**Solution:** Add explicit type hint: `let var: Type = ...;`

---

## 📊 Execution Order (Recommended)

**Batch 1 (Start First):**
1. HS1-Merge (critical path)
2. HS3-RemoveUnused (can run in parallel)

**Batch 2 (After Batch 1):**
3. HS2-CacheAdapter
4. HS2-Logging
5. HS2-MCP

**Batch 3 (After Batch 2):**
6. HS4-GitCore
7. HS4-StringRegex

**Batch 4 (Final):**
8. HS5-Roadmap
9. HS5-Summary
10. HS6-PRIntegration

---

## 🚨 If You Get Stuck

1. **Build fails:** Re-read task description, check example code
2. **Can't find file:** Use: `find . -name "<filename>"`
3. **Unsure about change:** Check git diff: `git diff --cached`
4. **Time running out:** Save progress in commit, update memory, escalate
5. **Type error:** Add explicit type annotation or check git2 v0.20 API docs

---

## 📍 Key Files to Know

- `Cargo.toml` (root) — workspace.dependencies declarations
- `crates/*/Cargo.toml` — individual crate manifests
- `crates/phenotype-{cache-adapter,logging,mcp}/Cargo.toml` — targets for WS2
- `crates/phenotype-git-core/src/lib.rs` — compilation errors
- `crates/phenotype-string/src/lib.rs` — regex errors
- `docs/worklogs/PHASE_3_OPTIMIZATION_WORKLOG_2026-03-30.md` — master reference

---

## ⏱️ Time Budget Per Stream

| Stream | Task | Time | Priority |
|--------|------|------|----------|
| HS1 | Merge branch | 15 min | 🔴 CRITICAL |
| HS2 | CacheAdapter | 5 min | 🟠 HIGH |
| HS2 | Logging | 3 min | 🟠 HIGH |
| HS2 | MCP | 10 min | 🟠 HIGH |
| HS3 | RemoveUnused | 5 min | 🟠 HIGH |
| HS4 | GitCore | 20 min | 🟠 HIGH |
| HS4 | StringRegex | 10 min | 🟠 HIGH |
| HS5 | Roadmap | 30 min | 🟡 MEDIUM |
| HS5 | Summary | 20 min | 🟡 MEDIUM |
| HS6 | PRIntegration | 45 min | 🟡 MEDIUM |
| **TOTAL** | | **2 hrs** | |

---

## 🎯 How to Know You're Done

- [ ] Your work stream task completed
- [ ] Code committed with proper message
- [ ] `cargo check` passes (if applicable)
- [ ] No unintended changes in git diff
- [ ] Memory updated: Mark your stream DONE ✅

---

## 🤝 Team Communication

**When you finish:** Update PHASE_3_OPTIMIZATION_WORKLOG_2026-03-30.md with ✅ next to your stream

**If blocked:** Post blocker in same worklog file under "Known Issues", pause work, wait for guidance

**When all done:** Lead will create final summary and close Phase 3

---

**Ready? Pick a stream above and get started! You've got this. 🚀**

