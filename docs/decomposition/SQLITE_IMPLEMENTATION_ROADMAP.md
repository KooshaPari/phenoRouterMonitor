# SQLite Adapter Decomposition — Implementation Roadmap

## Quick Reference

| Phase | Duration | LOC Removed | Tests Added | Key Output |
|-------|----------|------------|------------|-----------|
| 1: Structure | 3-5 min | 0 | 0 | Module skeleton + re-exports |
| 2: Sync Logic | 5-7 min | 400 | 8 | ConnectionPool isolated |
| 3: Query Builder | 5-7 min | 300 | 15 | QueryBuilder tested independently |
| 4: Migrations | 5-7 min | 250 | 10 | MigrationRunner with state machine |
| 5: Polish | 3-5 min | 632 | 5 | Backwards compat verified, lib.rs → 200 LOC |

**Total Effort:** ~32 min wall-clock, 8-15 tool calls per phase = 40-75 tool calls

---

## Phase 1: Create Module Structure & Skeleton (5 min)

### Goal
Establish module files, create public API re-exports, verify compilation.

### Work Items

#### WI-1.1: Create module files
1. Create `crates/agileplus-sqlite/src/store/` directory
2. Create `store/mod.rs` (re-exports)
3. Create `store/sync.rs` (empty module)
4. Create `store/query_builder.rs` (empty module)
5. Create `store/migrations.rs` (empty module)
6. Update `lib.rs` to declare `mod store;`

**Tests:** `cargo test` compiles and runs

---

#### WI-1.2: Define public trait contracts
1. Create trait stubs for `SyncStore<T>`
2. Create trait stubs for `QueryBuilder`
3. Create trait stubs for `MigrationRunner`

**Tests:** `cargo check` and `cargo doc` pass

---

#### WI-1.3: Verify public API stability
1. Run all existing tests without modification
2. Verify no compiler errors in dependent crates
3. Document current public API

**Tests:** `cargo test --all` passes

---

## Phase 2: Extract Sync Logic (~400 LOC) (7 min)

### Work Items

#### WI-2.1: Move connection pool implementation
Move `ConnectionPool`, `ConnectionConfig`, `SyncMetrics` to `store/sync.rs`
**Tests:** Connection pool creation, read/write isolation

#### WI-2.2: Move row mapping and synchronization
Move `RowMapper` trait, synchronization context types
**Tests:** Row type conversion tests

#### WI-2.3: Move transaction helpers
Move `read_tx`, `write_tx`, `bulk_insert`, `stream` implementations
**Tests:** Transaction rollback, bulk insert atomicity

#### WI-2.4: Move metrics and monitoring
Move `SyncMetrics` tracking and reporting
**Tests:** Metrics increment on operations

#### WI-2.5: Implement SyncStore trait
Implement trait for `ConnectionPool`
**Tests:** Trait method verification

#### WI-2.6: Update lib.rs re-exports
Remove moved code, add re-exports
**Tests:** `cargo test --all` passes

---

## Phase 3: Extract Query Builder (~300 LOC) (7 min)

### Work Items

#### WI-3.1: Move Filter type and operators
Move `Filter`, `Operator`, `Logic` enums and constructors
**Tests:** Filter eq/ne/gt/lt, parameterization checks

#### WI-3.2: Move Join type and specification
Move `JoinType`, `Join` struct and constructors
**Tests:** JOIN validation tests

#### WI-3.3: Move QueryBuilder struct and basic methods
Move `SqliteQueryBuilder`, `select()`, `from()`
**Tests:** Simple SELECT generation

#### WI-3.4: Move WHERE, JOIN, ORDER BY, LIMIT logic
Complete query building implementation
**Tests:** WHERE clause generation, pagination, ordering

#### WI-3.5: Move Aggregate function support
Move `Aggregate` type and GROUP BY support
**Tests:** COUNT, SUM, GROUP BY tests

#### WI-3.6: Move validation and SQL generation
Move `validate()`, SQL string building
**Tests:** SQL validation (missing FROM), generation correctness

#### WI-3.7: Implement QueryBuilder trait
Trait implementation for `SqliteQueryBuilder`
**Tests:** All WI-3 tests cover this

#### WI-3.8: Update lib.rs re-exports
Remove query builder code, add re-exports
**Tests:** `cargo test --all` passes

---

## Phase 4: Extract Migrations (~250 LOC) (7 min)

### Work Items

#### WI-4.1: Move Migration trait and version tracking
Move `Migration` trait, `MigrationVersion` struct
**Tests:** Migration trait contract, version tracking

#### WI-4.2: Move MigrationRunner implementation
Move `SqliteMigrationRunner`, migration registry
**Tests:** Migration ordering, duplicate rejection

#### WI-4.3: Move migration execution logic
Move `migrate()`, per-migration execution, transaction management
**Tests:** Single/multiple migration execution, partial target

#### WI-4.4: Move rollback logic
Move `rollback()` implementation
**Tests:** Rollback verification, multiple steps

#### WI-4.5: Move schema verification and validation
Move `verify()` method, `SchemaVerification` struct
**Tests:** Schema integrity verification

#### WI-4.6: Implement MigrationRunner trait
Trait implementation for `SqliteMigrationRunner`
**Tests:** All WI-4 tests cover this

#### WI-4.7: Update lib.rs re-exports
Remove migration code, add re-exports
**Tests:** `cargo test --all` passes

---

## Phase 5: Polish & Finalization (5 min)

### Work Items

#### WI-5.1: Backwards compatibility testing
Create integration tests for old public APIs
**Tests:** 5+ backwards compat tests

#### WI-5.2: Clean up lib.rs
Reduce to ~200 LOC, verify zero warnings
**Tests:** `cargo clippy`, `cargo fmt`, `cargo test`

#### WI-5.3: Add module documentation
Doc comments, usage examples for each module
**Tests:** `cargo doc` builds, examples compile

#### WI-5.4: Run full test suite
Unit + integration + backwards compat
**Tests:** All tests pass, zero warnings

#### WI-5.5: Update CHANGELOG
Document decomposition work, backwards compat guarantee
**Tests:** CHANGELOG reviewed

---

## Test Strategy Summary

### Unit Tests (No Database)
- Query builder: 15+ tests for SQL generation
- Property-based tests for SQL injection prevention
- ~200 LOC tests

### Integration Tests (With :memory: SQLite)
- Sync store: Connection pool, transactions, atomicity
- Migrations: State machine, rollback, verification
- ~300 LOC tests

### Backwards Compatibility Tests
- All old public APIs still work
- Repository CRUD operations unchanged
- Query builder API unchanged
- Migration API unchanged
- ~100 LOC tests

---

## Success Metrics

| Metric | Target | Status |
|--------|--------|--------|
| lib.rs LOC reduction | 87% (1582→200) | ___ |
| New test count | 33+ | ___ |
| Query builder unit tests | 15+ | ___ |
| Sync store tests | 8+ | ___ |
| Migration tests | 10+ | ___ |
| Backwards compat tests | 5+ | ___ |
| Coverage per module | ≥85% | ___ |
| Breaking changes | 0 | ___ |

---

## Timeline & Effort

| Phase | Duration | Parallel? | Notes |
|-------|----------|-----------|-------|
| 1. Structure | 5 min | - | Sequential setup |
| 2. Sync | 7 min | Yes (with 3,4) | Can run in parallel |
| 3. Query | 7 min | Yes (with 2,4) | Can run in parallel |
| 4. Migrations | 7 min | Yes (with 2,3) | Can run in parallel |
| 5. Polish | 5 min | No | Sequential finalization |

**Sequential Total:** ~32 min
**With 3 Parallel Agents (phases 2-4):** ~15 min wall-clock

---

## Next Steps

1. Assign Phase 1 to architect/lead agent
2. Execute Phases 2-4 in parallel (separate agents)
3. Run Phase 5 for final verification
4. Merge with full test coverage
5. Update dependent crates with optional optimizations
