# Planned But Never Created Libraries

**Date**: 2026-03-30
**Status**: Decision Required

The following libraries were referenced in the original workspace Cargo.toml but were **never created**:

## Non-Existent Libraries

| Library | Purpose | Status |
|---------|---------|--------|
| `hexagonal-rs` | Hexagonal architecture patterns | Not created |
| `hexkit` | Kit library for hexagonal | Not created |
| `cipher` | Cryptography utilities | Not created |
| `gauge` | Metrics/measurement | Not created |
| `logger` | Logging infrastructure | Not created |
| `metrics` | Metrics collection | Not created |
| `tracing` | Distributed tracing | Not created |
| `cli-framework` | CLI argument parsing | Not created |
| `config-core` | Configuration management | Not created |
| `xdd-lib-rs` | XDD utilities | Not created |
| `tools/forge` | Forge tooling | Not created |

## Recommendation

**Option A: Create as needed**
- Add these libraries only when a specific use case requires them
- Start with `logger`, `metrics`, `config-core` as they're commonly needed

**Option B: Remove from roadmap**
- Document these as "deferred" and remove from future planning
- Focus on existing plugin/intent/health system

**Option C: Create all**
- Create stub implementations for all 11 libraries
- Estimated time: 4-6 hours

## Decision

Select an option to proceed.

---

## Created Libraries (Implemented)

| Library | Status | Tests |
|---------|--------|-------|
| `nexus` | ✅ Active | ? |
| `plugin-registry` | ✅ Phase 2 | 4 |
| `plugin-sample` | ✅ Phase 2 | 0 |
| `plugin-cli` | ✅ Phase 2 | 0 |
| `plugin-git` | ✅ Phase 2 | 0 |
| `plugin-grpc` | ✅ Phase 2 | 0 |
| `plugin-integration` | ✅ Phase 2.4 | 1 |
| `intent-registry` | ✅ Phase 3 | 10 |
| `health-monitor` | ✅ Phase 4 | 7 |
