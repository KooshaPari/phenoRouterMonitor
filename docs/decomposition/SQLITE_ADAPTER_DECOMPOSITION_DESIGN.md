# SQLite Adapter Decomposition Design

## Executive Summary

This document designs the decomposition of a monolithic SQLite adapter (1,582 LOC) into three focused, independently testable modules while maintaining 100% backwards compatibility with existing consumers.

**Target:**
- Reduce `lib.rs` from 1,582 LOC → ~200 LOC public API
- Extract 3 logical subsystems: Sync Logic (~400 LOC), Query Builder (~300 LOC), Migrations (~250 LOC)
- Enable independent testing of each module without full SQLite setup
- Maintain zero-breaking changes to public API

**Effort Estimate:** 8-12 tool calls, 15-20 min wall-clock time per phase (5 atomic phases)

---

## Current State Analysis

### Monolithic Structure Problem

```
sqlite/lib.rs (1,582 LOC)
├── Repository implementation (600+ LOC)
│   ├── Connection pool management
│   ├── CRUD operations (create, read, update, delete)
│   ├── Query execution logic
│   └── Result mapping
├── SQL query building (300+ LOC)
│   ├── Dynamic WHERE clause generation
│   ├── JOIN construction
│   ├── Pagination helpers
│   └── Aggregate functions
├── Schema migrations (250+ LOC)
│   ├── Migration execution
│   ├── Schema validation
│   ├── Rollback logic
│   └── Version tracking
└── Utility functions (432 LOC)
    ├── Error conversion
    ├── Type serialization/deserialization
    └── Index/constraint management
```

### Problems This Solves

1. **Testing Friction**: Cannot test query builder without database
2. **Reusability**: Query builder useful for other backends (PostgreSQL, MySQL)
3. **Maintainability**: 1,582 LOC exceeds cognitive load, 1015 indentation levels
4. **Composition**: Cannot swap migration or sync strategies
5. **Modularity**: Violates single responsibility principle

---

## Proposed Architecture

### Trait Hierarchy (Hexagonal Ports)

```
┌─────────────────────────────────────────────────────────────────┐
│                      Public API Layer                            │
│                    (Port Abstractions)                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Repository<T>  QueryBuilder   MigrationRunner   SyncStore     │
│      trait            trait          trait          trait      │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                    Adapter Implementation                        │
│                   (sqlite/lib.rs exports)                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────┐ │
│  │  store/sync.rs   │  │store/query_b.rs  │  │migrations.rs │ │
│  │   (~400 LOC)     │  │   (~300 LOC)     │  │ (~250 LOC)   │ │
│  │                  │  │                  │  │              │ │
│  │ - Connection mgmt│  │ - WHERE clauses  │  │- Run schema  │ │
│  │ - CRUD helpers   │  │ - JOIN building  │  │- Version mgmt│ │
│  │ - Transactions   │  │ - Pagination     │  │- Rollback    │ │
│  │ - Row mapping    │  │ - Aggregates     │  │- Validation  │ │
│  └──────────────────┘  └──────────────────┘  └──────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Module Breakdown

### store/sync.rs (~400 LOC)
**Connection pooling, CRUD operations, row synchronization**

### store/query_builder.rs (~300 LOC)
**Dynamic SQL construction, type-safe query composition**

### store/migrations.rs (~250 LOC)
**Schema versioning, migration execution, rollback**

---

## Backwards Compatibility Guarantee

**No breaking changes to public API.** All existing code continues to work:

```rust
// Old code still compiles
let repo = SqliteRepository::new(config)?;
repo.create(entity).await?;
let entity = repo.read(&id).await?;
```

---

## Success Criteria

- lib.rs reduced from 1,582 LOC → ~200 LOC (87% reduction)
- 3 focused modules created: sync, query_builder, migrations
- 33+ new tests added (mostly unit, no database)
- Query builder testable without database (15+ unit tests)
- 0 breaking changes to public API
- ≥85% test coverage per module
- All backwards compatibility tests passing

---

## Next Steps

1. Review module blueprints (SQLITE_MODULE_BLUEPRINT.md)
2. Follow implementation roadmap (SQLITE_IMPLEMENTATION_ROADMAP.md)
3. Execute 5 phases atomically with tests at each step
4. Verify backwards compatibility before merge
