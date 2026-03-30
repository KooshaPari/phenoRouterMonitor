# Test File Deduplication Analysis Index

**Analysis Date**: 2026-03-30
**Status**: Complete & Ready for Execution
**Branch Target**: `refactor/deduplicate-tests`

## Document Index

### Quick Start
1. **START HERE**: [README.md](./README.md) (2 min read)
   - Quick reference overview
   - Key metrics and findings
   - File directory index

### Analysis Documents
2. **Full Analysis**: [TEST_DEDUPLICATION_ANALYSIS.md](./TEST_DEDUPLICATION_ANALYSIS.md)
   - Complete findings and data
   - Canonical location rules
   - 7-phase implementation strategy
   - Timeline and risk assessment
   - 4,500+ lines of detailed analysis

3. **Special Cases**: [DIVERGENCE_REPORT.md](./DIVERGENCE_REPORT.md)
   - 18 files with diverged content
   - Canonical vs diverged paths
   - Action items for each divergence
   - Manual consolidation recommendations

4. **Data Mapping**: [test_duplication_map.json](./test_duplication_map.json)
   - Machine-readable mapping of all 1,268 duplicates
   - Canonical → [duplicate paths] structure
   - File sizes and SHA256 hashes
   - Divergence flags
   - 711 KB JSON file (1,268 entries)

### Execution Documents
5. **Execution Plan**: [docs/worklogs/DEDUPLICATION_EXECUTION_PLAN.md](../worklogs/DEDUPLICATION_EXECUTION_PLAN.md)
   - 12-step execution guide
   - Timeline breakdown
   - Verification procedures
   - Commit strategy
   - Risk mitigation
   - 500+ lines of step-by-step instructions

6. **Final Report**: [docs/worklogs/DEDUPLICATION_ANALYSIS_REPORT.md](../worklogs/DEDUPLICATION_ANALYSIS_REPORT.md)
   - Executive summary
   - Detailed findings with tables
   - Root cause analysis
   - Top 20 duplicated files
   - Impact analysis
   - Success criteria
   - 13+ KB comprehensive report

## How to Use These Documents

### For Decision Makers
1. Read [README.md](./README.md) (2 min)
2. Review key findings in this INDEX (5 min)
3. Decide on approval (go/no-go)

### For Technical Reviewers
1. Read [README.md](./README.md) (2 min)
2. Review [TEST_DEDUPLICATION_ANALYSIS.md](./TEST_DEDUPLICATION_ANALYSIS.md) - Strategy section (10 min)
3. Check [DIVERGENCE_REPORT.md](./DIVERGENCE_REPORT.md) for special cases (5 min)
4. Review risks in [TEST_DEDUPLICATION_ANALYSIS.md](./TEST_DEDUPLICATION_ANALYSIS.md) - Risk Assessment (5 min)

### For Implementers
1. Read [README.md](./README.md) (2 min)
2. Follow [docs/worklogs/DEDUPLICATION_EXECUTION_PLAN.md](../worklogs/DEDUPLICATION_EXECUTION_PLAN.md) (60 min)
3. Reference [test_duplication_map.json](./test_duplication_map.json) for verification
4. Check [DIVERGENCE_REPORT.md](./DIVERGENCE_REPORT.md) before removing any files

### For Code Reviewers
1. Review PR title and description
2. Check [docs/worklogs/DEDUPLICATION_ANALYSIS_REPORT.md](../worklogs/DEDUPLICATION_ANALYSIS_REPORT.md) for context
3. Verify CI tests pass on canonical paths
4. Ensure 1,298 files removed and 18 diverged files kept
5. Approve and merge

## Key Metrics at a Glance

```
PROBLEM
├── Duplicate filenames: 1,268
├── Duplicate instances: 1,316
├── Space consumed: 9.81 MB
├── LOC to remove: ~207,765
└── Canonical files: 1,281

SOLUTION
├── Remove identical duplicates: 1,298 files
├── Keep diverged copies: 18 files
├── Keep canonical files: 1,281 files
├── Space recovered: 9.81 MB (50.7% reduction)
└── Execution time: ~60 minutes

SAFEGUARDS
├── Hash verification: SHA256
├── Canonical locations: Verified
├── Diverged handling: Explicit keep list
├── Reversibility: Complete (git-based)
└── Risk level: LOW-MEDIUM (well-mitigated)
```

## Canonical Locations (What We're Keeping)

```
platforms/thegent/tests/              ~500+ files
python/pheno-mcp/tests/               ~150+ files
python/pheno-core/tests/              ~100+ files
heliosCLI/harness/tests/              ~200+ files
phench/tests/                         ~50+ files
AgilePlus/*/tests/                    ~200+ files
[Other projects]                      ~1+ files
────────────────────────────────────────────
TOTAL: ~1,281 canonical test files
```

## Worktrees Being Cleaned

```
.worktrees/*/python/*/tests/          (remove duplicates)
platforms/worktrees/thegent/*/tests/  (remove duplicates)
heliosCLI/worktrees/*/tests/          (remove duplicates)
heliosCLI/.worktrees/*/tests/         (remove duplicates)
```

## Top 5 Most Duplicated Files

| File | Copies | Canonical Location |
|------|--------|-------------------|
| test_schema.py | 7 | heliosCLI/harness/tests/ |
| test_resilience.py | 6 | platforms/thegent/tests/chaos/ |
| test_integration_mcp_tools_agents.py | 5 | python/pheno-mcp/tests/ |
| test_mcp_entry_points.py | 5 | python/pheno-mcp/tests/ |
| test_agents_orchestration.py | 5 | python/pheno-mcp/tests/ |

(See [TEST_DEDUPLICATION_ANALYSIS.md](./TEST_DEDUPLICATION_ANALYSIS.md) for complete top 20)

## Diverged Tests (18 Files - All Kept)

Files with different content in canonical vs worktree:

1. test_batch_file_ops.py
2. test_cache.py
3. test_cli.py
4. test_cross_project.py
5. test_enterprise_compliance.py
6. test_git_parallelism.py
7. test_injection.py
8. test_mcp_tools.py
9. test_observability.py
10. test_path_utils.py
11. test_project_registry.py
12. test_resilience.py (2 diverged)
13. test_runner.py
14. test_schema.py (2 diverged)
15. test_session_hook.py
16. test_store.py
17. test_sub_agent_dispatcher.py
18. test_wl185_reflection_rollback.py

See [DIVERGENCE_REPORT.md](./DIVERGENCE_REPORT.md) for full details.

## Quick Decision Matrix

| Question | Answer | Reference |
|----------|--------|-----------|
| Will tests break? | No (canonical tests preserved) | Analysis Report p.3 |
| How much space saves? | 9.81 MB (50.7% reduction) | README.md, top of page |
| How long to execute? | ~60 minutes (mostly automated) | Execution Plan, Step 1-12 |
| Can we rollback? | Yes (git feature branch) | TEST_DEDUPLICATION_ANALYSIS.md, Rollback Plan |
| Any manual work? | Yes, 18 diverged files (future consolidation) | DIVERGENCE_REPORT.md |
| Is it safe? | HIGH confidence (hash-verified, comprehensive safeguards) | Analysis Report, Risk Assessment |

## Execution Commands (TL;DR)

```bash
# Step 1: Create branch
git checkout -b refactor/deduplicate-tests

# Step 2: Remove 1,298 identical duplicates
# (See DEDUPLICATION_EXECUTION_PLAN.md for detailed removal commands)
git rm -r [duplicate test paths]  # Multiple batches per worktree

# Step 3: Verify canonical tests intact
test -d platforms/thegent/tests && echo "✓ OK"
test -d python/pheno-mcp/tests && echo "✓ OK"

# Step 4: Final commit
git commit -m "refactor: deduplicate test files (1,298 removed, 9.81 MB saved)"

# Step 5: Push and create PR
git push -u origin refactor/deduplicate-tests
gh pr create --title "refactor: deduplicate test files (1,298 removed, 9.81 MB saved)"

# Step 6: Monitor CI and merge
# [Wait for CI, code review, merge]
```

See [docs/worklogs/DEDUPLICATION_EXECUTION_PLAN.md](../worklogs/DEDUPLICATION_EXECUTION_PLAN.md) for complete detailed steps.

## Document Status & Completeness

| Artifact | Status | Lines | Notes |
|----------|--------|-------|-------|
| README.md | Complete | 150 | Quick reference |
| TEST_DEDUPLICATION_ANALYSIS.md | Complete | 4,500+ | Full strategy + phases |
| DIVERGENCE_REPORT.md | Complete | 300+ | 18 special files |
| test_duplication_map.json | Complete | 1,268 entries | Machine-readable mapping |
| DEDUPLICATION_EXECUTION_PLAN.md | Complete | 500+ | 12-step guide |
| DEDUPLICATION_ANALYSIS_REPORT.md | Complete | 13 KB | Comprehensive final report |

**Total Documentation**: 6,500+ lines

## Next Actions (Recommended Sequence)

### Phase 1: Review & Approval (15 min)
- [ ] Read README.md (2 min)
- [ ] Skim TEST_DEDUPLICATION_ANALYSIS.md strategy section (5 min)
- [ ] Review DIVERGENCE_REPORT.md summary (5 min)
- [ ] Decision: approve/defer/modify (3 min)

### Phase 2: Execution (60 min)
- [ ] Follow DEDUPLICATION_EXECUTION_PLAN.md step-by-step
- [ ] Use test_duplication_map.json for reference
- [ ] Verify canonical tests after each phase
- [ ] Watch CI on PR

### Phase 3: Integration (10 min)
- [ ] Code review PR
- [ ] Verify CI passed
- [ ] Merge to main

### Phase 4: Verification (5 min)
- [ ] Verify branch merged
- [ ] Confirm git clone size reduced
- [ ] Document final metrics

## Support & Questions

**Q: Which file should I read first?**
A: [README.md](./README.md) - 2 minute quick overview

**Q: How do I know what to remove?**
A: [test_duplication_map.json](./test_duplication_map.json) - machine-readable mapping, or [DIVERGENCE_REPORT.md](./DIVERGENCE_REPORT.md) for special cases

**Q: How do I execute this?**
A: [docs/worklogs/DEDUPLICATION_EXECUTION_PLAN.md](../worklogs/DEDUPLICATION_EXECUTION_PLAN.md) - 12-step detailed guide

**Q: What if something breaks?**
A: See "Rollback Plan" in [TEST_DEDUPLICATION_ANALYSIS.md](./TEST_DEDUPLICATION_ANALYSIS.md) - changes fully reversible

**Q: What are the special cases?**
A: [DIVERGENCE_REPORT.md](./DIVERGENCE_REPORT.md) - 18 files with diverged content (all kept)

## Analysis Confidence

- **Confidence Level**: HIGH
- **Method**: Deterministic static file analysis
- **Verification**: SHA256 hash comparison
- **Scope**: Complete repository scan (no sampling)
- **Reproducibility**: 100% (can verify independently)

## File Organization Summary

```
.dedup/
├── INDEX.md                            (this file - navigation guide)
├── README.md                           (quick reference)
├── TEST_DEDUPLICATION_ANALYSIS.md      (complete analysis)
├── DIVERGENCE_REPORT.md                (18 special files)
└── test_duplication_map.json           (complete data mapping)

docs/worklogs/
├── DEDUPLICATION_EXECUTION_PLAN.md     (12-step guide)
└── DEDUPLICATION_ANALYSIS_REPORT.md    (comprehensive report)
```

---

**Generated**: 2026-03-30
**Status**: Complete & Ready for Execution
**Author**: Claude Code Agent
**Recommendation**: Execute as planned

Start here: [README.md](./README.md)
