# Cargo Workspace Audit — Quick Reference (2026-03-30)

## At a Glance

| Category | Count | Status |
|----------|-------|--------|
| Workspace Members | 7 | ✓ All valid |
| Members Version | 0.2.0 | ✓ Consistent |
| Workspace Dependencies | 24 | ⚠️ 8 unused |
| Orphaned Crate Dirs | 42 | 🔴 Critical |
| Recently Removed from Members | 12 | 🔴 Critical |

## Critical Issues

### Issue #1: 12 Removed Crates Still Exist
These were removed from `[workspace.members]` but directories remain:
- phenotype-config-loader
- phenotype-git-core
- phenotype-iter
- phenotype-logging
- phenotype-mcp
- phenotype-rate-limit
- phenotype-retry
- phenotype-state-machine
- phenotype-string
- phenotype-time
- phenotype-validation

**Action**: Decide to restore or archive.

### Issue #2: 23 AgilePlus Crates in Wrong Location
All `agileplus-*` crates (23 total) should be in separate workspace or clearly documented.

**Action**: Segregate or explicitly add to members list.

## Unused Workspace Dependencies (8 candidates)

Remove or document:
- `blake3` v1.5 — Hash function, not used
- `futures` v0.3 — Not used
- `lru` v0.12 — Caching, not used
- `once_cell` v1.19 — Singleton, not used
- `parking_lot` v0.12 — Locks, not used
- `strum` v0.26 — Enum utils, not used
- `phenotype-errors` (workspace.dependencies) — No member uses it

## Aspirational Dependencies (5)
Document intended use or remove:
- `anyhow` — Error context (future)
- `moka` — Advanced caching (future)
- `reqwest` — HTTP client (service integrations)
- `tracing` — Distributed tracing (observability)
- `hex` — Hex encoding (may be transitive)

## What's Working Well ✓

- All 7 members present with valid Cargo.toml
- Version consistency perfect (0.2.0)
- No circular dependencies
- Clean layering: error-core → errors → contracts → higher-level
- Appropriate build profiles (dev = 0, release = aggressive)
- Edition consistency (all 2021)
- MSRV sensible (1.75)

## Files

- **Full Audit**: `/docs/audits/CARGO_WORKSPACE_AUDIT_2026-03-30.md` (453 lines, 17KB)
  - Contains 12 detailed recommendations across 3 phases
  - Dependency usage table
  - Orphaned crate inventory
  - Build profile analysis
  - Conclusion and confidence assessment

---

**Audit Date**: 2026-03-30
**Repository**: phenotype-infrakit (https://github.com/KooshaPari/phenotype-infrakit)
**Status**: COMPLETED — Immediate action required on 2 critical issues
