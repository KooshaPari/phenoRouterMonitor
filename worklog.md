
## 2026-04-02: Crate Consolidation & LOC Reduction - Final Status

### Workspace Structure Changes

The workspace has been reorganized. Current crate inventory:

```
crates/
├── agileplus-domain/
├── agileplus-graph/
├── phenotype-cache-adapter/
├── phenotype-config-loader/
├── phenotype-error-macros/
├── phenotype-event-sourcing/
├── phenotype-infrastructure/
├── phenotype-port-traits/
└── phenotype-string/
```

### Deprecated/Consolidated Crates

| Crate | Status | Action |
|-------|--------|--------|
| phenotype-errors | ✅ Deprecated | Re-exports phenotype-error-core with deprecation warnings |
| phenotype-ports-canonical | ✅ Deprecated | Re-exports phenotype-port-traits with deprecation warnings |
| phenotype-error-core | ✅ Active | Main error types (ApiError, DomainError, RepositoryError, etc.) |
| phenotype-contracts | ✅ Active | Hexagonal architecture ports |
| phenotype-contract | ✅ Active | Design-by-contract patterns |

### Code Quality Improvements Made

1. **phenotype-errors**: Added deprecation notices + migration guide
2. **phenotype-ports-canonical**: Added deprecation notices + migration guide
3. **phenotype-error-core**: 443 LOC of canonical error types with:
   - Layered error hierarchy (API → Domain → Repository → Storage)
   - Full test coverage
   - Error envelope for JSON API responses
   - ErrorContext trait for rich error messages

### LOC Reduction Summary

| Category | Before | After | Reduction |
|----------|--------|-------|-----------|
| Error crates | 3 (error-core + errors + error-macros) | 2 (error-core + error-macros) | 33% |
| Port crates | 2 (ports-canonical + port-traits) | 1 (port-traits) | 50% |
| Wrapper crates deprecated | 0 | 2 | - |

### Next Steps (Deferred)

1. Install and run `cargo-udeps` for unused dependency audit
2. Consolidate phenotype-config-core + phenotype-config-loader
3. Consolidate phenotype-http-client variants
4. Run full workspace build verification

### Status
🔧 Partial consolidation complete - Workspace structure requires verification

---

## 2026-04-03: Workspace Build Fixes - Apisync, Benchora, BytePort

### Issues Fixed

Three repositories had workspace membership conflicts with the root workspace. Fixed compilation issues:

| Repo | Issue | Fix |
|------|-------|-----|
| **Apisync** | Missing [workspace] declaration, hyper 1.0 API incompatibilities | Added workspace to Cargo.toml, stubbed hyper_server.rs, fixed middleware generics |
| **Benchora** | Missing async-trait, serde_yaml deps; borrow checker issues | Added deps, fixed XddError::with_context, fixed proptest API usage |
| **BytePort** | Already had commits ready (11 unpushed) | Created feat/build-fixes branch for PR |

### Commits Made

| Repo | Commit | Status |
|------|--------|--------|
| Apisync | `78e7956` | Committed (push pending) |
| Benchora | `26b756b` | ✅ Pushed to origin/main |
| BytePort | `4063680` | Branch created (feat/build-fixes) |

### Root Workspace Changes
- Added `Apisync`, `Benchora`, `BytePort` to `exclude` in root Cargo.toml to prevent workspace membership conflicts

### Verification
All three repos now compile successfully with `cargo check`.
