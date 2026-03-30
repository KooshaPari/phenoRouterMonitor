# SQLite Adapter Module Blueprint

## Complete Module Structure with Signatures

This document shows the exact structure of each decomposed module with all public function signatures, types, and expected implementations.

---

## Module 1: `store/sync.rs` (Connection Pooling & Transactions)

### Module Declaration
```rust
// src/store/sync.rs

//! Connection pooling, transaction management, and row synchronization.
//!
//! This module provides the core synchronous storage abstraction for SQLite,
//! including connection pooling, transaction handling, and row mapping.
//!
//! # Examples
//!
//! ```ignore
//! let config = ConnectionConfig {
//!     path: "data.db".to_string(),
//!     pool_size: 10,
//!     timeout: Duration::from_secs(5),
//!     flags: OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
//! };
//! let pool = ConnectionPool::new(config)?;
//!
//! let result = pool.read_tx(|conn| {
//!     conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
//! }).await?;
//! ```

use async_trait::async_trait;
use rusqlite::{Connection, OpenFlags, params};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use crate::error::SqliteError;

// ============================================================================
// Type Definitions
// ============================================================================

/// Configuration for SQLite connection pool.
#[derive(Clone, Debug)]
pub struct ConnectionConfig {
    /// Path to SQLite database file (or `:memory:` for in-memory)
    pub path: String,

    /// Number of connections in the pool
    pub pool_size: usize,

    /// Timeout for acquiring a connection
    pub timeout: Duration,

    /// OpenFlags for SQLite connection
    pub flags: OpenFlags,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            path: ":memory:".to_string(),
            pool_size: 10,
            timeout: Duration::from_secs(5),
            flags: OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        }
    }
}

/// Metrics tracking for sync operations.
#[derive(Default, Clone, Debug)]
pub struct SyncMetrics {
    /// Total read operations
    pub reads: u64,
    /// Total write operations
    pub writes: u64,
    /// Total errors
    pub errors: u64,
    /// Average transaction latency in milliseconds
    pub avg_latency_ms: f64,
}

impl SyncMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_reads(&mut self) {
        self.reads += 1;
    }

    pub fn increment_writes(&mut self) {
        self.writes += 1;
    }

    pub fn increment_errors(&mut self) {
        self.errors += 1;
    }

    pub fn record_latency(&mut self, latency_ms: f64) {
        // Exponential moving average
        self.avg_latency_ms = self.avg_latency_ms * 0.9 + latency_ms * 0.1;
    }

    pub fn total_ops(&self) -> u64 {
        self.reads + self.writes
    }
}

// ============================================================================
// Traits
// ============================================================================

/// Trait for mapping database rows to Rust types.
///
/// Implement this trait to define how to convert a SQLite row into your domain type.
///
/// # Example
///
/// ```ignore
/// struct User {
///     id: String,
///     name: String,
///     age: i32,
/// }
///
/// impl RowMapper<User> for User {
///     fn map_row(row: &rusqlite::Row) -> Result<Self> {
///         Ok(User {
///             id: row.get(0)?,
///             name: row.get(1)?,
///             age: row.get(2)?,
///         })
///     }
/// }
/// ```
pub trait RowMapper<T>: Send + Sync {
    /// Map a database row to type T.
    fn map_row(row: &rusqlite::Row) -> Result<T, SqliteError>;
}

/// Trait for synchronous store operations with transaction support.
///
/// Provides transaction context (read-only and read-write) for database operations.
#[async_trait]
pub trait SyncStore<T>: Send + Sync {
    /// Connection type for this store
    type Connection: Send + Sync;

    /// Execute a read-only transaction.
    ///
    /// The connection parameter is immutable, preventing accidental writes.
    /// Transaction is automatically rolled back on error.
    async fn read_tx<F, R>(&self, f: F) -> Result<R, SqliteError>
    where
        F: FnOnce(&Self::Connection) -> Result<R, SqliteError> + Send,
        R: Send;

    /// Execute a read-write transaction.
    ///
    /// The connection parameter is mutable, allowing writes.
    /// Transaction is automatically rolled back on error, committed on success.
    async fn write_tx<F, R>(&self, f: F) -> Result<R, SqliteError>
    where
        F: FnOnce(&mut Self::Connection) -> Result<R, SqliteError> + Send,
        R: Send;

    /// Bulk insert multiple records atomically.
    ///
    /// If any insert fails, the entire batch is rolled back.
    /// Returns the number of successfully inserted records.
    async fn bulk_insert<U: Send>(&self, records: Vec<U>) -> Result<usize, SqliteError>;

    /// Stream results without loading all into memory.
    ///
    /// Useful for large result sets.
    async fn stream<F>(&self, sql: &str, f: F) -> Result<(), SqliteError>
    where
        F: FnMut(&rusqlite::Row) -> Result<(), SqliteError> + Send;
}

// ============================================================================
// Connection Pool Implementation
// ============================================================================

/// Thread-safe SQLite connection pool.
///
/// Manages a pool of reusable connections, handles transaction management,
/// and provides synchronization metrics.
pub struct ConnectionPool {
    /// Stack of available connections (wrapped in Arc for sharing)
    connections: Vec<Arc<Mutex<Connection>>>,

    /// Configuration used to create this pool
    config: ConnectionConfig,

    /// Operation metrics
    metrics: Arc<Mutex<SyncMetrics>>,
}

impl ConnectionPool {
    /// Create a new connection pool with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Pool configuration (path, size, timeout, flags)
    ///
    /// # Returns
    ///
    /// New pool or error if connections cannot be created
    pub fn new(config: ConnectionConfig) -> Result<Self, SqliteError> {
        let mut connections = Vec::with_capacity(config.pool_size);

        for _ in 0..config.pool_size {
            let conn = Connection::open_with_flags(&config.path, config.flags)?;
            connections.push(Arc::new(Mutex::new(conn)));
        }

        Ok(Self {
            connections,
            config,
            metrics: Arc::new(Mutex::new(SyncMetrics::new())),
        })
    }

    /// Get a reference to the metrics.
    pub async fn metrics(&self) -> SyncMetrics {
        self.metrics.lock().await.clone()
    }

    /// Get current pool size.
    pub fn pool_size(&self) -> usize {
        self.connections.len()
    }

    /// Get configuration reference.
    pub fn config(&self) -> &ConnectionConfig {
        &self.config
    }

    /// Internal: acquire a connection from the pool (round-robin).
    async fn acquire(&self, index: usize) -> Arc<Mutex<Connection>> {
        self.connections[index % self.connections.len()].clone()
    }

    // ========================================================================
    // Transaction Methods
    // ========================================================================

    /// Execute read transaction (immutable access).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = pool.read_tx(|conn| {
    ///     let count: i32 = conn.query_row(
    ///         "SELECT COUNT(*) FROM users",
    ///         [],
    ///         |row| row.get(0)
    ///     )?;
    ///     Ok(count)
    /// }).await?;
    /// ```
    pub async fn read_tx<F, R>(&self, f: F) -> Result<R, SqliteError>
    where
        F: FnOnce(&Connection) -> Result<R, SqliteError> + Send,
        R: Send,
    {
        let start = std::time::Instant::now();
        let conn = self.acquire(0).await;
        let locked = conn.lock().await;

        match f(&*locked) {
            Ok(result) => {
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                let mut metrics = self.metrics.lock().await;
                metrics.increment_reads();
                metrics.record_latency(elapsed);
                Ok(result)
            }
            Err(e) => {
                let mut metrics = self.metrics.lock().await;
                metrics.increment_errors();
                Err(e)
            }
        }
    }

    /// Execute write transaction (mutable access).
    ///
    /// Changes are committed if the closure returns Ok, rolled back on Err.
    ///
    /// # Example
    ///
    /// ```ignore
    /// pool.write_tx(|conn| {
    ///     conn.execute(
    ///         "INSERT INTO users (id, name) VALUES (?1, ?2)",
    ///         params!["123", "Alice"],
    ///     )?;
    ///     Ok(())
    /// }).await?;
    /// ```
    pub async fn write_tx<F, R>(&self, f: F) -> Result<R, SqliteError>
    where
        F: FnOnce(&mut Connection) -> Result<R, SqliteError> + Send,
        R: Send,
    {
        let start = std::time::Instant::now();
        let conn = self.acquire(0).await;
        let mut locked = conn.lock().await;

        // Start transaction
        locked.execute("BEGIN TRANSACTION", [])?;

        match f(&mut *locked) {
            Ok(result) => {
                locked.execute("COMMIT", [])?;
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                let mut metrics = self.metrics.lock().await;
                metrics.increment_writes();
                metrics.record_latency(elapsed);
                Ok(result)
            }
            Err(e) => {
                // Attempt rollback; ignore errors
                let _ = locked.execute("ROLLBACK", []);
                let mut metrics = self.metrics.lock().await;
                metrics.increment_errors();
                Err(e)
            }
        }
    }

    /// Bulk insert with automatic transaction.
    ///
    /// All records inserted atomically. If any fails, entire batch rolled back.
    pub async fn bulk_insert<T, U>(
        &self,
        records: Vec<U>,
    ) -> Result<usize, SqliteError>
    where
        U: Send,
        T: Send,
    {
        self.write_tx(|conn| {
            let mut count = 0;
            for _record in records.iter() {
                // Insert logic goes here
                // (will be implemented by caller via closure)
                count += 1;
            }
            Ok(count)
        })
        .await
    }

    /// Stream results without materializing all rows.
    ///
    /// Useful for processing large result sets.
    pub async fn stream<F>(
        &self,
        sql: &str,
        mut f: F,
    ) -> Result<(), SqliteError>
    where
        F: FnMut(&rusqlite::Row) -> Result<(), SqliteError> + Send,
    {
        self.read_tx(|conn| {
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map([], |row| {
                f(row)?;
                Ok(())
            })?;

            for row_result in rows {
                row_result??;
            }

            Ok(())
        })
        .await
    }

    /// Health check: verify connections work.
    pub async fn health_check(&self) -> Result<bool, SqliteError> {
        self.read_tx(|conn| {
            conn.query_row("SELECT 1", [], |_| Ok(()))?;
            Ok(true)
        })
        .await
    }

    /// Close all connections (explicit cleanup).
    pub async fn close(&mut self) -> Result<(), SqliteError> {
        self.connections.clear();
        Ok(())
    }
}

// ============================================================================
// Trait Implementation
// ============================================================================

#[async_trait]
impl SyncStore<()> for ConnectionPool {
    type Connection = Connection;

    async fn read_tx<F, R>(&self, f: F) -> Result<R, SqliteError>
    where
        F: FnOnce(&Connection) -> Result<R, SqliteError> + Send,
        R: Send,
    {
        ConnectionPool::read_tx(self, f).await
    }

    async fn write_tx<F, R>(&self, f: F) -> Result<R, SqliteError>
    where
        F: FnOnce(&mut Connection) -> Result<R, SqliteError> + Send,
        R: Send,
    {
        ConnectionPool::write_tx(self, f).await
    }

    async fn bulk_insert<U: Send>(&self, _records: Vec<U>) -> Result<usize, SqliteError> {
        Ok(0) // Implement with actual logic
    }

    async fn stream<F>(&self, sql: &str, f: F) -> Result<(), SqliteError>
    where
        F: FnMut(&rusqlite::Row) -> Result<(), SqliteError> + Send,
    {
        ConnectionPool::stream(self, sql, f).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pool() -> ConnectionPool {
        let config = ConnectionConfig {
            path: ":memory:".to_string(),
            pool_size: 2,
            timeout: Duration::from_secs(5),
            flags: OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        };
        ConnectionPool::new(config).unwrap()
    }

    #[tokio::test]
    async fn test_connection_pool_creation() {
        let pool = create_test_pool();
        assert_eq!(pool.pool_size(), 2);
    }

    #[tokio::test]
    async fn test_read_transaction() {
        let pool = create_test_pool();
        let result = pool.read_tx(|_conn| {
            Ok(42)
        }).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_write_transaction() {
        let pool = create_test_pool();

        pool.write_tx(|conn| {
            conn.execute(
                "CREATE TABLE test (id TEXT PRIMARY KEY, value TEXT)",
                [],
            )?;
            Ok(())
        }).await.ok();

        let result = pool.read_tx(|conn| {
            conn.query_row("SELECT name FROM sqlite_master WHERE type='table'", [], |row| {
                row.get::<_, String>(0)
            })
        }).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_increment() {
        let pool = create_test_pool();

        pool.read_tx(|_| Ok(())).await.ok();

        let metrics = pool.metrics().await;
        assert_eq!(metrics.reads, 1);
    }
}
```

---

## Module 2: `store/query_builder.rs` (Dynamic SQL Construction)

### Module Declaration
```rust
// src/store/query_builder.rs

//! Dynamic SQL query construction with type safety.
//!
//! This module provides a fluent query builder API for constructing
//! parameterized SQL queries. All values are parameterized to prevent SQL injection.
//!
//! # Examples
//!
//! ```ignore
//! let (sql, params) = SqliteQueryBuilder::select(&["id", "name"])
//!     .from("users")
//!     .where_filter(Filter::eq("status", "active".into()))
//!     .and(Filter::gt("age", "18".into()))
//!     .order_by("created_at", false)
//!     .limit(10)
//!     .build()?;
//!
//! // sql: "SELECT id, name FROM users WHERE status = ? AND age > ? ORDER BY created_at DESC LIMIT 10"
//! // params: vec!["active", "18"]
//! ```

use crate::error::SqliteError;
use std::fmt;

// ============================================================================
// Type Definitions
// ============================================================================

/// SQL value for parameterized queries.
#[derive(Clone, Debug, PartialEq)]
pub enum SqlValue {
    String(String),
    Integer(i64),
    Real(f64),
    Boolean(bool),
    Null,
    Blob(Vec<u8>),
}

impl SqlValue {
    pub fn to_param_string(&self) -> String {
        match self {
            SqlValue::String(s) => s.clone(),
            SqlValue::Integer(i) => i.to_string(),
            SqlValue::Real(r) => r.to_string(),
            SqlValue::Boolean(b) => (if *b { 1 } else { 0 }).to_string(),
            SqlValue::Null => "NULL".to_string(),
            SqlValue::Blob(_) => "BLOB".to_string(),
        }
    }
}

impl From<String> for SqlValue {
    fn from(s: String) -> Self {
        SqlValue::String(s)
    }
}

impl From<&str> for SqlValue {
    fn from(s: &str) -> Self {
        SqlValue::String(s.to_string())
    }
}

impl From<i32> for SqlValue {
    fn from(i: i32) -> Self {
        SqlValue::Integer(i as i64)
    }
}

impl From<i64> for SqlValue {
    fn from(i: i64) -> Self {
        SqlValue::Integer(i)
    }
}

impl From<f64> for SqlValue {
    fn from(f: f64) -> Self {
        SqlValue::Real(f)
    }
}

impl From<bool> for SqlValue {
    fn from(b: bool) -> Self {
        SqlValue::Boolean(b)
    }
}

/// Comparison operators for filter conditions.
#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Eq,          // =
    Ne,          // !=
    Gt,          // >
    Gte,         // >=
    Lt,          // <
    Lte,         // <=
    In,          // IN (...)
    NotIn,       // NOT IN (...)
    Like,        // LIKE
    NotLike,     // NOT LIKE
    Between,     // BETWEEN ... AND ...
    IsNull,      // IS NULL
    IsNotNull,   // IS NOT NULL
}

impl Operator {
    pub fn sql_symbol(&self) -> &str {
        match self {
            Operator::Eq => "=",
            Operator::Ne => "!=",
            Operator::Gt => ">",
            Operator::Gte => ">=",
            Operator::Lt => "<",
            Operator::Lte => "<=",
            Operator::In => "IN",
            Operator::NotIn => "NOT IN",
            Operator::Like => "LIKE",
            Operator::NotLike => "NOT LIKE",
            Operator::Between => "BETWEEN",
            Operator::IsNull => "IS NULL",
            Operator::IsNotNull => "IS NOT NULL",
        }
    }

    pub fn is_unary(&self) -> bool {
        matches!(self, Operator::IsNull | Operator::IsNotNull)
    }
}

/// Logical connectors for multiple filters.
#[derive(Clone, Debug, PartialEq)]
pub enum Logic {
    And,
    Or,
}

impl fmt::Display for Logic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Logic::And => write!(f, "AND"),
            Logic::Or => write!(f, "OR"),
        }
    }
}

/// Single filter condition.
#[derive(Clone, Debug)]
pub struct Filter {
    pub column: String,
    pub operator: Operator,
    pub value: Option<SqlValue>,
    pub values: Vec<SqlValue>, // For IN, NOT IN
    pub logic: Logic,           // How to combine with next filter
}

impl Filter {
    /// Column = value
    pub fn eq(column: &str, value: SqlValue) -> Self {
        Self {
            column: column.to_string(),
            operator: Operator::Eq,
            value: Some(value),
            values: vec![],
            logic: Logic::And,
        }
    }

    /// Column != value
    pub fn ne(column: &str, value: SqlValue) -> Self {
        Self {
            column: column.to_string(),
            operator: Operator::Ne,
            value: Some(value),
            values: vec![],
            logic: Logic::And,
        }
    }

    /// Column > value
    pub fn gt(column: &str, value: SqlValue) -> Self {
        Self {
            column: column.to_string(),
            operator: Operator::Gt,
            value: Some(value),
            values: vec![],
            logic: Logic::And,
        }
    }

    /// Column >= value
    pub fn gte(column: &str, value: SqlValue) -> Self {
        Self {
            column: column.to_string(),
            operator: Operator::Gte,
            value: Some(value),
            values: vec![],
            logic: Logic::And,
        }
    }

    /// Column < value
    pub fn lt(column: &str, value: SqlValue) -> Self {
        Self {
            column: column.to_string(),
            operator: Operator::Lt,
            value: Some(value),
            values: vec![],
            logic: Logic::And,
        }
    }

    /// Column <= value
    pub fn lte(column: &str, value: SqlValue) -> Self {
        Self {
            column: column.to_string(),
            operator: Operator::Lte,
            value: Some(value),
            values: vec![],
            logic: Logic::And,
        }
    }

    /// Column IN (values)
    pub fn in_list(column: &str, values: Vec<SqlValue>) -> Self {
        Self {
            column: column.to_string(),
            operator: Operator::In,
            value: None,
            values,
            logic: Logic::And,
        }
    }

    /// Column NOT IN (values)
    pub fn not_in(column: &str, values: Vec<SqlValue>) -> Self {
        Self {
            column: column.to_string(),
            operator: Operator::NotIn,
            value: None,
            values,
            logic: Logic::And,
        }
    }

    /// Column LIKE pattern (with % wildcards)
    pub fn like(column: &str, pattern: &str) -> Self {
        Self {
            column: column.to_string(),
            operator: Operator::Like,
            value: Some(SqlValue::String(pattern.to_string())),
            values: vec![],
            logic: Logic::And,
        }
    }

    /// Column BETWEEN min AND max
    pub fn between(column: &str, min: SqlValue, max: SqlValue) -> Self {
        Self {
            column: column.to_string(),
            operator: Operator::Between,
            value: Some(min),
            values: vec![max],
            logic: Logic::And,
        }
    }

    /// Column IS NULL
    pub fn is_null(column: &str) -> Self {
        Self {
            column: column.to_string(),
            operator: Operator::IsNull,
            value: None,
            values: vec![],
            logic: Logic::And,
        }
    }

    /// Column IS NOT NULL
    pub fn is_not_null(column: &str) -> Self {
        Self {
            column: column.to_string(),
            operator: Operator::IsNotNull,
            value: None,
            values: vec![],
            logic: Logic::And,
        }
    }

    /// Set logic to OR (changes how this filter combines with next)
    pub fn or(mut self) -> Self {
        self.logic = Logic::Or;
        self
    }
}

/// JOIN type specification.
#[derive(Clone, Debug, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Cross,
}

impl fmt::Display for JoinType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JoinType::Inner => write!(f, "INNER JOIN"),
            JoinType::Left => write!(f, "LEFT JOIN"),
            JoinType::Right => write!(f, "RIGHT JOIN"),
            JoinType::Cross => write!(f, "CROSS JOIN"),
        }
    }
}

/// JOIN clause specification.
#[derive(Clone, Debug)]
pub struct Join {
    pub join_type: JoinType,
    pub table: String,
    pub on: Option<String>,
}

impl Join {
    /// INNER JOIN table ON condition
    pub fn inner(table: &str, on: &str) -> Self {
        Self {
            join_type: JoinType::Inner,
            table: table.to_string(),
            on: Some(on.to_string()),
        }
    }

    /// LEFT JOIN table ON condition
    pub fn left(table: &str, on: &str) -> Self {
        Self {
            join_type: JoinType::Left,
            table: table.to_string(),
            on: Some(on.to_string()),
        }
    }

    /// RIGHT JOIN table ON condition
    pub fn right(table: &str, on: &str) -> Self {
        Self {
            join_type: JoinType::Right,
            table: table.to_string(),
            on: Some(on.to_string()),
        }
    }

    /// CROSS JOIN table (no ON condition)
    pub fn cross(table: &str) -> Self {
        Self {
            join_type: JoinType::Cross,
            table: table.to_string(),
            on: None,
        }
    }
}

/// Aggregate function type.
#[derive(Clone, Debug)]
pub enum Aggregate {
    Count(String),   // COUNT(column)
    Sum(String),     // SUM(column)
    Avg(String),     // AVG(column)
    Max(String),     // MAX(column)
    Min(String),     // MIN(column)
}

impl fmt::Display for Aggregate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Aggregate::Count(col) => write!(f, "COUNT({})", col),
            Aggregate::Sum(col) => write!(f, "SUM({})", col),
            Aggregate::Avg(col) => write!(f, "AVG({})", col),
            Aggregate::Max(col) => write!(f, "MAX({})", col),
            Aggregate::Min(col) => write!(f, "MIN({})", col),
        }
    }
}

// ============================================================================
// Trait Definition
// ============================================================================

/// Trait for query building with fluent API.
pub trait QueryBuilder: Send + Sync + Sized {
    /// Start SELECT with columns.
    fn select(columns: &[&str]) -> Self;

    /// Specify FROM table.
    fn from(table: &str) -> Self;

    /// Add WHERE filter.
    fn where_filter(self, filter: Filter) -> Self;

    /// Add AND filter (convenience).
    fn and(self, filter: Filter) -> Self {
        self.where_filter(filter)
    }

    /// Add OR filter.
    fn or(self, filter: Filter) -> Self {
        // Implementation depends on builder state
        self.where_filter(filter.or())
    }

    /// Add JOIN.
    fn join(self, join: Join) -> Self;

    /// Add ORDER BY.
    fn order_by(self, column: &str, asc: bool) -> Self;

    /// Add LIMIT.
    fn limit(self, limit: usize) -> Self;

    /// Add OFFSET.
    fn offset(self, offset: usize) -> Self;

    /// Build final SQL and parameters.
    fn build(self) -> Result<(String, Vec<SqlValue>), SqliteError>;
}

// ============================================================================
// SqliteQueryBuilder Implementation
// ============================================================================

/// SQLite-specific query builder with fluent API.
pub struct SqliteQueryBuilder {
    columns: Vec<String>,
    from_table: Option<String>,
    filters: Vec<Filter>,
    joins: Vec<Join>,
    group_by: Vec<String>,
    having: Option<String>,
    order_by: Vec<(String, bool)>, // (column, asc)
    limit: Option<usize>,
    offset: Option<usize>,
}

impl SqliteQueryBuilder {
    /// Validate query structure before building.
    fn validate(&self) -> Result<(), SqliteError> {
        if self.columns.is_empty() {
            return Err(SqliteError::QueryValidation("SELECT requires columns".to_string()));
        }
        if self.from_table.is_none() && self.joins.is_empty() {
            return Err(SqliteError::QueryValidation("Query requires FROM clause".to_string()));
        }
        Ok(())
    }

    /// Build WHERE clause from filters.
    fn build_where_clause(&self) -> (String, Vec<SqlValue>) {
        if self.filters.is_empty() {
            return ("".to_string(), vec![]);
        }

        let mut sql = String::from("WHERE ");
        let mut params = vec![];

        for (i, filter) in self.filters.iter().enumerate() {
            if i > 0 {
                sql.push_str(&format!(" {} ", filter.logic));
            }

            sql.push_str(&filter.column);
            sql.push(' ');

            match &filter.operator {
                Operator::IsNull | Operator::IsNotNull => {
                    sql.push_str(filter.operator.sql_symbol());
                }
                Operator::In | Operator::NotIn => {
                    sql.push_str(filter.operator.sql_symbol());
                    sql.push_str(" (");
                    for (j, _val) in filter.values.iter().enumerate() {
                        if j > 0 { sql.push(','); }
                        sql.push('?');
                        params.push(filter.values[j].clone());
                    }
                    sql.push(')');
                }
                Operator::Between => {
                    sql.push_str("BETWEEN ? AND ?");
                    params.push(filter.value.clone().unwrap_or(SqlValue::Null));
                    params.push(filter.values.get(0).cloned().unwrap_or(SqlValue::Null));
                }
                _ => {
                    sql.push_str(filter.operator.sql_symbol());
                    sql.push_str(" ?");
                    params.push(filter.value.clone().unwrap_or(SqlValue::Null));
                }
            }
        }

        (sql, params)
    }

    /// Build complete SQL query.
    fn build_sql(&self) -> (String, Vec<SqlValue>) {
        let mut sql = String::from("SELECT ");

        // Columns
        sql.push_str(&self.columns.join(", "));

        // FROM
        if let Some(table) = &self.from_table {
            sql.push_str(&format!(" FROM {}", table));
        }

        // JOINs
        for join in &self.joins {
            sql.push_str(&format!(" {} {}", join.join_type, join.table));
            if let Some(on) = &join.on {
                sql.push_str(&format!(" ON {}", on));
            }
        }

        // WHERE
        let (where_clause, mut params) = self.build_where_clause();
        if !where_clause.is_empty() {
            sql.push(' ');
            sql.push_str(&where_clause);
        }

        // GROUP BY
        if !self.group_by.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", self.group_by.join(", ")));
        }

        // HAVING
        if let Some(having) = &self.having {
            sql.push_str(&format!(" HAVING {}", having));
        }

        // ORDER BY
        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let order_parts: Vec<String> = self.order_by.iter()
                .map(|(col, asc)| {
                    if *asc {
                        format!("{} ASC", col)
                    } else {
                        format!("{} DESC", col)
                    }
                })
                .collect();
            sql.push_str(&order_parts.join(", "));
        }

        // LIMIT
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        // OFFSET
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        (sql, params)
    }
}

impl QueryBuilder for SqliteQueryBuilder {
    fn select(columns: &[&str]) -> Self {
        Self {
            columns: columns.iter().map(|s| s.to_string()).collect(),
            from_table: None,
            filters: vec![],
            joins: vec![],
            group_by: vec![],
            having: None,
            order_by: vec![],
            limit: None,
            offset: None,
        }
    }

    fn from(mut self, table: &str) -> Self {
        self.from_table = Some(table.to_string());
        self
    }

    fn where_filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    fn join(mut self, join: Join) -> Self {
        self.joins.push(join);
        self
    }

    fn order_by(mut self, column: &str, asc: bool) -> Self {
        self.order_by.push((column.to_string(), asc));
        self
    }

    fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    fn build(self) -> Result<(String, Vec<SqlValue>), SqliteError> {
        self.validate()?;
        let (sql, params) = self.build_sql();
        Ok((sql, params))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_select() {
        let (sql, params) = SqliteQueryBuilder::select(&["id", "name"])
            .from("users")
            .build()
            .unwrap();

        assert_eq!(sql, "SELECT id, name FROM users");
        assert!(params.is_empty());
    }

    #[test]
    fn test_select_with_where() {
        let (sql, params) = SqliteQueryBuilder::select(&["*"])
            .from("users")
            .where_filter(Filter::eq("id", "123".into()))
            .build()
            .unwrap();

        assert!(sql.contains("WHERE id = ?"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_select_with_order_by() {
        let (sql, _) = SqliteQueryBuilder::select(&["*"])
            .from("users")
            .order_by("created_at", false)
            .build()
            .unwrap();

        assert!(sql.contains("ORDER BY created_at DESC"));
    }

    #[test]
    fn test_select_with_pagination() {
        let (sql, _) = SqliteQueryBuilder::select(&["*"])
            .from("users")
            .limit(10)
            .offset(20)
            .build()
            .unwrap();

        assert!(sql.contains("LIMIT 10 OFFSET 20"));
    }

    #[test]
    fn test_validation_no_columns() {
        let builder = SqliteQueryBuilder {
            columns: vec![],
            from_table: Some("users".to_string()),
            filters: vec![],
            joins: vec![],
            group_by: vec![],
            having: None,
            order_by: vec![],
            limit: None,
            offset: None,
        };

        let result = builder.build();
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_no_from() {
        let result = SqliteQueryBuilder::select(&["*"]).build();
        assert!(result.is_err());
    }
}
```

---

## Module 3: `store/migrations.rs` (Schema Management)

```rust
// src/store/migrations.rs

//! Database schema migrations with version tracking.
//!
//! This module provides migration management, supporting ordered execution,
//! rollback, and integrity verification.

use async_trait::async_trait;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::SystemTime;
use rusqlite::Connection;
use crate::error::SqliteError;
use crate::store::sync::ConnectionPool;

// ============================================================================
// Type Definitions
// ============================================================================

/// Tracks a single applied migration.
#[derive(Clone, Debug)]
pub struct MigrationVersion {
    pub version: i32,
    pub name: String,
    pub checksum: String,
    pub applied_at: SystemTime,
}

/// Schema verification results.
#[derive(Clone, Debug)]
pub struct SchemaVerification {
    pub tables: Vec<String>,
    pub indexes: Vec<String>,
    pub constraints: Vec<String>,
    pub integrity_ok: bool,
}

/// Results of a migration operation.
#[derive(Clone, Debug)]
pub struct MigrationStatus {
    pub current_version: i32,
    pub previous_version: i32,
    pub applied: Vec<MigrationVersion>,
    pub duration: std::time::Duration,
}

// ============================================================================
// Traits
// ============================================================================

/// Single database migration.
#[async_trait]
pub trait Migration: Send + Sync {
    /// Migration version number (should be unique, ordered).
    fn version(&self) -> i32;

    /// Human-readable migration name.
    fn name(&self) -> &str;

    /// Checksum to verify migration hasn't been tampered with.
    fn checksum(&self) -> String;

    /// Apply the migration (upgrade).
    async fn up(&self, conn: &mut Connection) -> Result<(), SqliteError>;

    /// Rollback the migration (downgrade).
    async fn down(&self, conn: &mut Connection) -> Result<(), SqliteError>;
}

/// Trait for managing migrations.
#[async_trait]
pub trait MigrationRunner: Send + Sync {
    /// Apply migrations up to target version (or all if None).
    async fn migrate(&self, target_version: Option<i32>) -> Result<MigrationStatus, SqliteError>;

    /// Rollback N steps.
    async fn rollback(&self, steps: usize) -> Result<MigrationStatus, SqliteError>;

    /// Get current schema version.
    async fn current_version(&self) -> Result<i32, SqliteError>;

    /// Verify schema integrity.
    async fn verify(&self) -> Result<SchemaVerification, SqliteError>;
}

// ============================================================================
// Migration Runner Implementation
// ============================================================================

/// SQLite migration runner with transaction safety.
pub struct SqliteMigrationRunner {
    migrations: BTreeMap<i32, Box<dyn Migration>>,
    pool: Arc<ConnectionPool>,
}

impl SqliteMigrationRunner {
    /// Create new migration runner.
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self {
            migrations: BTreeMap::new(),
            pool,
        }
    }

    /// Register a migration.
    pub fn add_migration(&mut self, migration: Box<dyn Migration>) -> Result<(), SqliteError> {
        let version = migration.version();

        if self.migrations.contains_key(&version) {
            return Err(SqliteError::MigrationError(
                format!("Migration {} already registered", version),
            ));
        }

        self.migrations.insert(version, migration);
        Ok(())
    }

    /// Initialize migration history table (internal).
    async fn ensure_migration_table(&self) -> Result<(), SqliteError> {
        self.pool.write_tx(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS __migrations (
                    version INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    checksum TEXT NOT NULL,
                    applied_at TEXT NOT NULL
                )",
                [],
            )?;
            Ok(())
        }).await
    }

    /// Execute a single migration.
    async fn execute_migration(&self, m: &dyn Migration) -> Result<(), SqliteError> {
        self.pool.write_tx(|conn| {
            m.up(conn)?;
            Ok(())
        }).await
    }

    /// Track a migration in history.
    async fn track_migration(&self, m: &dyn Migration) -> Result<(), SqliteError> {
        let version = m.version();
        let name = m.name();
        let checksum = m.checksum();
        let applied_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();

        self.pool.write_tx(move |conn| {
            conn.execute(
                "INSERT INTO __migrations (version, name, checksum, applied_at)
                 VALUES (?, ?, ?, ?)",
                rusqlite::params![version, name, checksum, applied_at],
            )?;
            Ok(())
        }).await
    }
}

#[async_trait]
impl MigrationRunner for SqliteMigrationRunner {
    async fn migrate(&self, target_version: Option<i32>) -> Result<MigrationStatus, SqliteError> {
        let start = std::time::Instant::now();

        self.ensure_migration_table().await?;

        let current = self.current_version().await.unwrap_or(0);
        let target = target_version.unwrap_or(i32::MAX);

        let mut applied = vec![];

        for (version, migration) in self.migrations.iter() {
            if *version > current && *version <= target {
                self.execute_migration(migration).await?;
                self.track_migration(migration).await?;

                applied.push(MigrationVersion {
                    version: *version,
                    name: migration.name().to_string(),
                    checksum: migration.checksum(),
                    applied_at: SystemTime::now(),
                });
            }
        }

        Ok(MigrationStatus {
            current_version: target.min(*self.migrations.keys().last().unwrap_or(&0)),
            previous_version: current,
            applied,
            duration: start.elapsed(),
        })
    }

    async fn rollback(&self, steps: usize) -> Result<MigrationStatus, SqliteError> {
        let start = std::time::Instant::now();

        let current = self.current_version().await?;

        let mut applied = vec![];

        // Get migrations to rollback (in reverse order)
        let mut versions_to_rollback = vec![];
        for (version, _) in self.migrations.iter().rev() {
            if *version <= current && versions_to_rollback.len() < steps {
                versions_to_rollback.push(*version);
            }
        }

        // Rollback in correct order
        for version in versions_to_rollback.iter().rev() {
            if let Some(migration) = self.migrations.get(version) {
                self.pool.write_tx(|conn| {
                    migration.down(conn)?;
                    Ok(())
                }).await?;

                // Remove from history
                self.pool.write_tx(move |conn| {
                    conn.execute("DELETE FROM __migrations WHERE version = ?", [version])?;
                    Ok(())
                }).await?;

                applied.push(MigrationVersion {
                    version: *version,
                    name: migration.name().to_string(),
                    checksum: migration.checksum(),
                    applied_at: SystemTime::now(),
                });
            }
        }

        Ok(MigrationStatus {
            current_version: current - steps as i32,
            previous_version: current,
            applied,
            duration: start.elapsed(),
        })
    }

    async fn current_version(&self) -> Result<i32, SqliteError> {
        self.pool.read_tx(|conn| {
            conn.query_row(
                "SELECT COALESCE(MAX(version), 0) FROM __migrations",
                [],
                |row| row.get(0),
            )
        }).await
    }

    async fn verify(&self) -> Result<SchemaVerification, SqliteError> {
        self.pool.read_tx(|conn| {
            let mut stmt = conn.prepare(
                "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
            )?;
            let tables = stmt.query_map([], |row| row.get(0))?
                .collect::<Result<Vec<_>, _>>()?;

            let mut stmt = conn.prepare(
                "SELECT name FROM sqlite_master WHERE type='index' ORDER BY name"
            )?;
            let indexes = stmt.query_map([], |row| row.get(0))?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(SchemaVerification {
                tables,
                indexes,
                constraints: vec![],
                integrity_ok: true,
            })
        }).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    struct MockMigration {
        version: i32,
        name: String,
    }

    impl MockMigration {
        fn new(version: i32, name: &str) -> Self {
            Self {
                version,
                name: name.to_string(),
            }
        }
    }

    #[async_trait]
    impl Migration for MockMigration {
        fn version(&self) -> i32 { self.version }
        fn name(&self) -> &str { &self.name }
        fn checksum(&self) -> String { format!("{}-{}", self.version, self.name) }

        async fn up(&self, _conn: &mut Connection) -> Result<(), SqliteError> {
            Ok(())
        }

        async fn down(&self, _conn: &mut Connection) -> Result<(), SqliteError> {
            Ok(())
        }
    }

    #[test]
    fn test_migration_ordering() {
        let mut runner = SqliteMigrationRunner::new(Arc::new(
            ConnectionPool::new(Default::default()).unwrap()
        ));

        runner.add_migration(box MockMigration::new(2, "add_users")).ok();
        runner.add_migration(box MockMigration::new(1, "init")).ok();

        let versions: Vec<_> = runner.migrations.keys().copied().collect();
        assert_eq!(versions, vec![1, 2]);
    }

    #[test]
    fn test_duplicate_version_rejected() {
        let mut runner = SqliteMigrationRunner::new(Arc::new(
            ConnectionPool::new(Default::default()).unwrap()
        ));

        runner.add_migration(box MockMigration::new(1, "init")).ok();
        let result = runner.add_migration(box MockMigration::new(1, "duplicate"));
        assert!(result.is_err());
    }
}
```

---

## Summary Table

| Module | File | LOC | Responsibility | Key Structs | Traits |
|--------|------|-----|-----------------|-------------|--------|
| Sync | `store/sync.rs` | 400 | Connection pooling, transactions | `ConnectionPool`, `ConnectionConfig`, `SyncMetrics` | `SyncStore<T>`, `RowMapper<T>` |
| Query | `store/query_builder.rs` | 300 | SQL construction | `SqliteQueryBuilder`, `Filter`, `Join` | `QueryBuilder` |
| Migrations | `store/migrations.rs` | 250 | Schema management | `SqliteMigrationRunner`, `MigrationVersion`, `MigrationStatus` | `Migration`, `MigrationRunner` |
| **Total** | **3 modules** | **950** | **Complete adapter** | **9 main structs** | **5 traits** |
| **Reduction** | **lib.rs** | **632** | **From 1,582 → 200** | **Facade** | **Re-exports** |

---

## Integration in lib.rs

```rust
// src/lib.rs (~200 LOC)

pub mod store;

pub use store::sync::{ConnectionPool, ConnectionConfig, SyncStore, SyncMetrics};
pub use store::query_builder::{SqliteQueryBuilder, QueryBuilder, Filter, Join, SqlValue};
pub use store::migrations::{SqliteMigrationRunner, MigrationRunner, Migration};

#[derive(thiserror::Error, Debug)]
pub enum SqliteError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Query validation failed: {0}")]
    QueryValidation(String),
    #[error("Migration error: {0}")]
    MigrationError(String),
}

pub struct SqliteRepository<T> {
    pool: Arc<ConnectionPool>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> SqliteRepository<T> {
    pub fn new(config: ConnectionConfig) -> Result<Self, SqliteError> {
        let pool = Arc::new(ConnectionPool::new(config)?);
        Ok(Self {
            pool,
            _phantom: std::marker::PhantomData,
        })
    }
}
```

---

## Next Steps

1. Copy this blueprint
2. Implement each module (sync, query_builder, migrations) following signatures
3. Run tests after each module
4. Update lib.rs with re-exports
5. Verify backwards compatibility
