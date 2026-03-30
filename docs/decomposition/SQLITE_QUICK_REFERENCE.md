# SQLite Adapter Decomposition — Quick Reference

## One-Page Overview

Transform 1,582 LOC monolithic SQLite adapter into 3 focused modules.

```
Before:               After:
lib.rs (1,582 LOC)   lib.rs (200 LOC)
                     ├── store/sync.rs (400 LOC)
                     ├── store/query_builder.rs (300 LOC)
                     └── store/migrations.rs (250 LOC)
```

---

## Module Breakdown

| Module | LOC | Responsibility | Key Traits | Tests |
|--------|-----|-----------------|-----------|-------|
| `sync.rs` | 400 | Connection pooling, transactions, row sync | `SyncStore<T>` | 8 integration |
| `query_builder.rs` | 300 | Dynamic SQL construction | `QueryBuilder` | 15 unit (no DB) |
| `migrations.rs` | 250 | Schema versioning, migrations | `MigrationRunner` | 10 state machine |

---

## Trait Contracts (Hexagonal Ports)

```rust
// store/sync.rs
pub trait SyncStore<T>: Send + Sync {
    async fn read_tx<F, R>(&self, f: F) -> Result<R>;
    async fn write_tx<F, R>(&self, f: F) -> Result<R>;
    async fn bulk_insert(&self, records: Vec<T>) -> Result<usize>;
}

// store/query_builder.rs
pub trait QueryBuilder: Send + Sync + Sized {
    fn select(columns: &[&str]) -> Self;
    fn from(table: &str) -> Self;
    fn where_clause(self, filter: Filter) -> Self;
    fn build(self) -> (String, Vec<SqlValue>);
}

// store/migrations.rs
pub trait MigrationRunner: Send + Sync {
    async fn migrate(&self, target: Option<i32>) -> Result<MigrationStatus>;
    async fn rollback(&self, steps: usize) -> Result<MigrationStatus>;
}
```

---

## Public API (lib.rs Re-exports)

All existing APIs continue to work without changes.

```rust
pub use store::sync::SyncStore;
pub use store::query_builder::QueryBuilder;
pub use store::migrations::MigrationRunner;

pub struct SqliteRepository<T> { /* ... */ }
impl<T> Repository<T> for SqliteRepository<T> { /* ... */ }
```

**Zero Breaking Changes**

---

## Test Strategy

### Query Builder (No Database)
```rust
#[test]
fn test_simple_select() {
    let (sql, params) = SqliteQueryBuilder::select(&["id", "name"])
        .from("users")
        .where_filter(Filter::eq("id", "123".into()))
        .build().unwrap();

    assert_eq!(params.len(), 1); // Parameterized, safe
}
```

### Sync Store (In-Memory)
```rust
#[tokio::test]
async fn test_transaction_atomicity() {
    let pool = ConnectionPool::new(":memory:").unwrap();
    let count = pool.write_tx(|conn| {
        conn.execute("INSERT INTO users VALUES (?)", [1])?;
        Ok(())
    }).await.unwrap();
}
```

### Migrations (State Machine)
```rust
#[tokio::test]
async fn test_migration_rollback() {
    let runner = SqliteMigrationRunner::new(pool);
    runner.migrate(Some(2)).await.ok();
    runner.rollback(1).await.ok();

    assert_eq!(runner.current_version().await.unwrap(), 1);
}
```

---

## Execution Plan (5 Phases)

| Phase | Duration | Output | Parallel? |
|-------|----------|--------|-----------|
| 1. Module structure | 5 min | Skeleton + re-exports | - |
| 2. Extract sync.rs | 7 min | ConnectionPool isolated | Yes |
| 3. Extract query_builder.rs | 7 min | QueryBuilder tested | Yes |
| 4. Extract migrations.rs | 7 min | MigrationRunner tested | Yes |
| 5. Polish | 5 min | Backwards compat verified | No |

**Total:** ~32 min sequential (~15 min with 3 parallel agents)

---

## Backwards Compatibility Guarantee

**No breaking changes.** All existing code compiles and runs:

```rust
let repo = SqliteRepository::new(config)?;
let id = repo.create(entity).await?;
let entity = repo.read(&id).await?;

let (sql, params) = query_builder
    .select(&["*"])
    .from("users")
    .where_eq("id", "123")
    .build()?;

runner.migrate().await?;
runner.rollback(1).await?;
```

---

## Key Benefits

1. **Testability** — Query builder without database (15+ unit tests)
2. **Reusability** — Traits for PostgreSQL, MySQL adapters
3. **Maintainability** — Each module ≤500 LOC, clear responsibility
4. **Composability** — Swap implementations
5. **Performance** — No regression, optimized per module

---

## File Structure (After Decomposition)

```
src/
├── lib.rs (200 LOC)
├── store/
│   ├── mod.rs
│   ├── sync.rs (400 LOC)
│   ├── query_builder.rs (300 LOC)
│   └── migrations.rs (250 LOC)
└── tests/
    ├── sync_tests.rs (150 LOC)
    ├── query_builder_tests.rs (250 LOC)
    ├── migration_tests.rs (200 LOC)
    └── backwards_compat.rs (100 LOC)
```

---

## Success Criteria

✓ lib.rs: 1,582 → 200 LOC (87% reduction)
✓ 3 focused modules
✓ 33+ new tests
✓ 0 breaking changes
✓ 85%+ coverage per module
✓ Query builder testable without database

---

## Next Steps

1. Review design doc (SQLITE_ADAPTER_DECOMPOSITION_DESIGN.md)
2. Follow roadmap (SQLITE_IMPLEMENTATION_ROADMAP.md)
3. Implement using blueprint (SQLITE_MODULE_BLUEPRINT.md)
4. Verify backwards compatibility
5. Merge with full test coverage
