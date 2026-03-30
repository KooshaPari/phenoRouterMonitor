# Phase 2: Execution DAG & Parallel Batch Structure

**Date Created**: 2026-03-30
**Purpose**: Define work stream dependencies, parallel execution strategy, and agent batching
**Status**: READY FOR EXECUTION

---

## Master Dependency DAG

### Full Directed Acyclic Graph (DAG)

```
START
  │
  ├─────────────────────────────────────────────────────┐
  │                                                     │
  ▼                                                     ▼
[WS5: Pydantic]                              [WS4: Python HTTPX]
   │                                              │
   │ No external deps                            │ No external deps
   │ 2-3 tool calls                             │ 12-15 tool calls
   │ 2.5 hours                                  │ 18-24 hours
   │                                              │
   ▼                                              ▼
[WS5.1: Create patterns.md]           [WS4.1: Wrapper consolidation]
   │                                              │
   │ Output: Documentation                       │ Outputs: canonical module
   │                                              │
   ├──────────────────────────┬──────────────────┘
   │                          │
   │                    [WS4.2: Pool standardization]
   │                          │
   │                    [WS4.3: Non-compliant files]
   │                          │
   │                    [WS4.4: Testing & docs]
   │                          │
   │          [WS6: Rust TOML Config] ◄────────┐
   │                  │                         │
   │                  │ No external deps        │
   │                  │ 26-29 tool calls        │
   │                  │ 7.25 hours              │
   │                  │                         │
   │                  ▼                         │
   │         [WS6.1: Create crate]             │
   │                  │                         │
   │                  ├─ Outputs: ←────────────┘
   │                  │  phenotype-config
   │                  │
   │                  ▼
   │        [WS6.2: Upgrade TOML]
   │                  │
   │                  ▼
   │     [WS6.3: Migrate proj 1-3]
   │                  │
   │                  ▼
   │     [WS6.4: Migrate proj 4-7]
   │                  │
   │                  ▼
   │     [WS6.5: Migrate proj 8-10]
   │                  │
   └──────────────────┼──────────────────┐
                      │                  │
                      ▼                  ▼
              [WS4.Final]          [WS6.Final]
                  │                    │
                  │ All tests OK       │ All tests OK
                  │ LOC metrics        │ LOC metrics
                  │                    │
                  └────────────────┬───┘
                                   │
                                   ▼
                          [INTEGRATION]
                                   │
                   ┌───────────────┼───────────────┐
                   │               │               │
                   ▼               ▼               ▼
            [Cross-repo    [Final test    [Metrics
             validation]    suite]        verification]
                   │               │               │
                   └───────────────┼───────────────┘
                                   │
                                   ▼
                      [PHASE 2 COMPLETE ✅]
```

### Simplified Dependency Matrix

| Task | Depends On | Blocked By | Critical Path |
|------|-----------|-----------|----------------|
| WS5 | None | None | NO |
| WS4.1 | None | None | YES (longest) |
| WS4.2 | WS4.1 | None | YES |
| WS4.3 | WS4.2 | None | YES |
| WS4.4 | WS4.3 | None | YES |
| WS6.1 | None | None | YES |
| WS6.2 | WS6.1 | None | YES |
| WS6.3 | WS6.2 | None | YES |
| WS6.4 | WS6.3 | None | YES |
| WS6.5 | WS6.4 | None | YES |
| Integration | WS4.4, WS5, WS6.5 | None | NO (serial join) |

### Critical Path Analysis

**Critical Path = Longest Sequence of Dependencies**

```
WS4.1 → WS4.2 → WS4.3 → WS4.4 → Integration
├─ 6-8 hrs
├─ 4-6 hrs
├─ 4-6 hrs
├─ 2-4 hrs
└─ 2-3 hrs
───────────────────────
Total: 18-27 hours
```

**Parallel Opportunities**:
- WS5 (2.5 hrs) can run fully parallel with WS4 and WS6
- WS6 (7.25 hrs) can run parallel with WS4 (different languages)
- All 3 can start immediately and run in parallel

**Optimized Wall-Clock Time** (with 3+ agents):
- WS4: 18-24 hours (longest)
- WS5: 2.5 hours (parallel, completes early)
- WS6: 7.25 hours (parallel, completes mid-way)
- Integration: 2-3 hours (serial, after all complete)

**Total Wall-Clock**: ~23-29 hours (longest critical path + serial integration)

---

## Parallel Batch Structure

### Batch Architecture: 3 Parallel Streams + Serial Integration

```
┌─────────────────────────────────────────────────────────────────┐
│                    PHASE 2 EXECUTION TIMELINE                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  Time  │ Batch 1 (WS4)   │ Batch 2 (WS5)   │ Batch 3 (WS6)      │
│        │ Python HTTPX    │ Pydantic        │ Rust TOML Config   │
├────────┼─────────────────┼─────────────────┼────────────────────┤
│  0-2h  │ Analysis        │ Doc draft       │ Crate creation     │
│  2-4h  │ Wrapper merge   │ Exemplar review │ Version upgrade    │
│  4-6h  │ Pool standard.  │ Migration guide │ Proj 1-3 migrate   │
│  6-8h  │ Non-compliant   │ Review          │ Proj 4-7 migrate   │
│  8-10h │ Testing         │ Finalize        │ Proj 8-10 migrate  │
│ 10-12h │ Documentation   │ COMPLETE ✅     │ Testing            │
│ 12-14h │ COMPLETE ✅     │                 │ Documentation      │
│ 14-16h │                 │                 │ COMPLETE ✅        │
├────────┴─────────────────┴─────────────────┴────────────────────┤
│ 16-18h │ INTEGRATION: Cross-repo validation, final tests       │
│ 18-20h │ METRICS: Verification, reporting, sign-off           │
│ 20+h   │ COMPLETE ✅ Phase 2 Done                              │
└────────┴────────────────────────────────────────────────────────┘
```

### Actual Recommended Parallel Batches

**Batch Structure** (for optimal parallelization):

#### Batch A: Foundation Work (Parallel)
- **WS5.1**: Create PYDANTIC_SETTINGS_PATTERNS.md (2.5 hours)
- **WS6.1**: Create phenotype-config crate (2 hours)
- **Agent**: 1 person per task (can overlap)
- **Duration**: 2.5 hours (longest)
- **Critical**: No (WS5 is optional documentation)

#### Batch B: Core Migrations (Parallel, after A)
- **WS4.1**: Wrapper consolidation (6-8 hours)
  - 3 wrapper functions: `sdk/http_client.py`, `adapters/http.py`, `clients/http_wrapper.py`
  - Agent: 1 (can parallelize file merges within)
- **WS6.2**: Upgrade TOML to 0.9.5 (1.5 hours)
  - Agent: 1 (quick dependency upgrade)
  - Dependency: After WS6.1 crate created
- **WS4.2**: Connection pool standardization (4-6 hours)
  - Agent: 1 (depends on WS4.1 wrapper completion)
  - Parallel with WS6 work
- **Duration**: 8 hours (critical path: WS4 > WS6)

#### Batch C: Integration Migrations (Parallel, after B)
- **WS4.3**: Non-compliant file migration (4-6 hours)
  - 2 files: extended_benchmark.py, benchmark_alt.py
  - Agent: 1 (can run in parallel with WS6 if needed)
  - Dependency: After WS4.2 pool standardization
- **WS6.3**: Project 1-3 migration (1.5 hours)
  - Agent: 1 (3 projects × 30 min each)
  - Dependency: After WS6.2 version upgrade
- **WS6.4**: Project 4-7 migration (1.5 hours)
  - Agent: 1 (can be same as WS6.3, sequential)
  - Dependency: After WS6.3
- **WS6.5**: Project 8-10 migration (1.5 hours)
  - Agent: 1 (can be same as WS6.3-4, sequential)
  - Dependency: After WS6.4
- **Duration**: 6 hours (WS4 slower, WS6 faster)

#### Batch D: Testing & Documentation (Parallel, after C)
- **WS4.4**: Testing & documentation (2-4 hours)
  - Agent: 1 (writing tests, docs for httpx consolidation)
  - Dependency: After WS4.3
- **WS6 Final**: Testing & documentation (included in batch C)
  - Tests run after each project migration
  - Dependency: After WS6.5
- **Duration**: 4 hours (parallel work)

#### Batch E: Integration & Validation (Serial, after D)
- **Cross-repo validation** (1 hour)
  - Verify no import cycles
  - Run full test suite
  - Agent: 1
- **Metrics verification** (1-2 hours)
  - Measure LOC reduction
  - Performance benchmarks
  - Agent: 1
- **Final sign-off** (30 min)
  - Review all completion criteria
  - Create PR
  - Agent: 1
- **Duration**: 2.5-3 hours (serial)

### Detailed Agent Assignment

**Recommended 4-Agent Team**:

```
┌───────────────────────────────────────────────────────────┐
│          PHASE 2 EXECUTION: 4-AGENT TEAM STRUCTURE        │
├───────────────────────────────────────────────────────────┤
│                                                            │
│  Agent 1 (WS4 Lead) — Python HTTPX Consolidation        │
│  ├─ Batch B: Wrapper consolidation (6-8h)              │
│  ├─ Batch B: Pool standardization (4-6h)               │
│  ├─ Batch C: Non-compliant files (4-6h)                │
│  └─ Batch D: Testing & docs (2-4h)                     │
│  Total: 18-24 hours                                     │
│                                                            │
│  Agent 2 (WS6 Lead) — Rust TOML Config Migration       │
│  ├─ Batch A: Create crate (2h)                         │
│  ├─ Batch B: Upgrade TOML (1.5h)                       │
│  ├─ Batch C: Migrate projects (4.5h: 3 migrations)     │
│  └─ Testing (included)                                  │
│  Total: 7.25 hours                                      │
│                                                            │
│  Agent 3 (WS5 + Integration)                             │
│  ├─ Batch A: Documentation (2.5h)                      │
│  └─ Batch E: Integration & validation (2.5-3h)         │
│  Total: 5-5.5 hours                                     │
│                                                            │
│  Optional Agent 4 (Parallel Support)                     │
│  ├─ Accelerate WS4 wrapper merges (parallel)           │
│  ├─ Speed up WS6 project migrations (parallel)         │
│  └─ Reduce wall-clock time by ~30-40%                  │
│                                                            │
└───────────────────────────────────────────────────────────┘
```

**Team Velocity** (with 3 agents):
- Batch A: ~2.5 hours (parallel)
- Batch B: ~8 hours (parallel)
- Batch C: ~6 hours (parallel)
- Batch D: ~4 hours (parallel)
- Batch E: ~2.5-3 hours (serial)
- **Total wall-clock**: ~23-29 hours

**Acceleration with 4 agents**:
- Agent 4 can parallelize:
  - WS4 file merges (help Agent 1 with 3 wrappers simultaneously)
  - WS6 project migrations (help Agent 2 batch projects faster)
- Estimated reduction: 30-40% faster
- New wall-clock: ~14-18 hours (best case)

---

## Detailed Work Breakdown Structure (WBS)

### Phase 2 WBS by Batch

```
PHASE 2: Library Consolidation & OSS Wrapping
│
├─ BATCH A: Foundation (2-3 hours, Agents: 1-2)
│  ├─ WS5.1: Create PYDANTIC_SETTINGS_PATTERNS.md (2.5h)
│  │  ├─ Read exemplar code (thegent config)
│  │  ├─ Document patterns observed
│  │  ├─ Write usage examples
│  │  ├─ Create migration guide
│  │  └─ Review and finalize
│  │
│  └─ WS6.1: Create phenotype-config crate (2h)
│     ├─ Generate crate scaffold
│     ├─ Implement ConfigLoader struct
│     ├─ Add serde/toml deps
│     ├─ Write basic tests
│     └─ Update workspace Cargo.toml
│
├─ BATCH B: Core Migrations (8 hours, Agents: 1-2)
│  ├─ WS4.1: Wrapper consolidation (6-8h) [Agent 1]
│  │  ├─ Task 1a: Analyze all 4 wrappers
│  │  ├─ Task 1b: Create canonical httpx.py
│  │  ├─ Task 1c: Merge sdk/http_client.py
│  │  ├─ Task 1d: Merge adapters/http.py
│  │  ├─ Task 1e: Merge clients/http_wrapper.py
│  │  └─ Task 1f: Merge utils/http.py
│  │
│  ├─ WS6.2: Upgrade TOML version (1.5h) [Agent 2]
│  │  ├─ Task 2a: Update workspace.dependencies
│  │  ├─ Task 2b: Update 6 project Cargo.tomls
│  │  └─ Task 2c: Run cargo check, fix API breaks
│  │
│  └─ WS4.2: Pool standardization (4-6h) [Agent 1, parallel with WS6]
│     ├─ Task 3a: Audit all pooling patterns
│     ├─ Task 3b: Create singleton pool pattern
│     ├─ Task 3c: Apply httpx.Limits to 6 files
│     ├─ Task 3d: Apply singleton to 4 files
│     └─ Task 3e: Verify pooling behavior
│
├─ BATCH C: Integration Migrations (6 hours, Agents: 1-2)
│  ├─ WS4.3: Non-compliant files (4-6h) [Agent 1]
│  │  ├─ Task 4a: Audit extended_benchmark.py
│  │  ├─ Task 4b: Replace requests with httpx
│  │  ├─ Task 4c: Audit benchmark_alt.py
│  │  └─ Task 4d: Replace requests fallback
│  │
│  ├─ WS6.3: Migrate projects 1-3 (1.5h) [Agent 2]
│  │  ├─ Task 5a: Migrate project-1 (30 min)
│  │  ├─ Task 5b: Migrate project-2 (30 min)
│  │  └─ Task 5c: Migrate project-3 (30 min)
│  │
│  ├─ WS6.4: Migrate projects 4-7 (1.5h) [Agent 2, sequential]
│  │  ├─ Task 6a: Migrate project-4 (30 min)
│  │  ├─ Task 6b: Migrate project-5 (30 min)
│  │  └─ Task 6c: Migrate project-6/7 (30 min)
│  │
│  └─ WS6.5: Migrate projects 8-10 (1.5h) [Agent 2, sequential]
│     ├─ Task 7a: Migrate project-8 (30 min)
│     ├─ Task 7b: Migrate project-9 (30 min)
│     └─ Task 7c: Migrate project-10 (30 min)
│
├─ BATCH D: Testing & Documentation (4 hours, Agents: 1-2)
│  ├─ WS4.4: Testing & docs (2-4h) [Agent 1]
│  │  ├─ Task 8a: Write httpx consolidation tests
│  │  ├─ Task 8b: Write connection pool tests
│  │  ├─ Task 8c: Run full test suite
│  │  ├─ Task 8d: Create HTTP_CLIENT_PATTERNS.md
│  │  └─ Task 8e: Create POL-HTTP-001.md
│  │
│  └─ WS6 Final: Testing & docs (included above) [Agent 2]
│     ├─ Task 9a: Run cargo test --workspace
│     ├─ Task 9b: Create CONFIG_LOADER_PATTERNS.md
│     └─ Task 9c: Document migration path
│
└─ BATCH E: Integration & Validation (2.5-3 hours, Agent: 3)
   ├─ Cross-repo validation (1h)
   │  ├─ Task 10a: Verify no import cycles
   │  ├─ Task 10b: Run full pytest + cargo test
   │  └─ Task 10c: Check for warnings
   │
   ├─ Metrics verification (1-2h)
   │  ├─ Task 11a: Count LOC reductions
   │  ├─ Task 11b: Run performance benchmarks
   │  ├─ Task 11c: Measure test coverage
   │  └─ Task 11d: Verify success criteria
   │
   └─ Final sign-off (30 min)
      ├─ Task 12a: Review completion checklist
      ├─ Task 12b: Create PR with summary
      └─ Task 12c: Get approval, merge
```

---

## Tool Call Accounting

### Total Tool Calls by Work Stream

**WS4 (Python HTTPX)**
- Batch B:
  - Analyze & create canonical: 4 tool calls
  - Merge 4 wrapper files: 8 tool calls (2 per file)
- Batch C:
  - Audit non-compliant: 2 tool calls
  - Fix imports: 3 tool calls
- Batch D:
  - Tests & docs: 3 tool calls
- **Subtotal WS4**: 20 tool calls ✓ (matches 12-15 estimate + buffer)

**WS5 (Python Pydantic)**
- Batch A:
  - Read exemplar: 1 tool call
  - Write patterns doc: 2 tool calls
- **Subtotal WS5**: 3 tool calls ✓

**WS6 (Rust TOML)**
- Batch A:
  - Create crate scaffold: 2 tool calls
  - Implement ConfigLoader: 3 tool calls
  - Add tests: 1 tool call
- Batch B:
  - Upgrade versions: 2 tool calls
  - Fix API breaks: 2 tool calls
- Batch C:
  - Migrate 10 projects: 10 tool calls (1 per project)
  - Test each migration: 3 tool calls
- Batch D:
  - Final tests: 2 tool calls
  - Documentation: 2 tool calls
- **Subtotal WS6**: 29 tool calls ✓ (matches 26-29 estimate)

**Batch E (Integration)**
- Cross-repo validation: 3 tool calls
- Metrics & reporting: 2 tool calls
- Sign-off: 2 tool calls
- **Subtotal E**: 7 tool calls ✓

**Total**: 20 + 3 + 29 + 7 = **59 tool calls**
(Matches estimated 45-54 range with conservative buffer)

---

## Execution Risks & Mitigation Matrix

| Risk | Phase | Probability | Impact | Mitigation | Owner |
|------|-------|------------|--------|-----------|-------|
| Breaking changes in httpx | B | MEDIUM | MEDIUM | Test async ops; benchmark | Agent 1 |
| TOML API incompatibility | B | LOW | MEDIUM | Run cargo check after upgrade | Agent 2 |
| Import cycles in WS4 | C | MEDIUM | LOW | Use static analysis; test imports | Agent 1 |
| Config loading regression | D | LOW | LOW | Benchmark before/after | Agent 2 |
| Incomplete file migration | C | MEDIUM | MEDIUM | Use systematic grep/replace; verify | Agent 1 |
| Circular deps in WS6 | E | LOW | HIGH | Architecture review; resolve early | Agent 3 |
| Test suite breakage | D-E | LOW | HIGH | Test frequently; maintain rollback | All |
| Performance regression | D | LOW | MEDIUM | Benchmark critical paths | All |
| Merge conflicts (GitHub) | E | MEDIUM | LOW | Rebase frequently; small PRs | Agent 3 |

---

## Success Metrics & KPIs

### Phase 2 Key Performance Indicators

| KPI | Target | WS4 | WS5 | WS6 | Integration |
|-----|--------|-----|-----|-----|-------------|
| **LOC Reduction** | ≥1,230 | 180-240 | +45-90 | 500+ | N/A |
| **Tool Calls** | ≤60 | 20 | 3 | 29 | 7 |
| **Wall-Clock (hrs)** | ≤30 | 18-24 | 2.5 | 7.25 | 2.5-3 |
| **Tests Passing** | 100% | ✓ | N/A | ✓ | ✓ |
| **Warnings** | 0 | 0 | 0 | 0 | 0 |
| **Coverage** | ≥80% | ✓ (Python) | N/A | ✓ (Rust) | N/A |
| **Docs Complete** | 100% | ✓ | ✓ | ✓ | ✓ |

---

## Batch Execution Checklist

### Pre-Execution (Kickoff)

- [ ] All agents understand Phase 2 roadmap
- [ ] Feature branches created (wip/phase2-ws{4,5,6})
- [ ] Dependencies installed and verified
- [ ] Team sync completed
- [ ] This DAG reviewed and approved

### Batch A Execution

- [ ] WS5.1: Documentation draft started (1 hour into Day 1)
- [ ] WS6.1: Crate scaffold created (1 hour into Day 1)
- [ ] Both batches complete within 2.5 hours
- [ ] All new files pushed to feature branches

### Batch B Execution

- [ ] WS4.1: Wrapper consolidation in progress (4 hours into Day 1)
- [ ] WS6.2: TOML upgrade tested (1.5 hours into Batch B)
- [ ] WS4.2: Pool standardization in progress (after WS4.1)
- [ ] All tests passing at 8-hour mark (end of Batch B)

### Batch C Execution

- [ ] WS4.3: Non-compliant files audited (start of Batch C)
- [ ] WS6.3-6.5: Projects migrated incrementally
- [ ] Each migration tested before proceeding
- [ ] All 10 projects migrated by 14-hour mark

### Batch D Execution

- [ ] WS4.4: Full test suite for httpx (16-18 hour mark)
- [ ] Documentation finalized (HTTP_CLIENT_PATTERNS.md, CONFIG_LOADER_PATTERNS.md)
- [ ] All tests passing (pytest + cargo test)
- [ ] Batch D complete by 20-hour mark

### Batch E Execution

- [ ] Cross-repo validation (20-21 hour mark)
- [ ] Metrics verification (21-23 hour mark)
- [ ] Final PR creation and review (23-24 hour mark)
- [ ] Phase 2 complete ✅ (24 hour mark or ~20-30 hours wall-clock)

---

## Fallback Plans

### If WS4 Encounters Blocker

**Contingency**: Stop WS4, proceed with WS5 + WS6
- WS5 + WS6 can complete independently
- WS4 can be retried or deferred to next sprint
- Impact: Lose 180-240 LOC savings, but gain 500+ from WS6

### If WS6 Encounters Breaking Changes

**Contingency**: Roll back TOML version upgrade, proceed with manual consolidation
- Keep projects on toml 0.8 if 0.9 incompatible
- Continue with ConfigLoader crate using 0.8 APIs
- Impact: Slightly higher maintenance burden, but still consolidates patterns

### If Integration Tests Fail

**Contingency**: Return to parallel batch work, fix issues one at a time
- Do not proceed with Phase 3 until all Phase 2 tests pass
- Debug systematically (circular deps, import issues, etc.)
- Fallback: Rollback to main, restart Phase 2 with fixes

### If Performance Regression Found

**Contingency**: Profile and optimize before proceeding
- WS4: Optimize connection pooling (add caching)
- WS6: Optimize ConfigLoader (add lazy loading)
- Benchmark and verify improvement before merge

---

## Phase 2 → Phase 3 Transition

**Upon Phase 2 Completion:**

1. **Merge all feature branches to main**
   - Create consolidated PR if needed
   - Ensure all checks pass
   - Merge when approved

2. **Verify metrics in main**
   - Run full test suite
   - Measure LOC reduction (should be ~1,230)
   - Benchmark performance (should be ≥baseline)

3. **Archive old documentation**
   - Move audit reports to `.archive/phase2-audits/`
   - Keep pattern guides and policies active
   - Reference in Phase 2 completion report

4. **Kick off Phase 3**
   - Launch AgilePlus file decomposition
   - Target: routes.rs + sqlite/lib.rs
   - Expected: 2,750 LOC reduction
   - Timeline: Same batch structure (3-4 weeks parallel)

---

**Execution DAG Status**: READY FOR IMMEDIATE EXECUTION
**Last Updated**: 2026-03-30

