# Dependency Phase 2 — Execution Start (2026-04-01)

**Timeline:** 2026-04-01 to 2026-04-02 (1.5–2 days wall-clock, 20 hours effort)
**Execution Model:** Parallel Tracks (Track A + Track B)
**Canonical Repo:** `/Users/kooshapari/CodeProjects/Phenotype/repos`
**Branch:** `main`

---

## Quick Start for Agents

### Track A Agent (Consolidation Work — 8 hours Day 1)

**Your Work:**
1. **WP1 (1h):** Create cargo-udeps CI check
   - Create `.github/workflows/cargo-udeps.yml`
   - Set up blocking GitHub Actions for PRs
   - Document in `docs/reports/CARGO_UDEPS_SETUP.md`

2. **WP2 (2h):** Remove anyhow from 11 lib crates
   - Replace with `thiserror`
   - Update error definitions
   - Test: `cargo build --workspace` + `cargo test --workspace`
   - Document in `docs/reports/ANYHOW_REMOVAL_REPORT.md`

3. **WP3 (1h):** Consolidate feature flags
   - Move to workspace-level in root `Cargo.toml`
   - Update 28 crate `Cargo.toml` files
   - Test: `cargo build --all-features` + `--no-default-features`
   - Document in `docs/reports/FEATURE_CONSOLIDATION_REPORT.md`

4. **WP4 (4h):** Merge 52 error types into 8 canonical
   - Create canonical error enum in `phenotype-error-core`
   - Update 12 lib crates
   - Implement From<T> conversions
   - Test: `cargo test --workspace`
   - Document in `docs/reports/ERROR_CONSOLIDATION_REPORT.md`

**Checkpoints:**
- After WP1-3 (4h): `cargo build --workspace --all-features` + `--no-default-features`
- After WP4 (4h): `cargo test --workspace` + `cargo clippy --workspace -- -D warnings`

---

### Track B Agent (Performance Optimization — 8 hours Day 1)

**Your Work:**
1. **WP5 (2h):** Lazy-initialize 8+ regex patterns
   - Find regex usage: `grep -rn "regex::Regex::new" crates/*/src/`
   - Replace with `lazy_static!` from `lazy_static` crate
   - Test builds
   - Document in `docs/reports/REGEX_LAZY_INIT_REPORT.md`

2. **WP6 (4h):** Remove 45+ dead code suppressions
   - Find: `grep -rn "#\[allow(dead_code)\]" crates/`
   - Analyze each: Is it actually dead?
   - Remove suppressions + code for true dead code
   - Keep suppressions for intentional public APIs
   - Test: `cargo clippy --workspace -- -D warnings` (should be 0)
   - Document in `docs/reports/DEAD_CODE_REMOVAL_REPORT.md`

3. **WP7 (2h):** Audit 8 unsafe blocks (if time permits)
   - Find: `grep -rn "unsafe {" crates/*/src/`
   - Add SAFETY comments to each block
   - Verify no memory safety issues
   - Document in `docs/reports/UNSAFE_AUDIT_REPORT.md`

**Checkpoints:**
- After WP5-6 (6h): `cargo clippy --workspace -- -D warnings` (0 warnings expected)
- After WP7 (2h): All unsafe blocks have SAFETY comments

---

### Day 2 Sync Agent (Benchmarking & Documentation — 4 hours)

**Your Work:**
1. **WP8 (2h):** Performance benchmarking
   - Measure cold build: `time cargo build --workspace --release`
   - Measure incremental: `touch crates/*/src/lib.rs && time cargo build --release`
   - Measure binary size: `ls -lh target/release/phenotype-*`
   - Document in `docs/reports/PHASE2_PERFORMANCE_METRICS.md`

2. **WP9 (2h):** Documentation & completion
   - Update `CHANGELOG.md` with all 9 WPs
   - Create `docs/reference/DEPENDENCY_PHASE2_MIGRATION_GUIDE.md`
   - Create `docs/reference/DEPENDENCY_PHASE2_TROUBLESHOOTING.md`
   - Create `docs/reports/PHASE2_COMPLETION_CHECKLIST.md`

---

## Baseline State (2026-03-31)

```
Crates:           24 production + dependencies
Cargo.toml files: 60 across crates/
anyhow usage:     11 crates
dead_code:        62 suppressions
unsafe blocks:    142 instances
Build time:       81.2s (baseline)
Binary size:      3.2 MB (baseline)
Branch:           main (synced with origin/main)
Last commit:      a67fff87b (2026-03-30)
```

---

## Reference Documents

**Master Plan:** `/Users/kooshapari/CodeProjects/Phenotype/repos/DEPENDENCY_PHASE2_EXECUTION_PLAN.md`

**Validation:** `/Users/kooshapari/CodeProjects/Phenotype/repos/DEPENDENCY_PHASE2_VALIDATION.md`

**WP Details:** See plan document for full specifications (20 pages)

---

## Daily Sync Commands (Run Each Checkpoint)

```bash
# After each agent completes their WP
cd /Users/kooshapari/CodeProjects/Phenotype/repos

# Build test
cargo build --workspace

# Test suite
cargo test --workspace

# Quality check
cargo clippy --workspace -- -D warnings

# Format check
cargo fmt --check

# Metrics
echo "Crates with anyhow: $(grep -l 'anyhow' crates/*/Cargo.toml | wc -l)"
echo "Dead code suppressions: $(grep -r '#\[allow(dead_code)\]' crates/ | wc -l)"
echo "Build artifact size: $(du -sh target | cut -f1)"
```

---

## Expected Outcomes (By 2026-04-02 EOD)

1. ✅ **Build time:** 81.2s → <65s (-20%)
2. ✅ **Binary size:** 3.2MB → <3.05MB (-5%)
3. ✅ **Error consolidation:** 52 → 8 types
4. ✅ **Dead code:** 62 → ≤17 suppressions (45 removed)
5. ✅ **anyhow:** Removed from 11 lib crates
6. ✅ **Feature flags:** Consolidated to workspace level
7. ✅ **Regex optimization:** 8+ patterns lazy-initialized
8. ✅ **Unsafe audit:** 142 blocks documented with SAFETY comments
9. ✅ **All tests:** Pass with 0 warnings
10. ✅ **Documentation:** 10 reports + migration guide + troubleshooting

---

## Success Criteria (All Must Pass)

```
✅ cargo build --workspace (no errors)
✅ cargo test --workspace (all passing)
✅ cargo clippy --workspace -- -D warnings (0 warnings)
✅ Build time <65s (20% improvement from 81.2s)
✅ Binary size <3.05MB (5% improvement from 3.2MB)
✅ 11 crates migrated from anyhow to thiserror
✅ 52 → 8 error type consolidation verified
✅ 45+ dead_code suppressions removed
✅ 28 feature flags consolidated to workspace
✅ 8+ regex patterns lazy-initialized
✅ 142 unsafe blocks documented
✅ 10 reports generated
✅ Ready to merge to main
✅ Tag v0.3.0 prepared
```

---

## Timeline Overview

| When | Duration | What |
|------|----------|------|
| **2026-04-01 morning** | 4h | Track A: WP1-3 (Foundation) |
| **2026-04-01 morning** | 4h | Track B: WP5-6 (Optimization) |
| **2026-04-01 afternoon** | 4h | Track A: WP4 (Error consolidation) |
| **2026-04-01 afternoon** | 4h | Track B: WP5-6 complete |
| **2026-04-01 EOD** | — | Sync: Both tracks verify builds pass |
| **2026-04-02 morning** | 2h | Either: WP7 (Unsafe audit) |
| **2026-04-02 afternoon** | 2h | Either: WP8-9 (Bench + docs) |
| **2026-04-02 EOD** | — | Sign-off: All acceptance criteria met |

---

## Files to Review Before Starting

1. **DEPENDENCY_PHASE2_EXECUTION_PLAN.md** (20 pages)
   - Full WP specifications
   - Acceptance criteria
   - Detailed process for each task

2. **DEPENDENCY_PHASE2_VALIDATION.md** (12 pages)
   - Validation checkpoints
   - Expected results
   - Failure recovery procedures

3. **This document** (quick reference)

---

## Contact Points

If you hit a blocker:
1. Check **DEPENDENCY_PHASE2_VALIDATION.md** → **Failure Recovery Procedures**
2. Review the relevant **WP section** in **DEPENDENCY_PHASE2_EXECUTION_PLAN.md**
3. Run validation commands (see above)
4. Commit your work with `git add <files>` (avoid git stash)

---

## Next Phase (After Phase 2)

**Phase 3 (Roadmap):**
- Polyrepo vs. monorepo decision
- Macros audit & consolidation
- GC optimization
- Architecture finalization

**Estimated:** 2026-04-05 onwards

---

**Ready to Execute:** 2026-04-01 morning
**Questions?** Review the master plan (20 pages of detail)
**GO!** 🚀

---

*Created: 2026-03-31 | Status: Ready for Execution*
