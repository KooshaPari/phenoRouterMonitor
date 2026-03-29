# FORGE Research: Comprehensive Audit Findings

> **Agent:** FORGE
> **Date:** 2026-03-29
> **Session:** Wave 92 - Comprehensive Repository Audit
> **Priority:** P0-P1

---

## Executive Summary

Conducted deep research across three parallel tracks using subagents:
1. **Non-Canonical Folder Audit** - Identified 7 folders requiring action
2. **3rd Party Package Analysis** - Analyzed blackbox/graybox/whitebox usage
3. **Repo-Level Duplication** - Found 622 LOC of critical duplication

**Estimated Total LOC Impact:** ~1,400+ lines across cleanup and consolidation

---

## 1. Non-Canonical Folders Audit

### Findings Table

| Folder | Type | Content | Recommendation | Priority |
|--------|------|---------|----------------|----------|
| `.worktrees/phench-fix` | Orphaned worktree | phenotype-infrakit Rust workspace | **ARCHIVE** | HIGH |
| `.worktrees/gh-pages-deploy` | Orphaned worktree | Documentation deployment | **ARCHIVE** | HIGH |
| `worktrees/` | Empty | None | **DELETE** | HIGH |
| `platforms/thegent` | External clone | Full Python/Rust project (~3.9M lines) | **EVALUATE** | MEDIUM |
| `add/` | Empty | None | **DELETE** | HIGH |
| `worktree/` | Empty | None | **DELETE** | HIGH |
| `src/thegent/` | Partial copy | ~76K lines (subset of platforms/thegent) | **INVESTIGATE** | MEDIUM |
| `crates/` | Orphan project | phenotype-event-sourcing workspace | **ARCHIVE** | HIGH |
| `docs/node_modules/` | Generated | ~420K lines npm packages | **DELETE** | HIGH |
| `docs/reports/` | Artifacts | Audit reports | **KEEP** | - |

### Cleanup Commands

```bash
# Delete empty placeholders
rmdir worktrees/ worktree/ add/

# Archive orphaned worktrees (move to archive location)
mv .worktrees/phench-fix ~/Archives/phench-fix-20260329
mv .worktrees/gh-pages-deploy ~/Archives/gh-pages-deploy-20260329

# Remove node_modules (regeneratable)
rm -rf docs/node_modules/
```

### Action Items

- [ ] CLEAN-001: Delete `worktrees/`, `worktree/`, `add/` (empty placeholders)
- [ ] CLEAN-002: Archive `.worktrees/phench-fix/` to ~/Archives/
- [ ] CLEAN-003: Archive `.worktrees/gh-pages-deploy/` to ~/Archives/
- [ ] CLEAN-004: Delete `docs/node_modules/` (regeneratable)
- [ ] CLEAN-005: Investigate `src/thegent/` vs `platforms/thegent/` duplication
- [ ] CLEAN-006: Archive `crates/phenotype-event-sourcing/` (orphan content)
- [ ] CLEAN-007: Document purpose of `platforms/thegent/`

---

## 2. 3rd Party Package Analysis

### Usage Categories

#### BLACKBOX (Direct Dependencies - No Modification)

| Package | Version | Purpose | Status |
|---------|---------|---------|--------|
| `serde` | 1.0 | Serialization | ✅ Good |
| `serde_json` | 1.0 | JSON parsing | ✅ Good |
| `thiserror` | 2.0 | Error handling | ✅ Good |
| `chrono` | 0.4 | DateTime | ✅ Good |
| `sha2` | 0.10 | SHA-256 hashing | ✅ Good |
| `hex` | 0.4 | Hex encoding | ✅ Good |
| `dashmap` | 6.1 | Concurrent HashMap | ✅ Good |
| `orjson` | - | Fast JSON (Python) | ✅ Good |
| `watchfiles` | - | FS watching (Python) | ✅ Good |
| `rich` | - | Terminal formatting (Python) | ✅ Good |

#### GRAYBOX (Wrappers/Adapters)

| Wrapper | Location | Purpose | Status |
|---------|----------|---------|--------|
| `fast_toml_parser.py` | `src/thegent/infra/` | Auto-select TOML backend (rtoml/tomli/tomlkit) | 🟡 Could be Rust |
| `fast_yaml_parser.py` | `src/thegent/infra/` | Auto-select YAML backend (oyaml/ruamel/PyYAML) | 🟡 Could be Rust |
| `shim_subprocess.py` | `src/thegent/infra/` | Fallback to Rust shims | 🟡 Cross-language |

#### WHITEBOX (Forked/Modified) - NONE FOUND

No forked or patched external repositories identified.

### Unused Dependencies (Declared but Not Imported)

| Package | Declared | Usage | Recommendation |
|---------|----------|-------|----------------|
| `lru` | 0.12 | Not imported | REMOVE or implement LRU cache |
| `parking_lot` | 0.12 | Not imported | REMOVE or add sync utilities |
| `moka` | 0.12 | Not imported | REMOVE or implement async cache |

### Fork/Wrap Opportunities

| Opportunity | Package | Current | Effort | Priority |
|-------------|---------|---------|--------|----------|
| DashMap Wrapper | `dashmap` | Direct use in policy engine | Low (1-2 days) | MEDIUM |
| Regex Compilation Cache | `regex` | `Regex::new()` per evaluation | Low (1 day) | HIGH |
| Unified Config Parser | `fast_toml_parser` + `fast_yaml_parser` | Python wrappers | Medium (3-5 days) | MEDIUM |
| Async Cache Adapter | `moka` | Unused | Medium (2-3 days) | MEDIUM |

### Action Items

- [ ] PKG-001: Remove unused `lru`, `parking_lot`, `moka` from workspace
- [ ] PKG-002: Add Lazy<Regex> to Rule struct for caching
- [ ] PKG-003: Implement PolicyRegistry wrapper with metrics/TTL
- [ ] PKG-004: Extract config parsers to `phenotype-config-parser` crate
- [ ] PKG-005: Implement `phenotype-cache-adapter` using moka

---

## 3. Repo-Level Duplication Analysis

### CRITICAL: Complete File Duplication in phenotype-event-sourcing

Identical source files exist in two locations:

| File | Location A | Location B | Status |
|------|------------|------------|--------|
| `error.rs` | `src/` | `phenotype-event-sourcing/src/` | **IDENTICAL** |
| `hash.rs` | `src/` | `phenotype-event-sourcing/src/` | **IDENTICAL** |
| `event.rs` | `src/` | `phenotype-event-sourcing/src/` | **IDENTICAL** |
| `snapshot.rs` | `src/` | `phenotype-event-sourcing/src/` | **IDENTICAL** |
| `store.rs` | `src/` | `phenotype-event-sourcing/src/` | **SIMILAR** |
| `memory.rs` | `src/` | `phenotype-event-sourcing/src/` | **SIMILAR** |

**Impact:** ~622 lines of duplicated Rust code

**Root Cause:** Nested package structure confusion

### Error Type Duplication

| Error Type | Location | Lines | Status |
|-----------|----------|-------|--------|
| `EventSourcingError` | `phenotype-event-sourcing/src/error.rs` | ~46 | 🔴 DUPLICATED |
| `PolicyEngineError` | `phenotype-policy-engine/.../error.rs` | ~65 | 🟡 ISOLATED |
| `ports::Error` | `phenotype-contracts/.../outbound/mod.rs` | ~20 | 🟢 CONSOLIDATED |
| `inbound::Error` | `phenotype-contracts/.../inbound/mod.rs` | ~20 | 🟢 CONSOLIDATED |

### Empty Placeholder Crates

| Crate | Status | Lines | Action |
|-------|--------|-------|--------|
| `phenotype-cache-adapter` | EMPTY lib.rs | 1 | IMPLEMENT or DELETE |
| `phenotype-state-machine` | EMPTY lib.rs | 1 | IMPLEMENT or DELETE |

### LOC Impact Summary

| Category | Lines | Action |
|----------|-------|--------|
| phenotype-event-sourcing duplication | ~622 | SELECT CANONICAL, DELETE OTHER |
| Empty placeholders | 2 | IMPLEMENT or DELETE |
| Error type fragmentation | ~150 | CREATE error-core |
| **Total Impact** | ~774 | |

### Action Items

- [ ] DUP-001: Choose canonical location for phenotype-event-sourcing
- [ ] DUP-002: Remove duplicate files from non-canonical location
- [ ] DUP-003: Create `phenotype-error-core` crate (~150 LOC savings)
- [ ] DUP-004: Implement or delete `phenotype-cache-adapter`
- [ ] DUP-005: Implement or delete `phenotype-state-machine`

---

## Consolidated Action Items

### HIGH Priority (This Week)

| ID | Action | Category | Effort |
|----|--------|----------|--------|
| CLEAN-001 | Delete `worktrees/`, `worktree/`, `add/` | CLEANUP | Low |
| CLEAN-004 | Delete `docs/node_modules/` | CLEANUP | Low |
| CLEAN-002 | Archive `.worktrees/phench-fix/` | CLEANUP | Medium |
| CLEAN-006 | Archive `crates/phenotype-event-sourcing/` | CLEANUP | Medium |
| DUP-001 | Resolve phenotype-event-sourcing duplication | DUP | Low |
| DUP-002 | Remove duplicate files | DUP | Low |
| PKG-001 | Remove unused dependencies | PACKAGE | Low |

### MEDIUM Priority (This Month)

| ID | Action | Category | Effort |
|----|--------|----------|--------|
| DUP-003 | Create `phenotype-error-core` | DUP | Medium |
| PKG-002 | Cache regex compilations | PACKAGE | Low |
| PKG-003 | Implement PolicyRegistry wrapper | PACKAGE | Medium |
| CLEAN-005 | Investigate thegent duplication | CLEANUP | High |
| CLEAN-007 | Document platforms/thegent purpose | CLEANUP | Medium |

### LOW Priority (Future)

| ID | Action | Category | Effort |
|----|--------|----------|--------|
| DUP-004 | Implement phenotype-cache-adapter | DUP | Medium |
| DUP-005 | Implement phenotype-state-machine | DUP | Medium |
| PKG-004 | Extract config parsers to crate | PACKAGE | Medium |
| PKG-005 | Implement async cache adapter | PACKAGE | Medium |

---

## Files Modified/Created

| File | Action | Purpose |
|------|--------|---------|
| `docs/worklogs/WORK_LOG.md` | Updated | Wave 92 entry with all findings |
| `docs/worklogs/README.md` | Updated | Added Wave 92 summary |

---

## Related Documentation

| Document | Purpose |
|----------|---------|
| `docs/worklogs/DUPLICATION.md` | Extended duplication findings |
| `docs/worklogs/DEPENDENCIES.md` | Dependency analysis |
| `docs/worklogs/RESEARCH.md` | Tech radar |
| `docs/worklogs/ARCHITECTURE.md` | Port/trait analysis |

---

## Wave 93: LOC Reduction & External Package Deep Dive

**Date:** 2026-03-29
**Priority:** P0

### Summary

Expanded worklog audit with comprehensive LOC reduction analysis, external package fork/wrap strategies, and cross-repo duplication patterns.

### Key Accomplishments

1. **Created `LOC_REDUCTION.md`** (336 lines)
   - Comprehensive LOC reduction matrix (~3,190 LOC savings potential)
   - Phase 1-3 implementation plan
   - Dead code elimination targets
   - Boilerplate reduction opportunities
   - Test reduction strategies

2. **Enhanced `RESEARCH.md`** (219 new lines)
   - External package fork/wrap opportunities (casbin, cqrs-es, temporal-sdk, figment)
   - Package health indicators for 2026
   - Whitebox vs Blackbox strategy matrix
   - Implementation priority schedule (Week 1-4)

3. **Updated `README.md`**
   - Added LOC_REDUCTION.md entry
   - Added category summary for LOC_REDUCTION
   - Updated line counts for all files

### External Package Integration Matrix

| Package | Strategy | LOC Savings | Priority | Implementation |
|---------|----------|-------------|----------|----------------|
| `casbin-rs` | WRAP | 2-3k LOC | P0 | `agileplus-policy` |
| `cqrs-es` | WRAP | 3k LOC | P0 | Replace `eventually-rs` |
| `figment` | ADOPT | 500 LOC | P0 | Replace `config-rs` |
| `health-check` | FORK | 140 LOC | P1 | `agileplus-health` |
| `statig` | ADOPT | 300 LOC | P1 | Replace custom SM |
| `temporal-sdk` | WRAP | 4k LOC | P1 | Workflow engine |

### Cross-Repo Duplication Summary

| Pattern | Repos | LOC | Canonical |
|---------|-------|-----|-----------|
| phenotype-event-sourcing | x2 | ~1,400 | phenotype-infrakit |
| Error types | `agileplus-*` x8 | ~600 | `agileplus-error-core` |
| Health checks | `agileplus-*` x3 | ~80 | `agileplus-health` |
| Config loading | `agileplus-*` x4 | ~500 | `libs/config-core` |

### LOC Savings by Phase

| Phase | Focus | Savings | Priority |
|-------|-------|---------|----------|
| Phase 1 | Quick wins (derive macros, remove duplicates) | ~2,000 LOC | P0 |
| Phase 2 | Medium refactors (error-core, health-core) | ~800 LOC | P1 |
| Phase 3 | Major refactors (port traits, external adoption) | ~1,500 LOC | P2 |

### Files Created/Modified

| File | Action | Lines Added |
|------|--------|-------------|
| `LOC_REDUCTION.md` | Created | +336 |
| `RESEARCH.md` | Enhanced | +219 |
| `README.md` | Updated | +20 |

### Next Steps

- [ ] Evaluate casbin-rs for policy enforcement
- [ ] Create `agileplus-error-core` crate
- [ ] Integrate figment for config loading
- [ ] Fork health-check for unified health status
- [ ] Remove nested duplicate crates (Phase 1)

---

_Last updated: 2026-03-29 (Wave 93)_
