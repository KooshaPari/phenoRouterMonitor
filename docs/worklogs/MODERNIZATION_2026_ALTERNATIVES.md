# 2026 Technology Alternatives Research

**Project:** [cross-repo]
**Category:** research, modernization
**Status:** completed
**Priority:** P1

## Research Date: 2026-04-03

This document tracks 2026 alternatives for existing technology choices across the Phenotype ecosystem.

---

## TypeScript / JavaScript

### TypeScript

| Current | Latest | Recommendation | Notes |
|---------|--------|----------------|-------|
| 5.8.2 | **7.x** | ✅ Upgrade | TS7 now available - significant perf improvements |

### State Management (SolidJS)

| Current | 2026 Alternative | Recommendation | Notes |
|---------|------------------|----------------|-------|
| Manual `createSignal`/`createStore` | `@tanstack/solid-query` v5 | ✅ Add | Server state caching, optimistic updates |
| Custom hooks | `solid-hook-form` | Consider | Form validation |
| - | `@solidjs/router` | Consider | If SPA routing needed |

### Build Tools

| Current | 2026 Alternative | Recommendation | Notes |
|---------|------------------|----------------|-------|
| esbuild + solid plugin | Current is optimal | ✅ Keep | esbuild fastest for bundling |
| Vite 5.x | Vite 6.x | ✅ Upgrade when deps support | Better HMR |
| Turborepo | Bun scripts | ✅ Keep | Bun workspace sufficient |

### Linting

| Current | 2026 Alternative | Recommendation | Notes |
|---------|------------------|----------------|-------|
| Biome 2.4.10 | oxlint (VoidZero) | Keep oxlint | Performance advantage |
| oxlint 1.57.0 | Keep current | ✅ Keep | VoidZero direction |

### Testing

| Current | 2026 Alternative | Recommendation | Notes |
|---------|------------------|----------------|-------|
| Vitest 4.1.2 | Vitest 5.x | ✅ Upgrade | Faster parallelization |
| Playwright 1.58.2 | Playwright 2.x | ✅ Upgrade | Improved tracing |

---

## Python

### HTTP Client

| Current | 2026 Alternative | Recommendation | Notes |
|---------|------------------|----------------|-------|
| `requests` | `httpx` | ⚠️ Migrate | Async support, modern API |
| - | `aiohttp` | Deprecate | Consolidate to httpx |

### Retry Logic

| Current | 2026 Alternative | Recommendation | Notes |
|---------|------------------|----------------|-------|
| `tenacity` | `stamina` | ✅ Migrate | Hynek's opinionated wrapper |
| custom retry | `backoff` crate (Rust) | Consider | If in Rust code |

### Data Validation

| Current | 2026 Alternative | Recommendation | Notes |
|---------|------------------|----------------|-------|
| Pydantic v2 | Current | ✅ Keep | Already on latest |
| - | `pydantic-settings` | ✅ Add | For env config (missing in some projects) |

### Package Management

| Current | 2026 Alternative | Recommendation | Notes |
|---------|------------------|----------------|-------|
| uv | Current | ✅ Keep | Already modern |
| pip | Deprecate | ✅ Use uv | Throughout |

### Python Version

| Current | Target | Recommendation | Notes |
|---------|--------|----------------|-------|
| 3.12+ | 3.14 | Plan | Awaiting CI runner support |

---

## Rust

### Config

| Current | 2026 Alternative | Recommendation | Notes |
|---------|------------------|----------------|-------|
| toml 0.8 | toml 0.9.5 | ✅ Standardize | Version fragmentation |
| figment | Keep | ✅ Keep | Good for flexible loading |
| phenotype-config-core | Keep | ⚠️ Promote | Currently underutilized |

### Error Handling

| Current | 2026 Alternative | Recommendation | Notes |
|---------|------------------|----------------|-------|
| thiserror (93%) | Current | ✅ Keep | Already standardized |
| custom errors | Consolidate | Plan | libs/error-core |

---

## DevOps / CI

### Bun

| Current | Target | Recommendation | Notes |
|---------|--------|----------------|-------|
| 1.2.20 | Latest | ✅ Pin org-wide | Currently varies |

### Node Package Manager

| Current | Target | Recommendation | Notes |
|---------|--------|----------------|-------|
| npm + bun (dual) | Bun only | ✅ Consolidate | portage viewer fix in PR #250 |

---

## Key Migration Priorities

### This Week
1. TypeScript 7 upgrade in heliosApp
2. Migrate portage `requests` → `httpx`
3. Fix `extended_benchmark.py` mixed imports

### This Month
1. Add `@tanstack/solid-query` to heliosApp
2. Consolidate viewer lockfiles (verify PR #250)
3. Plan Python 3.14 upgrade path
4. Clean up stale heliosCLI worktrees

### This Quarter
1. Create shared libraries (harbor-adapters, harbor-benchmarks)
2. Document config governance patterns
3. Consolidate port/trait interfaces

---

## References

- TypeScript 7 announcement: https://devblogs.microsoft.com/typescript/announcing-typescript-7/
- TanStack Solid Query: https://tanstack.com/query/latest/solid/overview
- stamina (Python retry): https://pypi.org/project/stamina/
- VoidZero/oxlint: https://oxc-project.com/

---

_Last updated: 2026-04-03_