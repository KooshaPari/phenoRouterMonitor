# phenoSDK Extraction Strategy

## Overview

**phenoSDK** extracts reusable functionality into shareable @phenotype packages. Enables code reuse while maintaining clear ownership and versioning.

**Status**: Strategy Defined (2026-03-29)
**Target**: Extract 5+ packages; reduce duplication 30%

---

## 4-Tier Extraction Strategy

### Tier 1: Consolidation (Immediate)
Consolidate functionality existing in multiple places.

**Candidates**:
- Error handling (85+ enums → 5 types)
- Health checks (5+ → 1 trait)
- Config loading (4+ → 1 unified)
- Logging middleware (6+ → 1 generic)

**Effort**: Low (500-800 LOC)
**Timeline**: 2-3 weeks
**ROI**: High

### Tier 2: Extraction (Phase 1-2)
Extract well-defined isolated code.

**Candidates**:
- Retry with backoff
- Cache adapters
- Secret management
- Time utilities

**Effort**: Medium (1,200-1,800 LOC)
**Timeline**: 4-6 weeks
**ROI**: Medium

### Tier 3: Abstraction (Phase 2-3)
Abstract patterns into traits/generics.

**Effort**: High (2,000-3,000 LOC)
**Timeline**: 8-12 weeks
**ROI**: Very High

### Tier 4: Platform Libraries (Phase 3+)
Full platforms extracted.

**Effort**: Very High (10,000+ LOC)
**Timeline**: 16+ weeks
**ROI**: Strategic

---

## Tier 1 Progress

| Package | Status | Impact |
|---------|--------|--------|
| phenotype-error-core | ✅ Complete | 85+ → 5 types |
| phenotype-health | ✅ Complete | 5+ → 1 trait |
| phenotype-config-core | ✅ Complete | 4+ → 1 unified |
| phenotype-logging | In Progress | 6+ → 1 middleware |
| phenotype-time | Queued | Duration + scheduling |

---

## Publishing & Versioning

### Version Scheme
- `MAJOR.MINOR.PATCH` (SemVer)
- `0.x.y` during stabilization
- `1.0.0` when stable

### Locations

**Workspace** (local):
```toml
phenotype-error-core = { path = "../crates/phenotype-error-core" }
```

**Published** (crates.io):
```toml
phenotype-error-core = "1.0.0"
```

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Code Reuse | 30% LOC reduction |
| Package Count | 5+ extracted |
| Adoption | 100% of applicable repos |
| Test Coverage | 80%+ per package |
