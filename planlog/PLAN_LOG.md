# Plan Log

## Audit Planning Sessions

### 2026-03-29 - Code Duplication & Consolidation Audit

**Session ID:** audit-planning-20260329
**Agent:** Sage (Research) + Muse (Planning) + Forge (Implementation)
**Objective:** Identify code duplication and consolidation opportunities across AgilePlus and cross-repo

### Pre-Session Research (Sage)

#### Scope Defined:
1. **Intra-repo duplication** - within AgilePlus across crates
2. **Cross-repo patterns** - between AgilePlus, phenotype-shared, libs/
3. **Libification candidates** - patterns extractable to shared libraries
4. **Productization candidates** - internal→external patterns
5. **3rd party gaps** - missing/duplicate external dependencies

#### Initial Plan:
```
1. Health check pattern audit (3+ implementations)
2. Error type audit (15+ error enums)
3. Config loading pattern audit (TOML/YAML/env)
4. Port/trait architecture audit
5. API response pattern audit
6. External crate recommendations
7. LOC savings quantification
```

### Session Execution

**Phase 1: Discovery (20 min)**
- Search for thiserror::Error patterns
- Search for health check implementations
- Search for config loading patterns
- Search for port traits

**Phase 2: Deep Dive (30 min)**
- Read health.rs files in agileplus-cache, agileplus-graph, agileplus-nats
- Read error.rs files across 15+ crates
- Read config files across domains
- Read port interface definitions

**Phase 3: Quantification (15 min)**
- Count LOC per pattern category
- Identify canonical locations
- Estimate savings

**Phase 4: Documentation (15 min)**
- Create AGILEPLUS_DUPLICATION_AUDIT_20260329.md
- Update DUPLICATION_AUDIT.md
- Create AUDIT_FRAMEWORK.md
- Create AGILEPLUS_DECOMPOSITION_AUDIT.md

### Deliverables

| Deliverable | Status | Location |
|-------------|--------|----------|
| AUDIT_FRAMEWORK.md | ✅ | docs/reports/ |
| AGILEPLUS_DUPLICATION_AUDIT_20260329.md | ✅ | docs/reports/ |
| AGILEPLUS_DECOMPOSITION_AUDIT.md | ✅ | docs/reports/ |
| CODEBASE_ATLAS.md | ✅ Updated | docs/reports/ |
| DUPLICATION_AUDIT.md | ✅ Updated | Root |

### Estimated Savings

| Pattern | Current LOC | Savings |
|---------|-------------|---------|
| Health checks | 140 | 80 |
| Error types | 504 | 150 |
| Config loaders | 449 | 200 |
| API types | 224 | 50 |
| **Total** | **1,317** | **480** |

### Next Session: Decomposition Audit

**Scheduled:** 2026-03-29 (follow-up)

**Focus Areas:**
1. Crate boundary analysis
2. Overly large crate identification
3. Undersized crate consolidation
4. Feature domain mapping

---

## 2026-03-29 - Extended Audit Session

**Session ID:** extended-audit-20260329
**Objective:** Continue audit with additional pattern categories

### Additional Patterns Investigated

1. **Builder Pattern Proliferation**
   - EventQuery builder (9 methods)
   - CacheConfig builder
   - NATS config builder

2. **Async Trait Patterns**
   - SnapshotStore using #[async_trait] but NOT in phenotype-port-interfaces

3. **Phenotype-Shared vs AgilePlus Divergence**
   - Two hexagonal ecosystems with overlapping concerns
   - PortError vs DomainError
   - Repository trait vs StoragePort

4. **Connection Pool Patterns**
   - bb8 vs deadpool inconsistency
   - Pool management duplication

5. **Snapshot Patterns**
   - Event snapshots vs Git snapshots vs P2P snapshots

6. **Cross-Language Patterns**
   - hexagonal-ts not integrated with Rust patterns
   - phenotype-config (TypeScript) vs config-core (unused Rust)

### Action Items Generated

| Priority | Count | Examples |
|----------|-------|----------|
| 🔴 CRITICAL | 1 | Create agileplus-health crate |
| 🟡 HIGH | 6 | agileplus-error-core, integrate config-core, move SnapshotStore |
| 🟠 MEDIUM | 10 | QueryBuilder trait, port consolidation, hexagonal-ts evaluation |
| 🟢 LOW | 8 | content_hash audit, generic snapshot trait, bb8→deadpool |

### Files Updated

- DUPLICATION_AUDIT.md (+150 lines)
- docs/reports/CODEBASE_ATLAS.md (decomposition section added)

---

## Planning Notes

### For 30 Agent Distribution

| Category | Primary Agent | Subagents |
|----------|--------------|-----------|
| Duplication | Sage | Parallel file analysis |
| Decomposition | Sage | Crate boundary analysis |
| Dependencies | Sage | Package inventory |
| Quality | Muse | Test coverage, complexity |
| Productization | Forge | Pattern identification |
| Consolidation | Sage | Fragmented impl analysis |

### Coordination Protocol

1. Check docs/reports/README.md before starting
2. Update audit log before starting work
3. Use checkbox format for action items
4. Cite findings as filepath:line
5. Report cross-repo impacts to DUPLICATION_AUDIT.md

---

*Log maintained by Forge agent*
*Last updated: 2026-03-29T07:37:00Z*
