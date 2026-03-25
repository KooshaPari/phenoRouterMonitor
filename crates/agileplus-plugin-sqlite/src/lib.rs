//! AgilePlus SQLite Plugin — persistence layer adapter.
//!
//! Implements `StoragePlugin` trait for the AgilePlus plugin system.
//! Uses rusqlite with WAL mode and foreign keys for data integrity.

mod error;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rusqlite::{params, Connection};

use agileplus_plugin_core::{
    error::{PluginError, PluginResult},
    traits::{AdapterPlugin, StoragePlugin},
};

pub use error::SqliteError;

/// SQLite-backed storage adapter implementing `StoragePlugin`.
///
/// Uses a single write-serialized connection protected by a Mutex.
/// WAL mode is enabled to allow concurrent reads; all writes are serialized.
pub struct SqliteStoragePlugin {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl SqliteStoragePlugin {
    /// Create a new SQLite storage plugin from a database path.
    pub fn new(db_path: impl AsRef<Path>) -> PluginResult<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        let conn = Connection::open(&db_path)
            .map_err(|e| PluginError::Initialization(format!("failed to open db: {}", e)))?;

        // Enable WAL mode for concurrent reads
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| PluginError::Initialization(format!("WAL pragma failed: {}", e)))?;

        // Enable foreign key enforcement
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|e| PluginError::Initialization(format!("FK pragma failed: {}", e)))?;

        // Run migrations
        Self::run_migrations(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        })
    }

    /// Create an in-memory database for testing.
    #[allow(dead_code)]
    pub fn in_memory() -> PluginResult<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| PluginError::Initialization(format!("failed to open in-memory db: {}", e)))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| PluginError::Initialization(format!("pragma failed: {}", e)))?;

        Self::run_migrations(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: PathBuf::from(":memory:"),
        })
    }

    fn run_migrations(conn: &Connection) -> PluginResult<()> {
        // Create tables for AgilePlus domain
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS features (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                slug TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                state TEXT NOT NULL DEFAULT 'draft',
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS work_packages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                feature_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                state TEXT NOT NULL DEFAULT 'backlog',
                priority TEXT NOT NULL DEFAULT 'medium',
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (feature_id) REFERENCES features(id)
            );

            CREATE TABLE IF NOT EXISTS audit_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                feature_id INTEGER NOT NULL,
                entry_type TEXT NOT NULL,
                actor TEXT NOT NULL,
                details TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (feature_id) REFERENCES features(id)
            );

            CREATE TABLE IF NOT EXISTS plugin_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .map_err(|e| PluginError::Initialization(format!("migration failed: {}", e)))?;

        Ok(())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, PluginError> {
        self.conn
            .lock()
            .map_err(|e| PluginError::Operation(format!("mutex poisoned: {}", e)))
    }
}

impl std::fmt::Debug for SqliteStoragePlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteStoragePlugin")
            .field("db_path", &self.db_path)
            .finish()
    }
}

impl AdapterPlugin for SqliteStoragePlugin {
    fn name(&self) -> &str {
        "sqlite-storage"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn initialize(&self, _config: agileplus_plugin_core::traits::PluginConfig) -> PluginResult<()> {
        // Ensure database is accessible by checking the schema
        let conn = self.lock()?;
        conn.query_row("SELECT COUNT(*) FROM sqlite_master", [], |_| Ok(()))
            .map_err(|e| PluginError::Operation(format!("init check failed: {}", e)))?;
        Ok(())
    }
}

#[async_trait]
impl StoragePlugin for SqliteStoragePlugin {
    // -- Feature operations --

    async fn create_feature(&self, feature: &serde_json::Value) -> PluginResult<i64> {
        let conn = self.lock()?;
        let slug = feature
            .get("slug")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PluginError::Validation("missing slug".to_string()))?;
        let name = feature
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PluginError::Validation("missing name".to_string()))?;
        let description = feature.get("description").and_then(|v| v.as_str());
        let state = feature
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("draft");

        conn.execute(
            "INSERT INTO features (slug, name, description, state) VALUES (?1, ?2, ?3, ?4)",
            params![slug, name, description, state],
        )
        .map_err(|e| PluginError::Operation(format!("failed to create feature: {}", e)))?;

        Ok(conn.last_insert_rowid())
    }

    async fn get_feature_by_slug(&self, slug: &str) -> PluginResult<Option<serde_json::Value>> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare("SELECT id, slug, name, description, state, created_at, updated_at FROM features WHERE slug = ?1")
            .map_err(|e| PluginError::Operation(format!("query prepare failed: {}", e)))?;

        let result = stmt.query_row([slug], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "slug": row.get::<_, String>(1)?,
                "name": row.get::<_, String>(2)?,
                "description": row.get::<_, Option<String>>(3)?,
                "state": row.get::<_, String>(4)?,
                "created_at": row.get::<_, String>(5)?,
                "updated_at": row.get::<_, String>(6)?,
            }))
        });

        match result {
            Ok(feature) => Ok(Some(feature)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(PluginError::Operation(format!(
                "failed to get feature: {}",
                e
            ))),
        }
    }

    async fn get_feature_by_id(&self, id: i64) -> PluginResult<Option<serde_json::Value>> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare("SELECT id, slug, name, description, state, created_at, updated_at FROM features WHERE id = ?1")
            .map_err(|e| PluginError::Operation(format!("query prepare failed: {}", e)))?;

        let result = stmt.query_row([id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "slug": row.get::<_, String>(1)?,
                "name": row.get::<_, String>(2)?,
                "description": row.get::<_, Option<String>>(3)?,
                "state": row.get::<_, String>(4)?,
                "created_at": row.get::<_, String>(5)?,
                "updated_at": row.get::<_, String>(6)?,
            }))
        });

        match result {
            Ok(feature) => Ok(Some(feature)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(PluginError::Operation(format!(
                "failed to get feature: {}",
                e
            ))),
        }
    }

    async fn update_feature_state(&self, id: i64, state: &str) -> PluginResult<()> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE features SET state = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![state, id],
        )
        .map_err(|e| PluginError::Operation(format!("failed to update feature: {}", e)))?;
        Ok(())
    }

    async fn list_all_features(&self) -> PluginResult<Vec<serde_json::Value>> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare("SELECT id, slug, name, description, state, created_at, updated_at FROM features ORDER BY created_at DESC")
            .map_err(|e| PluginError::Operation(format!("query prepare failed: {}", e)))?;

        let features = stmt
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "slug": row.get::<_, String>(1)?,
                    "name": row.get::<_, String>(2)?,
                    "description": row.get::<_, Option<String>>(3)?,
                    "state": row.get::<_, String>(4)?,
                    "created_at": row.get::<_, String>(5)?,
                    "updated_at": row.get::<_, String>(6)?,
                }))
            })
            .map_err(|e| PluginError::Operation(format!("query failed: {}", e)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(features)
    }

    // -- Work package operations --

    async fn create_work_package(&self, wp: &serde_json::Value) -> PluginResult<i64> {
        let conn = self.lock()?;
        let feature_id = wp
            .get("feature_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| PluginError::Validation("missing feature_id".to_string()))?;
        let title = wp
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PluginError::Validation("missing title".to_string()))?;
        let description = wp.get("description").and_then(|v| v.as_str());
        let state = wp
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("backlog");
        let priority = wp
            .get("priority")
            .and_then(|v| v.as_str())
            .unwrap_or("medium");

        conn.execute(
            "INSERT INTO work_packages (feature_id, title, description, state, priority) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![feature_id, title, description, state, priority],
        )
        .map_err(|e| PluginError::Operation(format!("failed to create work package: {}", e)))?;

        Ok(conn.last_insert_rowid())
    }

    async fn get_work_package(&self, id: i64) -> PluginResult<Option<serde_json::Value>> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare("SELECT id, feature_id, title, description, state, priority, created_at, updated_at FROM work_packages WHERE id = ?1")
            .map_err(|e| PluginError::Operation(format!("query prepare failed: {}", e)))?;

        let result = stmt.query_row([id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "feature_id": row.get::<_, i64>(1)?,
                "title": row.get::<_, String>(2)?,
                "description": row.get::<_, Option<String>>(3)?,
                "state": row.get::<_, String>(4)?,
                "priority": row.get::<_, String>(5)?,
                "created_at": row.get::<_, String>(6)?,
                "updated_at": row.get::<_, String>(7)?,
            }))
        });

        match result {
            Ok(wp) => Ok(Some(wp)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(PluginError::Operation(format!(
                "failed to get work package: {}",
                e
            ))),
        }
    }

    async fn update_wp_state(&self, id: i64, state: &str) -> PluginResult<()> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE work_packages SET state = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![state, id],
        )
        .map_err(|e| PluginError::Operation(format!("failed to update work package: {}", e)))?;
        Ok(())
    }

    // -- Audit operations --

    async fn append_audit_entry(&self, entry: &serde_json::Value) -> PluginResult<i64> {
        let conn = self.lock()?;
        let feature_id = entry
            .get("feature_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| PluginError::Validation("missing feature_id".to_string()))?;
        let entry_type = entry
            .get("entry_type")
            .and_then(|v| v.as_str())
            .unwrap_or("info");
        let actor = entry
            .get("actor")
            .and_then(|v| v.as_str())
            .unwrap_or("system");
        let details = entry.get("details").map(|v| v.to_string());

        conn.execute(
            "INSERT INTO audit_entries (feature_id, entry_type, actor, details) VALUES (?1, ?2, ?3, ?4)",
            params![feature_id, entry_type, actor, details],
        )
        .map_err(|e| PluginError::Operation(format!("failed to append audit entry: {}", e)))?;

        Ok(conn.last_insert_rowid())
    }

    async fn get_audit_trail(&self, feature_id: i64) -> PluginResult<Vec<serde_json::Value>> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare("SELECT id, feature_id, entry_type, actor, details, created_at FROM audit_entries WHERE feature_id = ?1 ORDER BY created_at DESC")
            .map_err(|e| PluginError::Operation(format!("query prepare failed: {}", e)))?;

        let entries = stmt
            .query_map([feature_id], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "feature_id": row.get::<_, i64>(1)?,
                    "entry_type": row.get::<_, String>(2)?,
                    "actor": row.get::<_, String>(3)?,
                    "details": row.get::<_, Option<String>>(4)?,
                    "created_at": row.get::<_, String>(5)?,
                }))
            })
            .map_err(|e| PluginError::Operation(format!("query failed: {}", e)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
    }
}

impl SqliteStoragePlugin {
    /// Expose the underlying connection for advanced use cases.
    #[allow(dead_code)]
    pub fn connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }

    /// Get the database path.
    #[allow(dead_code)]
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_plugin() -> SqliteStoragePlugin {
        SqliteStoragePlugin::in_memory().expect("failed to create in-memory plugin")
    }

    #[test]
    fn test_new_and_init() {
        let plugin = create_test_plugin();
        plugin.initialize(agileplus_plugin_core::traits::PluginConfig {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            adapter_config: serde_json::json!({}),
        }).expect("init failed");
    }

    #[test]
    fn test_feature_operations() {
        let plugin = create_test_plugin();
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            // Create a feature
            let feature = serde_json::json!({
                "slug": "test-feature",
                "name": "Test Feature",
                "description": "A test feature"
            });

            let id = plugin.create_feature(&feature).await.unwrap();
            assert!(id > 0);

            // Get by slug
            let retrieved = plugin.get_feature_by_slug("test-feature").await.unwrap();
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap()["name"], "Test Feature");

            // Update state
            plugin.update_feature_state(id, "active").await.unwrap();

            // Get by id
            let by_id = plugin.get_feature_by_id(id).await.unwrap();
            assert!(by_id.is_some());
            assert_eq!(by_id.unwrap()["state"], "active");

            // List all
            let all = plugin.list_all_features().await.unwrap();
            assert_eq!(all.len(), 1);
        });
    }

    #[test]
    fn test_work_package_operations() {
        let plugin = create_test_plugin();
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            // Create a feature first
            let feature = serde_json::json!({
                "slug": "wp-test",
                "name": "WP Test"
            });
            let feature_id = plugin.create_feature(&feature).await.unwrap();

            // Create a work package
            let wp = serde_json::json!({
                "feature_id": feature_id,
                "title": "Test Work Package",
                "description": "Description",
                "priority": "high"
            });

            let wp_id = plugin.create_work_package(&wp).await.unwrap();
            assert!(wp_id > 0);

            // Get work package
            let retrieved = plugin.get_work_package(wp_id).await.unwrap();
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap()["title"], "Test Work Package");

            // Update state
            plugin.update_wp_state(wp_id, "in_progress").await.unwrap();
        });
    }

    #[test]
    fn test_audit_operations() {
        let plugin = create_test_plugin();
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            // Create a feature first
            let feature = serde_json::json!({
                "slug": "audit-test",
                "name": "Audit Test"
            });
            let feature_id = plugin.create_feature(&feature).await.unwrap();

            // Add audit entries
            let entry = serde_json::json!({
                "feature_id": feature_id,
                "entry_type": "created",
                "actor": "test",
                "details": "{\"action\": \"created\"}"
            });

            plugin.append_audit_entry(&entry).await.unwrap();

            // Get audit trail
            let trail = plugin.get_audit_trail(feature_id).await.unwrap();
            assert_eq!(trail.len(), 1);
            assert_eq!(trail[0]["entry_type"], "created");
        });
    }
}
