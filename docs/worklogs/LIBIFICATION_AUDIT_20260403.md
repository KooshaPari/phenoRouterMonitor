# Libification & Modernization Audit - 2026-04-03

**Project:** [heliosApp], [portage], [heliosCLI]
**Category:** libification, modernization
**Status:** completed
**Priority:** P1

## Summary

Comprehensive audit of heliosApp, portage, and heliosCLI projects to identify:
- Library extraction opportunities
- 2026 modern alternatives for existing utilities
- Active worktree status and cleanup needs
- Decomposition opportunities

---

## Key Discovery: SolidJS (NOT React)

**Critical finding:** heliosApp uses **SolidJS**, NOT React. Many DUPLICATION_AUDIT items reference "React" patterns but heliosApp needs "Solid" equivalents.

| React Pattern | SolidJS Equivalent | Status |
|---------------|-------------------|--------|
| React Query | TanStack Solid Query | Not installed |
| Zustand | Solid's createStore | Already using |
| React Router | @solidjs/router | Not installed |

---

## heliosApp Findings

### Current Tech Stack

| Category | Current Version | Assessment |
|----------|-----------------|------------|
| **Runtime** | Bun 1.2.20 | ✅ Up-to-date |
| **UI Framework** | SolidJS 1.9.12 | ✅ Current |
| **TypeScript** | 5.8.2 | ⚠️ TS7 available - upgrade needed |
| **Testing** | Vitest 4.1.2 + Playwright 1.58.2 | ✅ Current |
| **Linting** | Biome 2.4.10 + oxlint 1.57.0 | ✅ Current |
| **HTTP Client** | ky 1.14.3 | ✅ Modern |

### Workspace Structure

```
heliosApp/
├── apps/
│   ├── runtime/       # Main runtime application
│   ├── desktop/       # Desktop application  
│   └── colab-renderer/ # Colab renderer
├── packages/
│   ├── errors/        # ✅ Already extracted
│   ├── ids/           # ID utilities
│   ├── logger/        # Structured logging
│   ├── runtime-core/  # Core runtime functionality
│   └── types/         # Shared TypeScript types
```

### Library Extraction Candidates

| Candidate | Location | Est LOC | 2026 Alternative |
|-----------|----------|---------|------------------|
| **@helios/event-bus** | `apps/runtime/src/protocol/bus.ts` | ~300 | Native SolidJS signals |
| **@helios/pty** | `apps/runtime/src/pty/` | ~1000+ | Core competency |
| **@helios/secrets** | `apps/runtime/src/secrets/` | ~400 | Consider webcrypto |
| **TanStack Solid Query** | Not installed | - | `@tanstack/solid-query` v5 |

---

## portage Findings

### Python Modernization

| Current | Target | Effort |
|---------|--------|--------|
| `requests` | `httpx` | MEDIUM - single file |
| `tenacity` | `stamina` | LOW - drop-in |
| Python 3.12+ | 3.14 | MEDIUM |
| `pydantic` | Add `pydantic-settings` | LOW |

### New Library Extractions

| Library | Scope | LOC Savings |
|---------|-------|-------------|
| `harbor-adapters` | Swappable sandbox abstraction | 400+ |
| `harbor-benchmarks` | Benchmark runner patterns | 300-500 |
| `harbor-config` | Unify config loading | 150-300 |
| `harbor-sandbox` | Docker/e2b/daytona/k8s interfaces | 400+ |

### Viewer Status (Already Modernized)
- TypeScript 5.9.2 ✅ strict enabled
- oxlint present ✅
- PR #250: dual lockfiles → single Bun lockfile ✅

---

## heliosCLI Findings

### Config Pattern Analysis

| Pattern | Location | Complexity |
|---------|----------|------------|
| **Pattern A: Basic** | `crates/phenotype-config-core/` | Low (~275 LOC) |
| **Pattern B: Layered + Edit** | `heliosCLI/codex-rs/config/` | High (~2000+ LOC) |
| **Pattern C: Enum-Based** | `platforms/thegent/` | Minimal |

### Duplication Found
- Error types in both systems
- File loading logic duplicated
- Multi-source merging duplicated

**Recommendation:** Keep systems separate but document in CONFIG_GOVERNANCE.md

---

## Active Worktrees Status

### heliosApp-wtrees
| Branch | Status | Files Changed |
|--------|--------|---------------|
| `stabilize` | Uncommitted | 200+ |
| `claude-md-standardize` | Ahead 9, Behind 60 | 80+ |
| `phase2-decompose` | Ahead 7, Behind 642 | Highly diverged |

### heliosCLI-wtrees
| Branch | Status | Issue |
|--------|--------|-------|
| `review-orchestrator` | Ahead 4038 | Massive divergence |
| `release-v0.1.0` | Abandoned | Empty directory |

### portage
| PR | Status |
|----|--------|
| #250 | Viewer Bun cleanup - ready to merge |

---

## WS1-6 Libification Status

| Work Stream | Status | Compliance |
|-------------|--------|------------|
| **WS1** - thiserror | ✅ 93% complete | 13/14 files |
| **WS3** - Zod | ✅ 100% | All compliant |
| **WS4** - httpx | ⚠️ 95% | `extended_benchmark.py` mixed libs |
| **WS5** - Pydantic | ✅ 100% | All compliant |
| **WS6** - TOML | ✅ Complete | Fragmented |

### Critical Issues
1. **`extended_benchmark.py`** - mixes requests + httpx + aiohttp
2. **HTTP wrappers** - 3 duplicates: `FastHTTPClient`, `HTTPClient`, `HTTPConnectionPool`
3. **phenotype-config-core** - underutilized

---

## 2026 Alternatives Summary

| Category | Current | Recommended 2026 |
|----------|---------|------------------|
| State (Solid) | Manual signals | `@tanstack/solid-query` |
| TypeScript | 5.8.2 | **7.x** (now available) |
| HTTP (Py) | requests | httpx |
| Retry (Py) | tenacity | stamina |
| Linting | Biome + oxlint | Keep oxlint (VoidZero) |
| Python | 3.12+ | 3.14 when available |

---
---

## Documentation Duplication Found

**Analysis Date:** 2026-04-03  
**See:** `DOCUMENTATION_DUPLICATION_ANALYSIS.md` for details

### Summary of Doc Overlap

| Topic | Files Affected | LOC Overlap |
|-------|---------------|-------------|
| Error Handling | 6 files | ~1,000 LOC |
| Retry Logic | 5 files | ~400 LOC |
| Config Loading | 6 files | ~2,000 LOC |
| Serialization | 3 files | ~350 LOC |

### Archive Candidates

After consolidating content to this audit, these files can be archived:
- `docs/reports/CROSS_PROJECT_DUPLICATION_ANALYSIS.md`
- `docs/reports/DECOMPOSITION_AUDIT.md`
- `docs/worklogs/DUPLICATION_EXPANSION_20260329.md`
- `docs/worklogs/DUPLICATION_AUDIT_SUMMARY.md`
- `docs/worklogs/EXPANSION_COMPLETION_REPORT.md`

---

## Code Duplication Summary (from Research)

### Error Handling - P0 Priority

| Metric | Current | Target |
|--------|---------|--------|
| Duplicate error enums | 15+ | 1 canonical |
| LOC duplicated | ~850 | ~0 |
| Library status | phenotype-error-core UNUSED | Promote |

### Retry Logic - P0 Priority

| Metric | Current | Target |
|--------|---------|--------|
| Duplicate implementations | 4 | 1 (backoff crate) |
| LOC duplicated | ~186 | ~20 |
| Library status | phenotype-retry UNDERUTILIZED | Adopt |

### Config Loading - P1 Priority

| Metric | Current | Target |
|--------|---------|--------|
| Duplicate implementations | 5-8 | 1 (figment) |
| LOC duplicated | ~650-1200 | ~200 |
| Library status | config-core EDITION MISMATCH | Fix + promote |

### Serialization - P1 Priority

| Metric | Current | Target |
|--------|---------|--------|
| Format | Manual JSON | rkyv + prost |
| Performance | 100ms | 25ms (4x) |
| Zero-copy | No | Yes |

---

## Action Items

- [ ] Consolidate error handling to phenotype-error-core
- [ ] Adopt backoff crate for retry logic
- [ ] Fix config-core edition mismatch + adopt figment
- [ ] Evaluate rkyv for zero-copy serialization
- [ ] Archive duplicate documentation files

### Near-term (This Quarter)
- [ ] Portage: Migrate requests → httpx
- [ ] Consolidate HTTP wrappers into single lib
- [ ] heliosCLI: Clean up stale worktrees
- [ ] Plan Python 3.14 upgrade

### Long-term
- [ ] Create harbor-adapters from portage patterns
- [ ] Document CONFIG_GOVERNANCE.md
- [ ] Consolidate port/trait interfaces

---

_Last updated: 2026-04-03_