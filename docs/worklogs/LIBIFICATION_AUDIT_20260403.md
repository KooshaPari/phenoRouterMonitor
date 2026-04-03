# Libification & Modernization Audit - 2026-04-03

**Project:** [heliosApp], [portage], [heliosCLI]
**Category:** libification, modernization
**Status:** in_progress
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

## Action Items

### Immediate (This Sprint)
- [ ] Upgrade heliosApp to TypeScript 7
- [ ] Complete portage PR #250 viewer lockfile cleanup
- [ ] Fix extended_benchmark.py mixed HTTP imports
- [ ] Add @tanstack/solid-query to heliosApp

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