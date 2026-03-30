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

### Module Breakdown

#### 1. `store/sync.rs` — Sync Logic (~400 LOC)

**Responsibility:** Connection pooling, CRUD operations, row synchronization

```rust
pub trait SyncStore<T>: Send + Sync {
    type Connection: Send + Sync;

    /// Establish connection pool
    fn new(config: &ConnectionConfig) -> Result<Self>;

    /// Execute read in transaction
    async fn read_tx<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Self::Connection) -> Result<R> + Send;

    /// Execute write in transaction
    async fn write_tx<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Self::Connection) -> Result<R> + Send;

    /// Bulk insert with atomicity
    async fn bulk_insert(&self, records: Vec<T>) -> Result<usize>;

    /// Stream results without loading all in memory
    async fn stream<F>(&self, sql: &str, f: F) -> Result<()>
    where
        F: FnMut(T) -> Result<()> + Send;
}
```

**Contents:**
- `ConnectionPool` struct with metrics
- `SyncContext` for transaction handling
- `RowMapper` trait for type conversion
- `BulkInsertStrategy` for batch operations
- Retry logic for transient failures

**Test Strategy:**
- Unit tests: use in-memory database (`:memory:`)
- No external fixtures required
- Mock `RowMapper` for type conversion testing

---

#### 2. `store/query_builder.rs` — Query Builder (~300 LOC)

**Responsibility:** Dynamic SQL construction, type-safe query composition

```rust
pub trait QueryBuilder: Send + Sync + Sized {
    type FilterExpr;
    type JoinExpr;

    /// SELECT with columns
    fn select(columns: &[&str]) -> Self;

    /// FROM table
    fn from(table: &str) -> Self;

    /// WHERE conditions (fluent)
    fn where_clause(self, filter: Self::FilterExpr) -> Self;

    /// INNER/LEFT JOIN
    fn join(self, join: Self::JoinExpr) -> Self;

    /// ORDER BY with direction
    fn order_by(self, column: &str, asc: bool) -> Self;

    /// LIMIT and OFFSET
    fn paginate(self, limit: usize, offset: usize) -> Self;

    /// Build final SQL
    fn build(self) -> (String, Vec<SqlValue>);
}

pub struct SqliteQueryBuilder { /* ... */ }
pub struct Filter { /* ... */ }
pub struct Join { /* ... */ }
```

**Contents:**
- `SqliteQueryBuilder` fluent API
- `Filter` combinators (AND, OR, NOT)
- `Join` specification (INNER, LEFT, RIGHT, CROSS)
- `Aggregate` for COUNT/SUM/AVG/MAX
- `Parameterized` values for SQL injection prevention
- AST validation before building SQL

**Test Strategy:**
- No database required — pure SQL string tests
- Property-based tests (proptest) for SQL validity
- Snapshot tests for complex queries
- Edge cases: empty filters, null handling, escaping

---

#### 3. `store/migrations.rs` — Migration Runner (~250 LOC)

**Responsibility:** Schema versioning, migration execution, rollback

```rust
pub trait MigrationRunner: Send + Sync {
    /// Register a migration
    fn add_migration(&mut self, migration: Box<dyn Migration>) -> Result<()>;

    /// Apply all pending migrations
    async fn migrate(&self, target_version: Option<i32>) -> Result<MigrationStatus>;

    /// Rollback to previous version
    async fn rollback(&self, steps: usize) -> Result<MigrationStatus>;

    /// Get current schema version
    async fn current_version(&self) -> Result<i32>;

    /// Verify schema integrity
    async fn verify(&self) -> Result<SchemaVerification>;
}

pub trait Migration: Send + Sync {
    fn version(&self) -> i32;
    fn name(&self) -> &str;
    async fn up(&self, conn: &mut Connection) -> Result<()>;
    async fn down(&self, conn: &mut Connection) -> Result<()>;
}
```

**Contents:**
- `SqliteMigrationRunner` with version tracking
- `MigrationRegistry` for ordered execution
- `SchemaValidator` for integrity checks
- `MigrationState` for transaction safety
- Checksum validation (prevent tampering)
- Dry-run capability

**Test Strategy:**
- Isolated in-memory database per test
- Fixture migrations (up/down pairs)
- State machine tests (verify valid transitions)
- Rollback verification
- Schema validation tests

---

## Trait Extraction Strategy

### Public Port Traits (→ `phenotype-contracts`)

These traits should be extracted to a contracts crate for cross-crate reuse:

```rust
// phenotype-contracts/src/ports/storage/mod.rs

pub trait Repository<T: Send + Sync>: Send + Sync {
    type Error: std::error::Error;

    async fn create(&self, entity: T) -> Result<String, Self::Error>;
    async fn read(&self, id: &str) -> Result<Option<T>, Self::Error>;
    async fn update(&self, id: &str, entity: T) -> Result<(), Self::Error>;
    async fn delete(&self, id: &str) -> Result<bool, Self::Error>;
    async fn list(&self, filter: Filter) -> Result<Vec<T>, Self::Error>;
}

pub trait QueryBuilder: Send + Sync + Sized {
    fn select(columns: &[&str]) -> Self;
    fn from(table: &str) -> Self;
    fn where_clause(self, filter: Filter) -> Self;
    fn build(self) -> (String, Vec<Value>);
}

pub trait MigrationRunner: Send + Sync {
    async fn migrate(&self, target: Option<i32>) -> Result<()>;
    async fn rollback(&self, steps: usize) -> Result<()>;
}
```

### Backward Compatibility

**Public API remains unchanged:**

```rust
// lib.rs continues to re-export everything
pub use store::sync::SyncStore;
pub use store::query_builder::QueryBuilder;
pub use store::migrations::MigrationRunner;

// Concrete implementation
pub struct SqliteRepository { /* ... */ }

impl<T> Repository<T> for SqliteRepository {
    // Delegates to internal modules
}
```

Existing code continues working:
```rust
// Old code still compiles
let repo = SqliteRepository::new(config)?;
repo.create(entity).await?;
```

---

## Module Blueprints

### `store/sync.rs` Function Signatures

```rust
pub mod sync {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use rusqlite::Connection;

    pub struct ConnectionPool {
        connections: Arc<Vec<Mutex<Connection>>>,
        config: ConnectionConfig,
        metrics: Arc<SyncMetrics>,
    }

    pub struct ConnectionConfig {
        pub path: String,
        pub pool_size: usize,
        pub timeout: Duration,
        pub flags: OpenFlags,
    }

    pub struct SyncMetrics {
        reads: AtomicU64,
        writes: AtomicU64,
        errors: AtomicU64,
        avg_latency_ms: AtomicU64,
    }

    pub trait RowMapper<T> {
        fn map_row(row: &Row) -> Result<T>;
    }

    impl ConnectionPool {
        pub fn new(config: ConnectionConfig) -> Result<Self> { /* ... */ }

        pub async fn read_tx<F, R>(&self, f: F) -> Result<R>
        where
            F: FnOnce(&Connection) -> Result<R> + Send,
        { /* ... */ }

        pub async fn write_tx<F, R>(&self, f: F) -> Result<R>
        where
            F: FnOnce(&mut Connection) -> Result<R> + Send,
        { /* ... */ }

        pub async fn bulk_insert<T: RowMapper>(
            &self,
            records: Vec<T>,
        ) -> Result<usize> { /* ... */ }

        pub async fn stream<F>(&self, sql: &str, mut f: F) -> Result<()>
        where
            F: FnMut(&Row) -> Result<()> + Send,
        { /* ... */ }

        pub fn metrics(&self) -> &SyncMetrics { /* ... */ }
    }
}
```

### `store/query_builder.rs` Function Signatures

```rust
pub mod query_builder {
    use std::fmt;

    #[derive(Clone, Debug)]
    pub enum Operator {
        Eq, Ne, Gt, Gte, Lt, Lte,
        In, NotIn, Like, Between,
        IsNull, IsNotNull,
    }

    #[derive(Clone, Debug)]
    pub struct Filter {
        column: String,
        operator: Operator,
        value: Option<SqlValue>,
        logic: Logic, // AND or OR
    }

    #[derive(Clone, Debug)]
    pub enum Logic { And, Or }

    #[derive(Clone, Debug)]
    pub enum JoinType { Inner, Left, Right, Cross }

    #[derive(Clone, Debug)]
    pub struct Join {
        join_type: JoinType,
        table: String,
        on: Option<String>,
    }

    pub struct SqliteQueryBuilder {
        columns: Vec<String>,
        from_table: Option<String>,
        filters: Vec<Filter>,
        joins: Vec<Join>,
        order_by: Vec<(String, bool)>, // (column, asc)
        limit: Option<usize>,
        offset: Option<usize>,
    }

    impl SqliteQueryBuilder {
        pub fn select(columns: &[&str]) -> Self { /* ... */ }
        pub fn from(table: &str) -> Self { /* ... */ }
        pub fn where_filter(self, filter: Filter) -> Self { /* ... */ }
        pub fn and(self, filter: Filter) -> Self { /* ... */ }
        pub fn or(self, filter: Filter) -> Self { /* ... */ }
        pub fn join(self, join: Join) -> Self { /* ... */ }
        pub fn order_by(self, column: &str, asc: bool) -> Self { /* ... */ }
        pub fn limit(self, limit: usize) -> Self { /* ... */ }
        pub fn offset(self, offset: usize) -> Self { /* ... */ }
        pub fn build(self) -> Result<(String, Vec<SqlValue>)> { /* ... */ }

        fn validate(&self) -> Result<()> { /* ... */ }
    }

    impl Filter {
        pub fn eq(column: &str, value: SqlValue) -> Self { /* ... */ }
        pub fn ne(column: &str, value: SqlValue) -> Self { /* ... */ }
        pub fn gt(column: &str, value: SqlValue) -> Self { /* ... */ }
        pub fn in_list(column: &str, values: Vec<SqlValue>) -> Self { /* ... */ }
        pub fn between(column: &str, min: SqlValue, max: SqlValue) -> Self { /* ... */ }
        pub fn like(column: &str, pattern: &str) -> Self { /* ... */ }
    }

    impl Join {
        pub fn inner(table: &str, on: &str) -> Self { /* ... */ }
        pub fn left(table: &str, on: &str) -> Self { /* ... */ }
    }
}
```

### `store/migrations.rs` Function Signatures

```rust
pub mod migrations {
    use std::collections::BTreeMap;

    pub struct MigrationVersion {
        pub version: i32,
        pub name: String,
        pub checksum: String,
        pub applied_at: SystemTime,
    }

    pub struct SchemaVerification {
        pub tables: Vec<String>,
        pub indexes: Vec<String>,
        pub constraints: Vec<String>,
        pub integrity_ok: bool,
    }

    pub struct MigrationStatus {
        pub current_version: i32,
        pub previous_version: i32,
        pub applied: Vec<MigrationVersion>,
        pub duration: Duration,
    }

    pub trait Migration: Send + Sync {
        fn version(&self) -> i32;
        fn name(&self) -> &str;
        fn checksum(&self) -> String;
        async fn up(&self, conn: &mut Connection) -> Result<()>;
        async fn down(&self, conn: &mut Connection) -> Result<()>;
    }

    pub struct SqliteMigrationRunner {
        migrations: BTreeMap<i32, Box<dyn Migration>>,
        pool: Arc<ConnectionPool>,
    }

    impl SqliteMigrationRunner {
        pub fn new(pool: Arc<ConnectionPool>) -> Self { /* ... */ }

        pub fn add_migration(&mut self, migration: Box<dyn Migration>) -> Result<()> { /* ... */ }

        pub async fn migrate(
            &self,
            target_version: Option<i32>,
        ) -> Result<MigrationStatus> { /* ... */ }

        pub async fn rollback(
            &self,
            steps: usize,
        ) -> Result<MigrationStatus> { /* ... */ }

        pub async fn current_version(&self) -> Result<i32> { /* ... */ }

        pub async fn verify(&self) -> Result<SchemaVerification> { /* ... */ }

        async fn execute_migration(&self, m: &dyn Migration) -> Result<()> { /* ... */ }

        async fn track_migration(&self, m: &dyn Migration) -> Result<()> { /* ... */ }
    }

    struct MigrationHistory {
        version: i32,
        name: String,
        checksum: String,
        applied_at: SystemTime,
    }
}
```

---

## Test Isolation Strategy

### Unit Tests (No Database Required)

#### 1. Query Builder Tests
```rust
// tests/store/query_builder.rs
#[test]
fn test_simple_select() {
    let builder = SqliteQueryBuilder::select(&["id", "name"])
        .from("users");
    let (sql, params) = builder.build().unwrap();
    assert_eq!(sql, "SELECT id, name FROM users");
    assert!(params.is_empty());
}

#[test]
fn test_where_with_and() {
    let builder = SqliteQueryBuilder::select(&["*"])
        .from("users")
        .where_filter(Filter::eq("status", "active".into()))
        .and(Filter::gt("age", "18".into()));
    let (sql, params) = builder.build().unwrap();
    assert!(sql.contains("WHERE"));
    assert_eq!(params.len(), 2);
}

// Property-based test
#[test]
fn test_sql_injection_prevention() {
    // Filter always parameterizes values
    let filter = Filter::like("name", "'; DROP TABLE users; --");
    // Value is never concatenated into SQL
}
```

#### 2. Migration Tests
```rust
// tests/store/migrations.rs
#[tokio::test]
async fn test_migration_ordering() {
    let mut runner = create_test_runner();
    runner.add_migration(box MockMigration::new(1, "init")).ok();
    runner.add_migration(box MockMigration::new(2, "add_users")).ok();

    // Verify version order is preserved
    let versions: Vec<_> = runner.migrations.keys().copied().collect();
    assert_eq!(versions, vec![1, 2]);
}

#[tokio::test]
async fn test_rollback_state_machine() {
    // Use in-memory :memory: database
    let conn = Connection::open(":memory:").unwrap();
    let runner = create_test_runner_with_db(&conn);

    runner.migrate(Some(2)).await.ok();
    runner.rollback(1).await.ok();

    let version = runner.current_version().await.unwrap();
    assert_eq!(version, 1);
}
```

### Integration Tests (With Database)

#### 3. Sync Store Tests
```rust
// tests/store/sync.rs
#[tokio::test]
async fn test_connection_pool_isolation() {
    let pool = create_test_pool();

    // Read should not block writes
    let h1 = tokio::spawn({
        let pool = pool.clone();
        async move {
            pool.read_tx(|conn| {
                // Simulate slow read
                std::thread::sleep(Duration::from_millis(100));
                Ok(42)
            }).await
        }
    });

    let h2 = tokio::spawn({
        let pool = pool.clone();
        async move {
            pool.write_tx(|conn| {
                // Write should not be blocked
                Ok(())
            }).await
        }
    });

    tokio::time::timeout(Duration::from_secs(1), async {
        h1.await.ok();
        h2.await.ok();
    }).await.ok();
}

#[tokio::test]
async fn test_bulk_insert_atomicity() {
    let pool = create_test_pool();

    let records = vec![record(1), record(2), record(3)];
    let count = pool.bulk_insert(records).await.unwrap();

    assert_eq!(count, 3);

    // Verify all records exist
    let total = pool.read_tx(|conn| {
        conn.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0))
    }).await.unwrap();
    assert_eq!(total, 3);
}
```

### Fixture Strategy

```rust
// tests/fixtures.rs
fn create_test_pool() -> Arc<ConnectionPool> {
    let config = ConnectionConfig {
        path: ":memory:".to_string(),
        pool_size: 2,
        timeout: Duration::from_secs(5),
        flags: OpenFlags::default(),
    };
    Arc::new(ConnectionPool::new(config).unwrap())
}

struct MockMigration {
    version: i32,
    name: String,
}

#[async_trait]
impl Migration for MockMigration {
    fn version(&self) -> i32 { self.version }
    fn name(&self) -> &str { &self.name }
    async fn up(&self, _conn: &mut Connection) -> Result<()> { Ok(()) }
    async fn down(&self, _conn: &mut Connection) -> Result<()> { Ok(()) }
}
```

---

## Backwards Compatibility Guarantee

### Public API Surface (No Breaking Changes)

```rust
// lib.rs — public exports remain stable

pub use store::sync::{
    ConnectionPool,
    ConnectionConfig,
    SyncStore,
    RowMapper,
};

pub use store::query_builder::{
    SqliteQueryBuilder,
    QueryBuilder,
    Filter,
    Join,
};

pub use store::migrations::{
    SqliteMigrationRunner,
    MigrationRunner,
    Migration,
};

// Concrete struct continues to implement all traits
pub struct SqliteRepository<T> {
    pool: Arc<ConnectionPool>,
    _phantom: PhantomData<T>,
}

impl<T> Repository<T> for SqliteRepository<T>
where
    T: Send + Sync + RowMapper,
{
    // Delegates to internal modules
    async fn create(&self, entity: T) -> Result<String> {
        self.pool.write_tx(|conn| {
            // Uses query builder internally
            let (sql, params) = QueryBuilder::for_insert::<T>();
            // Executes via sync store
            conn.execute(&sql, params)
        }).await
    }
}

// All existing usage continues to work:
let repo = SqliteRepository::new(config)?;
let id = repo.create(entity).await?;
let entity = repo.read(&id).await?;
```

### Migration Path for Users

**No action required** — all existing code compiles without changes.

**Optional optimization** — users can now test components independently:

```rust
// Before: Must test with full database
#[test]
fn test_data_flow() {
    let db = create_test_db();
    // ...
}

// After: Test query builder without database
#[test]
fn test_query_generation() {
    let builder = SqliteQueryBuilder::select(&["*"])
        .from("users")
        .where_filter(Filter::eq("id", "123".into()));
    let (sql, _) = builder.build().unwrap();
    assert!(sql.contains("WHERE"));
}
```

---

## Commit Sequence (Atomic Steps)

### Phase 1: Create Module Structure

**Commit 1a:** Create module files with `lib.rs` re-exports
```bash
cargo new --lib crates/agileplus-sqlite
mkdir -p crates/agileplus-sqlite/src/store
touch crates/agileplus-sqlite/src/store/{sync,query_builder,migrations}.rs
```

**Commit 1b:** Extract sync logic (400 LOC)
- Move connection pool, transactions, row mapping
- All tests pass (existing tests use new module)
- Zero LOC change in public API

**Commit 1c:** Extract query builder (300 LOC)
- Move SQL construction, filters, joins
- Add unit tests (no database required)
- Existing queries continue to work

**Commit 1d:** Extract migrations (250 LOC)
- Move schema versioning, migration runner
- Add state machine tests
- Existing migrations unchanged

### Phase 2: Add Contract Traits

**Commit 2a:** Define trait contracts
- `SyncStore` trait in `store/sync.rs`
- `QueryBuilder` trait in `store/query_builder.rs`
- `MigrationRunner` trait in `store/migrations.rs`

**Commit 2b:** Implement traits on concrete types
- `SqliteRepository` implements all three
- Tests verify implementation correctness
- Public API surface frozen

### Phase 3: Extract to phenotype-contracts

**Commit 3a:** Move trait definitions
```bash
# In phenotype-contracts crate
# phenotype-contracts/src/ports/storage/mod.rs
pub trait SyncStore<T> { /* ... */ }
pub trait QueryBuilder { /* ... */ }
pub trait MigrationRunner { /* ... */ }
```

**Commit 3b:** Re-export from sqlite crate
```rust
// agileplus-sqlite/src/lib.rs
pub use phenotype_contracts::ports::storage::{
    SyncStore,
    QueryBuilder,
    MigrationRunner,
};
```

### Phase 4: Add Tests

**Commit 4a:** Unit tests (query builder)
- 15+ tests for SQL generation
- Property-based tests for SQL injection
- ~200 LOC tests

**Commit 4b:** Integration tests (sync + migrations)
- Connection pool isolation tests
- Transaction atomicity tests
- Migration state machine tests
- ~300 LOC tests

**Commit 4c:** Backwards compatibility tests
- All existing queries still compile
- All existing migrations still run
- ~100 LOC tests

### Phase 5: Documentation & Cleanup

**Commit 5a:** Add module documentation
- Examples for each module
- Trait usage guide
- ~150 LOC docs

**Commit 5b:** Clean up old monolithic code
- Remove duplicate logic from lib.rs
- Final lib.rs: ~200 LOC public API
- Verify zero regressions

---

## Trait Hierarchy Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                   phenotype-contracts                             │
│           (Reusable trait definitions)                            │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐            │
│  │  SyncStore   │  │QueryBuilder  │  │MigrationRun  │            │
│  │   <T>        │  │              │  │   ner        │            │
│  │              │  │              │  │              │            │
│  │+ read_tx()   │  │+ select()    │  │+ migrate()   │            │
│  │+ write_tx()  │  │+ where()     │  │+ rollback()  │            │
│  │+ bulk_ins()  │  │+ join()      │  │+ verify()    │            │
│  │+ stream()    │  │+ order_by()  │  │              │            │
│  │              │  │+ build()     │  │              │            │
│  └──────────────┘  └──────────────┘  └──────────────┘            │
│         ▲                  ▲                  ▲                   │
│         │                  │                  │                   │
│         └──────────────────┴──────────────────┘                   │
│                      (impl)                                        │
│                                                                   │
├──────────────────────────────────────────────────────────────────┤
│                   agileplus-sqlite                                │
│        (Concrete SQLite implementations)                         │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │           SqliteRepository<T>                            │   │
│  │   (Implements all three traits)                          │   │
│  │                                                           │   │
│  │ ┌─────────────────────────────────────────────────────┐  │   │
│  │ │            pub struct ConnectionPool               │  │   │
│  │ │   (impl SyncStore via delegate pattern)            │  │   │
│  │ │                                                     │  │   │
│  │ │ ┌──────────────────────────────────────────────┐   │  │   │
│  │ │ │ store/sync.rs (~400 LOC)                    │   │  │   │
│  │ │ │ - Connection pool mgmt (70 LOC)             │   │  │   │
│  │ │ │ - Transaction context (80 LOC)              │   │  │   │
│  │ │ │ - Row mapping (120 LOC)                     │   │  │   │
│  │ │ │ - Bulk insert strategy (90 LOC)             │   │  │   │
│  │ │ │ - Metrics & monitoring (40 LOC)             │   │  │   │
│  │ │ └──────────────────────────────────────────────┘   │  │   │
│  │ └─────────────────────────────────────────────────────┘  │   │
│  │                                                           │   │
│  │ ┌─────────────────────────────────────────────────────┐  │   │
│  │ │        store/query_builder.rs (~300 LOC)           │  │   │
│  │ │   (impl QueryBuilder)                              │  │   │
│  │ │                                                     │  │   │
│  │ │ ┌──────────────────────────────────────────────┐   │  │   │
│  │ │ │ - SqliteQueryBuilder (120 LOC)              │   │  │   │
│  │ │ │ - Filter combinators (80 LOC)               │   │  │   │
│  │ │ │ - JOIN specification (60 LOC)               │   │  │   │
│  │ │ │ - Aggregate functions (40 LOC)              │   │  │   │
│  │ │ └──────────────────────────────────────────────┘   │  │   │
│  │ └─────────────────────────────────────────────────────┘  │   │
│  │                                                           │   │
│  │ ┌─────────────────────────────────────────────────────┐  │   │
│  │ │     store/migrations.rs (~250 LOC)                │  │   │
│  │ │   (impl MigrationRunner)                          │  │   │
│  │ │                                                     │  │   │
│  │ │ ┌──────────────────────────────────────────────┐   │  │   │
│  │ │ │ - SqliteMigrationRunner (100 LOC)           │   │  │   │
│  │ │ │ - Migration tracking (80 LOC)               │   │  │   │
│  │ │ │ - Rollback logic (50 LOC)                   │   │  │   │
│  │ │ │ - Schema validation (20 LOC)                │   │  │   │
│  │ │ └──────────────────────────────────────────────┘   │  │   │
│  │ └─────────────────────────────────────────────────────┘  │   │
│  │                                                           │   │
│  │ ┌─────────────────────────────────────────────────────┐  │   │
│  │ │            lib.rs (~200 LOC)                       │  │   │
│  │ │   - Re-exports (public API)                        │  │   │
│  │ │   - Error types                                    │  │   │
│  │ │   - Main Repository facade                        │  │   │
│  │ │   - Utility functions                             │  │   │
│  │ └─────────────────────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
```

---

## Success Criteria

### Structural Metrics
- [ ] lib.rs reduced from 1,582 LOC → ~200 LOC (87% reduction)
- [ ] 3 focused modules created: sync (400), query_builder (300), migrations (250)
- [ ] All traits extracted and documented
- [ ] Zero breaking changes to public API

### Test Coverage
- [ ] Query builder: 15+ unit tests (no database)
- [ ] Sync store: 8+ integration tests
- [ ] Migrations: 10+ state machine tests
- [ ] Backwards compatibility: 5+ regression tests
- [ ] Overall coverage: ≥85% for decomposed modules

### Code Quality
- [ ] All modules ≤500 LOC
- [ ] Cyclomatic complexity ≤8 per function
- [ ] All public items documented with examples
- [ ] Zero `#[allow(dead_code)]` suppressions
- [ ] All clippy warnings resolved

### Performance
- [ ] No performance regression in CRUD operations
- [ ] Connection pool scales linearly with pool_size
- [ ] Query builder generates optimal SQL (verified via benchmarks)

---

## Risk Mitigation

| Risk | Probability | Mitigation |
|------|------------|------------|
| Breaking existing code | Low | Comprehensive backwards compat tests before merge |
| Performance regression | Low | Benchmark suite before/after |
| Migration bugs | Medium | Dry-run mode + state machine verification |
| SQL injection in builder | Low | Always parameterize values + fuzz testing |
| Connection pool deadlock | Medium | Timeout on all acquisitions, test isolation |

---

## Conclusion

This decomposition design transforms a 1,582 LOC monolithic adapter into three focused, independently testable modules while maintaining 100% backwards compatibility. The trait hierarchy enables:

1. **Reusability** — Query builder useful for other backends
2. **Testability** — Unit test query builder without database
3. **Maintainability** — Each module ≤500 LOC, single responsibility
4. **Composability** — Swap implementations (e.g., PostgreSQL builder)
5. **Evolution** — Add new adapters by implementing traits

**Implementation Timeline:** 5 atomic phases, ~20 min wall-clock per phase = 2 hours total for complete decomposition with full test coverage.
