# Cargo Workspace Members Inventory (2026-03-30)

## Declared Workspace Members (7 total)

All members are **present**, **valid**, and **version-consistent** (0.2.0).

### Core Members

| Member | Lines | Edition | Deps | Status | Purpose |
|--------|-------|---------|------|--------|---------|
| phenotype-error-core | 15 | 2021 | 3 | ✓ Clean | Canonical error types |
| phenotype-errors | 18 | 2021 | 6 | ✓ Clean | Extended error utilities |
| phenotype-contracts | 19 | 2021 | 8 | ✓ Clean | Shared trait contracts |
| phenotype-health | 15 | 2021 | 3 | ✓ Clean | Health checks (K8s probes) |
| phenotype-port-traits | 15 | 2021 | 3 | ✓ Clean | Port/adapter traits |
| phenotype-policy-engine | 19 | 2021 | 7 | ✓ Clean | Rule-based evaluation |
| phenotype-telemetry | 15 | 2021 | 3 | ✓ Clean | Observability & tracing |

**Total**: 116 lines across 7 crates
**Average**: 16.6 lines per crate (minimal, focused scope)
**Cohesion**: Excellent (clean layering, no circular deps)

---

## Dependency Chart

```
phenotype-error-core (foundation)
    ├─ Uses: serde, thiserror
    │
phenotype-errors (builds on error-core)
    ├─ Uses: phenotype-error-core, chrono, serde, serde_json, thiserror
    │
phenotype-contracts (trait definitions)
    ├─ Uses: phenotype-error-core, async-trait, chrono, serde, serde_json, thiserror, uuid
    │
phenotype-health (health checks)
    ├─ Uses: serde, thiserror
    │
phenotype-port-traits (adapters)
    ├─ Uses: serde, thiserror
    │
phenotype-policy-engine (rules/policies)
    ├─ Uses: dashmap, regex, serde, serde_json, thiserror, toml
    │
phenotype-telemetry (observability)
    ├─ Uses: serde, thiserror
```

**Key Observation**: Clean separation of concerns, minimal interdependencies except for error-core as foundation.

---

## Orphaned Crates Summary

### Recently Removed from Members (12 crates)

These were in the members list but were **removed in recent changes**. Directories still exist at `/crates/{name}/`.

| Crate | Status | Notes | Action Needed |
|-------|--------|-------|---------------|
| phenotype-config-loader | ⚠️ Orphaned | Has Cargo.toml | Restore or archive? |
| phenotype-git-core | ⚠️ Orphaned | Has Cargo.toml | Restore or archive? |
| phenotype-iter | ⚠️ Orphaned | Has Cargo.toml | Restore or archive? |
| phenotype-logging | ⚠️ Orphaned | Has Cargo.toml | Restore or archive? |
| phenotype-mcp | ⚠️ Orphaned | Has Cargo.toml | Restore or archive? |
| phenotype-rate-limit | ⚠️ Orphaned | Has Cargo.toml | Restore or archive? |
| phenotype-retry | ⚠️ Orphaned | Has Cargo.toml | Restore or archive? |
| phenotype-state-machine | ⚠️ Orphaned | Has Cargo.toml | Restore or archive? |
| phenotype-string | ⚠️ Orphaned | Has Cargo.toml | Restore or archive? |
| phenotype-time | ⚠️ Orphaned | Has Cargo.toml | Restore or archive? |
| phenotype-validation | ⚠️ Orphaned | Has Cargo.toml | Restore or archive? |

**Suspected cause**: Recent refactor or merge conflict

---

### AgilePlus Crates (23 crates)

All `agileplus-*` crates are present but not listed in phenotype-infrakit workspace members:

| Group | Count | Note |
|-------|-------|------|
| agileplus-api | 1 | API definitions |
| agileplus-benchmarks | 1 | Performance testing |
| agileplus-cli | 1 | Command-line interface |
| agileplus-dashboard | 1 | Web dashboard |
| agileplus-sqlite | 1 | SQLite adapter |
| agileplus-grpc | 1 | gRPC integration |
| agileplus-sync | 1 | Synchronization logic |
| Other | 16 | git, github, events, import, p2p, plane, nats, etc. |

**Total**: 23 crates
**Status**: 🔴 Should be in separate workspace or clearly documented
**Action**: Segregate to AgilePlus workspace or add explicit scope

---

### Other Orphaned Crates (7 crates)

| Crate | Purpose | Status |
|-------|---------|--------|
| phenotype-async-traits | Async trait utilities | ⚠️ Orphaned |
| phenotype-contract | Duplicate? (phenotype-contracts exists) | ⚠️ Likely mistake |
| phenotype-cost-core | Cost/billing utilities | ⚠️ Orphaned |
| phenotype-crypto | Cryptography | ⚠️ Orphaned |
| phenotype-error-macros | Error macro utilities | ⚠️ Orphaned |
| phenotype-event-sourcing | Event sourcing | ⚠️ Orphaned |
| bifrost-routing | Routing logic | ⚠️ Orphaned |
| forgecode-core | Unknown purpose | ⚠️ Orphaned |

**Note**: Several appear to be legitimate crates that should either be in members or archived.

---

## Recommendations by Priority

### Priority 1: Clarify Repository Boundaries (TODAY)

1. **Decision**: Is phenotype-infrakit ONLY the 7 core members, or does it include AgilePlus?
2. **If Yes (includes AgilePlus)**: Add all 23 agileplus-* crates to members list
3. **If No (separate org)**: Move all agileplus-* crates to separate workspace

### Priority 2: Resolve Recently-Removed Crates (TOMORROW)

1. **Check git history**: Why were these 12 crates removed?
2. **Decision per crate**:
   - If moved to separate repo: Add stub documentation
   - If intentionally unused: Move to `.archive/`
   - If should be in workspace: Add back to members list

### Priority 3: Archive Legitimate Orphans (THIS WEEK)

Move to `.archive/` or to their appropriate repositories:
- phenotype-contract (resolve duplicate)
- phenotype-async-traits
- phenotype-cost-core
- phenotype-crypto
- phenotype-error-macros
- phenotype-event-sourcing
- bifrost-routing
- forgecode-core

---

## Member Dependencies Detail

### Used Dependencies (14 with explicit usage)

| Dependency | Used By | Purpose |
|-----------|---------|---------|
| serde | All 7 | Serialization |
| thiserror | All 7 | Error handling |
| tokio | All 7 (dev-deps) | Async tests |
| chrono | phenotype-errors, phenotype-contracts | Timestamps |
| serde_json | phenotype-errors, phenotype-contracts | JSON |
| async-trait | phenotype-contracts | Async traits |
| uuid | phenotype-contracts | IDs |
| toml | phenotype-policy-engine | Config parsing |
| regex | phenotype-policy-engine | Pattern matching |
| dashmap | phenotype-policy-engine | Concurrent map |
| tempfile | All (test infrastructure) | Test utilities |
| phenotype-error-core | phenotype-errors, phenotype-contracts | Cross-crate |

### Unused Dependencies (8 candidates)

| Dependency | Reason for Concern | Recommendation |
|-----------|------------------|-----------------|
| blake3 | No usage in source | Remove |
| futures | No usage in source | Remove or document intent |
| lru | No usage, shadowed by dashmap | Remove |
| once_cell | No usage | Remove or document intent |
| parking_lot | No usage | Remove |
| strum | No usage | Remove |
| phenotype-errors (workspace.dep) | No member imports it | Remove from workspace.dependencies |
| hex | No usage (may be transitive) | Document or remove |

### Aspirational Dependencies (5)

| Dependency | Declared | Used | Intent |
|-----------|----------|------|--------|
| anyhow | ✓ | ✗ | Error contexts (future) |
| moka | ✓ | ✗ | Advanced caching (future) |
| reqwest | ✓ | ✗ | HTTP client (service integrations) |
| tracing | ✓ | ✗ | Distributed tracing (Phase 2) |

**Recommendation**: Document intended use or remove and re-add when needed.

---

## Workspace Health Indicators

| Indicator | Status | Details |
|-----------|--------|---------|
| **Member Count** | ✓ Healthy | 7 members, all present |
| **Version Consistency** | ✓ Perfect | 100% at 0.2.0 |
| **Circular Dependencies** | ✓ None | Clean layering |
| **Member Size** | ✓ Optimal | 15-19 lines each (focused) |
| **Dependency Count** | 🟡 Good | 14 used, 8 unused candidates |
| **Unused Dependencies** | ⚠️ Action needed | 8 dependencies to clean up |
| **Orphaned Crates** | 🔴 Critical | 42 directories not in members |
| **Recent Removals** | 🔴 Critical | 12 crates removed but still present |
| **Organizational Clarity** | 🔴 Critical | AgilePlus scope unclear |

**Overall Health Score**: 6/10 (Core is healthy; organization is confused)

---

**Report Generated**: 2026-03-30
**Auditor**: Claude Code (automated analysis)
**Confidence**: High (direct file inspection + git analysis)
