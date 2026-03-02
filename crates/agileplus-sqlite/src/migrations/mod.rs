//! Migration system for agileplus-sqlite.
//!
//! Migrations are embedded as SQL files and applied in order on startup.
//! Applied migrations are tracked in the `_migrations` meta table.

use rusqlite::{Connection, Result as SqlResult};

use agileplus_domain::error::DomainError;

// Embedded SQL migrations
const MIGRATION_001: &str = include_str!("001_create_features.sql");
const MIGRATION_002: &str = include_str!("002_create_work_packages.sql");
const MIGRATION_003: &str = include_str!("003_create_governance_contracts.sql");
const MIGRATION_004: &str = include_str!("004_create_audit_log.sql");
const MIGRATION_005: &str = include_str!("005_create_evidence.sql");
const MIGRATION_006: &str = include_str!("006_create_policy_rules.sql");
const MIGRATION_007: &str = include_str!("007_create_metrics.sql");
const MIGRATION_008: &str = include_str!("008_create_wp_dependencies.sql");
const MIGRATION_009: &str = include_str!("009_create_indexes.sql");
const MIGRATION_010: &str = include_str!("010_create_events.sql");
const MIGRATION_011: &str = include_str!("011_create_snapshots.sql");
const MIGRATION_012: &str = include_str!("012_create_sync_mappings.sql");
const MIGRATION_013: &str = include_str!("013_create_api_keys.sql");
const MIGRATION_014: &str = include_str!("014_create_device_nodes.sql");

/// All migrations in order: (name, up_sql, down_sql)
const MIGRATIONS: &[(&str, &str)] = &[
    ("001_create_features", MIGRATION_001),
    ("002_create_work_packages", MIGRATION_002),
    ("003_create_governance_contracts", MIGRATION_003),
    ("004_create_audit_log", MIGRATION_004),
    ("005_create_evidence", MIGRATION_005),
    ("006_create_policy_rules", MIGRATION_006),
    ("007_create_metrics", MIGRATION_007),
    ("008_create_wp_dependencies", MIGRATION_008),
    ("009_create_indexes", MIGRATION_009),
    ("010_create_events", MIGRATION_010),
    ("011_create_snapshots", MIGRATION_011),
    ("012_create_sync_mappings", MIGRATION_012),
    ("013_create_api_keys", MIGRATION_013),
    ("014_create_device_nodes", MIGRATION_014),
];

/// Parse the UP section from a migration SQL file.
fn parse_up(sql: &str) -> &str {
    // Format is:
    //   -- UP
    //   <sql>
    //   -- DOWN
    //   <sql>
    if let Some(up_start) = sql.find("-- UP") {
        let after_up = &sql[up_start + 5..];
        if let Some(down_start) = after_up.find("-- DOWN") {
            return after_up[..down_start].trim();
        }
        return after_up.trim();
    }
    sql.trim()
}

/// Parse the DOWN section from a migration SQL file.
fn parse_down(sql: &str) -> &str {
    if let Some(down_start) = sql.find("-- DOWN") {
        return sql[down_start + 7..].trim();
    }
    ""
}

fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

/// Runs database schema migrations.
pub struct MigrationRunner<'a> {
    conn: &'a Connection,
}

impl<'a> MigrationRunner<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Create the migrations tracking table if it doesn't exist.
    fn ensure_meta_table(&self) -> Result<(), DomainError> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS _migrations (
                    id         INTEGER PRIMARY KEY AUTOINCREMENT,
                    name       TEXT    UNIQUE NOT NULL,
                    applied_at TEXT    NOT NULL
                );",
            )
            .map_err(map_err)
    }

    /// Check whether a migration has already been applied.
    fn is_applied(&self, name: &str) -> SqlResult<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM _migrations WHERE name = ?1",
            rusqlite::params![name],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Apply all pending migrations in order.
    pub fn run_all(&self) -> Result<(), DomainError> {
        self.ensure_meta_table()?;

        for (name, sql) in MIGRATIONS {
            if self.is_applied(name).map_err(map_err)? {
                continue;
            }

            let up_sql = parse_up(sql);
            self.conn.execute_batch(up_sql).map_err(|e| {
                DomainError::Storage(format!("migration {name} failed: {e}"))
            })?;

            let now = chrono::Utc::now().to_rfc3339();
            self.conn
                .execute(
                    "INSERT INTO _migrations (name, applied_at) VALUES (?1, ?2)",
                    rusqlite::params![name, now],
                )
                .map_err(map_err)?;
        }

        Ok(())
    }

    /// Roll back the most recently applied migration.
    pub fn rollback_last(&self) -> Result<(), DomainError> {
        self.ensure_meta_table()?;

        let last_name: Option<String> = self
            .conn
            .query_row(
                "SELECT name FROM _migrations ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(map_err)?;

        let Some(name) = last_name else {
            return Ok(()); // Nothing to roll back
        };

        // Find the migration SQL
        let migration = MIGRATIONS.iter().find(|(n, _)| *n == name.as_str());
        if let Some((_, sql)) = migration {
            let down_sql = parse_down(sql);
            if !down_sql.is_empty() {
                self.conn.execute_batch(down_sql).map_err(|e| {
                    DomainError::Storage(format!("rollback of {name} failed: {e}"))
                })?;
            }
        }

        self.conn
            .execute(
                "DELETE FROM _migrations WHERE name = ?1",
                rusqlite::params![name],
            )
            .map_err(map_err)?;

        Ok(())
    }
}

/// Extension trait to add `.optional()` on rusqlite query results.
trait OptionalExt<T> {
    fn optional(self) -> SqlResult<Option<T>>;
}

impl<T> OptionalExt<T> for SqlResult<T> {
    fn optional(self) -> SqlResult<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
