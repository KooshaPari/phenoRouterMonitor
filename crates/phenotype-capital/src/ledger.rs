//! Capital ledger — SQLite-backed persistence for accounts, secrets, and
//! consumption records.

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::path::Path;

use phenotype_cost_core::{BudgetManager, Cost, CostError};
use phenotype_error_core::{PhenotypeError, Result};

// ── Entities ─────────────────────────────────────────────────────────────────

/// A resource account tracked in the ledger.
#[derive(Debug, Clone)]
pub struct ResourceAccount {
    pub id: i64,
    pub account_id: String,
    pub category: String,
    pub account_type: String,
    pub env_var: Option<String>,
    pub plan: Option<String>,
    pub tags: String,
    pub is_active: bool,
    pub last_validated: Option<i64>,
    pub freshness_status: String,
}

/// A secret entry in the ledger.
#[derive(Debug, Clone)]
pub struct SecretRecord {
    pub id: i64,
    pub account_id: String,
    pub env_var: String,
    pub value_hash: String,
    pub last_validated: Option<i64>,
    pub status: String,
    pub rotation_interval_days: u32,
    pub next_rotation: Option<i64>,
}

/// A consumption record (token usage, API calls, etc.).
#[derive(Debug, Clone)]
pub struct ConsumptionRecord {
    pub id: i64,
    pub account_id: String,
    pub project_id: String,
    pub agent_id: String,
    pub tokens_used: u64,
    pub api_calls: u64,
    pub cost_cents: u64,
    pub timestamp: i64,
}

/// Daily summary for an account.
#[derive(Debug, Clone, Default)]
pub struct DailySummary {
    pub account_id: String,
    pub date: String,
    pub total_tokens: u64,
    pub total_api_calls: u64,
    pub total_cost_cents: u64,
}

// ── CapitalLedger ────────────────────────────────────────────────────────────

/// SQLite-backed capital ledger.
pub struct CapitalLedger {
    conn: Connection,
}

impl CapitalLedger {
    /// Open a file-backed database, enable WAL mode, and run migrations.
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        if let Some(parent) = db_path.as_ref().parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                PhenotypeError::Storage(format!("failed to create db directory: {e}"))
            })?;
        }
        let conn = Connection::open(&db_path)
            .map_err(|e| PhenotypeError::Storage(format!("failed to open capital db: {e}")))?;
        Self::configure_and_migrate(conn)
    }

    /// Open an in-memory database (for tests).
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| PhenotypeError::Storage(format!("failed to open in-memory db: {e}")))?;
        Self::configure_and_migrate(conn)
    }

    fn configure_and_migrate(mut conn: Connection) -> Result<Self> {
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;",
        )
        .map_err(|e| PhenotypeError::Storage(format!("pragma failed: {e}")))?;

        Self::run_migrations(&mut conn)?;

        Ok(Self { conn })
    }

    fn run_migrations(conn: &mut Connection) -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS resource_accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id TEXT UNIQUE NOT NULL,
                category TEXT NOT NULL,
                account_type TEXT NOT NULL,
                env_var TEXT,
                plan TEXT,
                tags TEXT DEFAULT '',
                is_active INTEGER DEFAULT 1,
                last_validated INTEGER,
                freshness_status TEXT DEFAULT 'unknown'
            );

            CREATE TABLE IF NOT EXISTS secrets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id TEXT NOT NULL REFERENCES resource_accounts(account_id),
                env_var TEXT NOT NULL,
                value_hash TEXT NOT NULL,
                last_validated INTEGER,
                status TEXT DEFAULT 'unknown',
                rotation_interval_days INTEGER DEFAULT 90,
                next_rotation INTEGER
            );

            CREATE TABLE IF NOT EXISTS consumption (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id TEXT NOT NULL,
                project_id TEXT NOT NULL DEFAULT '',
                agent_id TEXT NOT NULL DEFAULT '',
                tokens_used INTEGER DEFAULT 0,
                api_calls INTEGER DEFAULT 0,
                cost_cents INTEGER DEFAULT 0,
                timestamp INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS allocations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id TEXT NOT NULL,
                account_id TEXT NOT NULL,
                daily_token_budget INTEGER DEFAULT 0,
                daily_api_budget INTEGER DEFAULT 0,
                daily_cost_budget_cents INTEGER DEFAULT 0,
                UNIQUE(project_id, account_id)
            );

            CREATE INDEX IF NOT EXISTS idx_consumption_account
                ON consumption(account_id, timestamp);
            CREATE INDEX IF NOT EXISTS idx_consumption_project
                ON consumption(project_id, timestamp);
            CREATE INDEX IF NOT EXISTS idx_secrets_account
                ON secrets(account_id);
            CREATE INDEX IF NOT EXISTS idx_secrets_status
                ON secrets(status);",
        )
        .map_err(|e| PhenotypeError::Storage(format!("migration failed: {e}")))?;

        Ok(())
    }

    // ── Account CRUD ─────────────────────────────────────────────────────────

    /// Register or update a resource account.
    pub fn upsert_account(
        &self,
        account_id: &str,
        category: &str,
        account_type: &str,
    ) -> Result<i64> {
        self.conn
            .execute(
                "INSERT INTO resource_accounts (account_id, category, account_type)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(account_id) DO UPDATE SET
                 category = excluded.category,
                 account_type = excluded.account_type
             RETURNING id",
                params![account_id, category, account_type],
            )
            .map_err(|e| PhenotypeError::Storage(format!("upsert account failed: {e}")))
    }

    /// Get all registered accounts.
    pub fn list_accounts(&self) -> Result<Vec<ResourceAccount>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, category, account_type, env_var, plan,
                    tags, is_active, last_validated, freshness_status
             FROM resource_accounts ORDER BY category, account_id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ResourceAccount {
                id: row.get(0)?,
                account_id: row.get(1)?,
                category: row.get(2)?,
                account_type: row.get(3)?,
                env_var: row.get(4)?,
                plan: row.get(5)?,
                tags: row.get(6)?,
                is_active: row.get(7)?,
                last_validated: row.get(8)?,
                freshness_status: row.get(9)?,
            })
        })?;
        rows.collect::<std::result::Result<_, _>>()
            .map_err(|e| PhenotypeError::Storage(format!("query accounts failed: {e}")))
    }

    /// Update account freshness status.
    pub fn update_account_status(&self, account_id: &str, status: &str) -> Result<()> {
        let now = Utc::now().timestamp();
        self.conn.execute(
            "UPDATE resource_accounts SET freshness_status = ?1, last_validated = ?2
             WHERE account_id = ?3",
            params![status, now, account_id],
        )?;
        Ok(())
    }

    // ── Secret CRUD ──────────────────────────────────────────────────────────

    /// Store or update a secret (value is hashed, not stored in plaintext).
    pub fn upsert_secret(
        &self,
        account_id: &str,
        env_var: &str,
        value: &str,
        rotation_days: u32,
    ) -> Result<i64> {
        let hash = hash_secret(value);
        let now = Utc::now().timestamp();
        let next_rotation = now + (rotation_days as i64 * 86400);

        self.conn.execute(
            "INSERT INTO secrets (account_id, env_var, value_hash, last_validated,
                                  status, rotation_interval_days, next_rotation)
             VALUES (?1, ?2, ?3, ?4, 'valid', ?5, ?6)
             ON CONFLICT DO NOTHING",
            params![account_id, env_var, hash, now, rotation_days, next_rotation],
        )?;

        self.conn
            .query_row(
                "SELECT id FROM secrets WHERE account_id = ?1 AND env_var = ?2",
                params![account_id, env_var],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| PhenotypeError::Storage(format!("query secret failed: {e}")))
    }

    /// Get all secrets.
    pub fn list_secrets(&self) -> Result<Vec<SecretRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, env_var, value_hash, last_validated,
                    status, rotation_interval_days, next_rotation
             FROM secrets ORDER BY account_id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SecretRecord {
                id: row.get(0)?,
                account_id: row.get(1)?,
                env_var: row.get(2)?,
                value_hash: row.get(3)?,
                last_validated: row.get(4)?,
                status: row.get(5)?,
                rotation_interval_days: row.get(6)?,
                next_rotation: row.get(7)?,
            })
        })?;
        rows.collect::<std::result::Result<_, _>>()
            .map_err(|e| PhenotypeError::Storage(format!("query secrets failed: {e}")))
    }

    /// Update secret status after validation.
    pub fn update_secret_status(&self, account_id: &str, status: &str) -> Result<()> {
        let now = Utc::now().timestamp();
        self.conn.execute(
            "UPDATE secrets SET status = ?1, last_validated = ?2
             WHERE account_id = ?3",
            params![status, now, account_id],
        )?;
        Ok(())
    }

    /// Get secrets that are stale (past rotation interval).
    pub fn stale_secrets(&self) -> Result<Vec<SecretRecord>> {
        let now = Utc::now().timestamp();
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, env_var, value_hash, last_validated,
                    status, rotation_interval_days, next_rotation
             FROM secrets
             WHERE next_rotation IS NOT NULL AND next_rotation < ?1
                OR status = 'stale' OR status = 'invalid'",
        )?;
        let rows = stmt.query_map(params![now], |row| {
            Ok(SecretRecord {
                id: row.get(0)?,
                account_id: row.get(1)?,
                env_var: row.get(2)?,
                value_hash: row.get(3)?,
                last_validated: row.get(4)?,
                status: row.get(5)?,
                rotation_interval_days: row.get(6)?,
                next_rotation: row.get(7)?,
            })
        })?;
        rows.collect::<std::result::Result<_, _>>()
            .map_err(|e| PhenotypeError::Storage(format!("query stale secrets failed: {e}")))
    }

    // ── Consumption Tracking ─────────────────────────────────────────────────

    /// Record consumption for an account + project + agent.
    pub fn record_consumption(
        &self,
        account_id: &str,
        project_id: &str,
        agent_id: &str,
        tokens: u64,
        api_calls: u64,
        cost_cents: u64,
    ) -> Result<i64> {
        let now = Utc::now().timestamp();
        self.conn
            .execute(
                "INSERT INTO consumption (account_id, project_id, agent_id,
                                       tokens_used, api_calls, cost_cents, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![account_id, project_id, agent_id, tokens, api_calls, cost_cents, now],
            )
            .map_err(|e| PhenotypeError::Storage(format!("record consumption failed: {e}")))
    }

    /// Get daily summary for an account.
    pub fn daily_summary(&self, account_id: &str) -> Result<DailySummary> {
        let now = Utc::now();
        let day_start = now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let day_end = day_start + 86400;

        self.conn
            .query_row(
                "SELECT COALESCE(SUM(tokens_used), 0),
                    COALESCE(SUM(api_calls), 0),
                    COALESCE(SUM(cost_cents), 0)
             FROM consumption
             WHERE account_id = ?1 AND timestamp >= ?2 AND timestamp < ?3",
                params![account_id, day_start, day_end],
                |row| {
                    Ok(DailySummary {
                        account_id: account_id.to_string(),
                        date: now.date_naive().to_string(),
                        total_tokens: row.get(0)?,
                        total_api_calls: row.get(1)?,
                        total_cost_cents: row.get(2)?,
                    })
                },
            )
            .map_err(|e| PhenotypeError::Storage(format!("daily summary query failed: {e}")))
    }

    /// Get all consumption records for an account (last N days).
    pub fn consumption_history(
        &self,
        account_id: &str,
        days: u32,
    ) -> Result<Vec<ConsumptionRecord>> {
        let cutoff = Utc::now().timestamp() - (days as i64 * 86400);
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, project_id, agent_id,
                    tokens_used, api_calls, cost_cents, timestamp
             FROM consumption
             WHERE account_id = ?1 AND timestamp >= ?2
             ORDER BY timestamp DESC",
        )?;
        let rows = stmt.query_map(params![account_id, cutoff], |row| {
            Ok(ConsumptionRecord {
                id: row.get(0)?,
                account_id: row.get(1)?,
                project_id: row.get(2)?,
                agent_id: row.get(3)?,
                tokens_used: row.get(4)?,
                api_calls: row.get(5)?,
                cost_cents: row.get(6)?,
                timestamp: row.get(7)?,
            })
        })?;
        rows.collect::<std::result::Result<_, _>>()
            .map_err(|e| PhenotypeError::Storage(format!("consumption history query failed: {e}")))
    }

    // ── Allocation Management ────────────────────────────────────────────────

    /// Set resource allocation for a project.
    pub fn set_allocation(
        &self,
        project_id: &str,
        account_id: &str,
        daily_token_budget: u64,
        daily_api_budget: u64,
        daily_cost_budget_cents: u64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO allocations (project_id, account_id, daily_token_budget,
                                      daily_api_budget, daily_cost_budget_cents)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(project_id, account_id) DO UPDATE SET
                 daily_token_budget = excluded.daily_token_budget,
                 daily_api_budget = excluded.daily_api_budget,
                 daily_cost_budget_cents = excluded.daily_cost_budget_cents",
            params![
                project_id,
                account_id,
                daily_token_budget,
                daily_api_budget,
                daily_cost_budget_cents
            ],
        )?;
        Ok(())
    }

    /// Get allocation for a project + account.
    pub fn get_allocation(
        &self,
        project_id: &str,
        account_id: &str,
    ) -> Result<Option<(u64, u64, u64)>> {
        self.conn
            .query_row(
                "SELECT daily_token_budget, daily_api_budget, daily_cost_budget_cents
             FROM allocations WHERE project_id = ?1 AND account_id = ?2",
                params![project_id, account_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(|e| PhenotypeError::Storage(format!("get allocation failed: {e}")))
    }

    /// Check if a proposed spend would exceed the project's budget.
    pub fn check_budget(
        &self,
        project_id: &str,
        account_id: &str,
        proposed_tokens: u64,
    ) -> std::result::Result<(), CostError> {
        let Some((token_budget, _, _)) = self.get_allocation(project_id, account_id)? else {
            return Ok(());
        };
        if token_budget == 0u64 {
            return Ok(());
        }

        let summary = self.daily_summary(account_id)?;
        let remaining = token_budget.saturating_sub(summary.total_tokens);

        if proposed_tokens > remaining {
            return Err(CostError::BudgetExceeded {
                needed: Cost::from_tokens(proposed_tokens),
                available: Cost::from_tokens(remaining),
            });
        }
        Ok(())
    }

    /// Build a BudgetManager from the current allocation + consumption.
    pub fn budget_manager(
        &self,
        project_id: &str,
        account_id: &str,
    ) -> Result<Option<BudgetManager>> {
        let Some((token_budget, _, _)) = self.get_allocation(project_id, account_id)? else {
            return Ok(None);
        };
        if token_budget == 0 {
            return Ok(None);
        }

        let summary = self.daily_summary(account_id)?;
        let total = Cost::from_tokens(token_budget);
        let mut manager = BudgetManager::new(total);

        if summary.total_tokens > 0 {
            let spent = Cost::from_tokens(summary.total_tokens);
            let _ = manager.spend(spent);
        }

        Ok(Some(manager))
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn hash_secret(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    // Traces to: FR-CAP-002
    use super::*;

    fn make_ledger() -> CapitalLedger {
        CapitalLedger::in_memory().expect("in-memory ledger")
    }

    #[test]
    fn upsert_and_list_accounts() {
        let ledger = make_ledger();
        let id = ledger.upsert_account("llm.groq", "llm", "api_key").unwrap();
        assert!(id > 0);

        let accounts = ledger.list_accounts().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account_id, "llm.groq");
        assert_eq!(accounts[0].category, "llm");
    }

    #[test]
    fn update_account_status() {
        let ledger = make_ledger();
        ledger.upsert_account("llm.groq", "llm", "api_key").unwrap();
        ledger.update_account_status("llm.groq", "valid").unwrap();

        let accounts = ledger.list_accounts().unwrap();
        assert_eq!(accounts[0].freshness_status, "valid");
        assert!(accounts[0].last_validated.is_some());
    }

    #[test]
    fn upsert_and_list_secrets() {
        let ledger = make_ledger();
        ledger.upsert_account("llm.groq", "llm", "api_key").unwrap();
        let id = ledger
            .upsert_secret("llm.groq", "GROQ_API_KEY", "secret-value", 90)
            .unwrap();
        assert!(id > 0);

        let secrets = ledger.list_secrets().unwrap();
        assert_eq!(secrets.len(), 1);
        assert_eq!(secrets[0].account_id, "llm.groq");
        assert_eq!(secrets[0].env_var, "GROQ_API_KEY");
        assert_eq!(secrets[0].status, "valid");
    }

    #[test]
    fn secret_hash_is_not_plaintext() {
        let ledger = make_ledger();
        ledger.upsert_account("llm.groq", "llm", "api_key").unwrap();
        ledger
            .upsert_secret("llm.groq", "GROQ_API_KEY", "my-secret-key", 90)
            .unwrap();

        let secrets = ledger.list_secrets().unwrap();
        assert_ne!(secrets[0].value_hash, "my-secret-key");
        assert_eq!(secrets[0].value_hash.len(), 64);
    }

    #[test]
    fn stale_secrets_detection() {
        let ledger = make_ledger();
        ledger.upsert_account("llm.groq", "llm", "api_key").unwrap();
        ledger
            .upsert_secret("llm.groq", "GROQ_API_KEY", "key1", 90)
            .unwrap();

        // Manually set next_rotation to past
        let past = Utc::now().timestamp() - 86400;
        ledger
            .conn
            .execute(
                "UPDATE secrets SET next_rotation = ?1 WHERE account_id = ?2",
                params![past, "llm.groq"],
            )
            .unwrap();

        let stale = ledger.stale_secrets().unwrap();
        assert_eq!(stale.len(), 1);
    }

    #[test]
    fn record_and_summarize_consumption() {
        let ledger = make_ledger();
        ledger.upsert_account("llm.groq", "llm", "api_key").unwrap();

        ledger
            .record_consumption("llm.groq", "my-project", "forge", 1000, 5, 0)
            .unwrap();
        ledger
            .record_consumption("llm.groq", "my-project", "forge", 2000, 3, 0)
            .unwrap();

        let summary = ledger.daily_summary("llm.groq").unwrap();
        assert_eq!(summary.total_tokens, 3000);
        assert_eq!(summary.total_api_calls, 8);
    }

    #[test]
    fn consumption_history() {
        let ledger = make_ledger();
        ledger.upsert_account("llm.groq", "llm", "api_key").unwrap();
        ledger
            .record_consumption("llm.groq", "proj-a", "forge", 500, 2, 0)
            .unwrap();
        ledger
            .record_consumption("llm.groq", "proj-b", "sage", 300, 1, 0)
            .unwrap();

        let history = ledger.consumption_history("llm.groq", 7).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].agent_id, "sage");
        assert_eq!(history[1].agent_id, "forge");
    }

    #[test]
    fn set_and_get_allocation() {
        let ledger = make_ledger();
        ledger
            .set_allocation("my-project", "llm.groq", 100_000, 1000, 0)
            .unwrap();

        let alloc = ledger.get_allocation("my-project", "llm.groq").unwrap();
        assert!(alloc.is_some());
        let (tokens, api, cost) = alloc.unwrap();
        assert_eq!(tokens, 100_000);
        assert_eq!(api, 1000);
        assert_eq!(cost, 0);
    }

    #[test]
    fn budget_check_passes() {
        let ledger = make_ledger();
        ledger.upsert_account("llm.groq", "llm", "api_key").unwrap();
        ledger
            .set_allocation("my-project", "llm.groq", 100_000, 0, 0)
            .unwrap();

        let result = ledger.check_budget("my-project", "llm.groq", 50_000);
        assert!(result.is_ok());
    }

    #[test]
    fn budget_check_fails_when_exceeded() {
        let ledger = make_ledger();
        ledger.upsert_account("llm.groq", "llm", "api_key").unwrap();
        ledger
            .set_allocation("my-project", "llm.groq", 100_000, 0, 0)
            .unwrap();

        let result = ledger.check_budget("my-project", "llm.groq", 150_000);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CostError::BudgetExceeded { .. }
        ));
    }

    #[test]
    fn budget_manager_from_allocation() {
        let ledger = make_ledger();
        ledger.upsert_account("llm.groq", "llm", "api_key").unwrap();
        ledger
            .set_allocation("my-project", "llm.groq", 100_000, 0, 0)
            .unwrap();
        ledger
            .record_consumption("llm.groq", "my-project", "forge", 30_000, 0, 0)
            .unwrap();

        let manager = ledger.budget_manager("my-project", "llm.groq").unwrap();
        assert!(manager.is_some());
        let manager = manager.unwrap();
        assert_eq!(manager.remaining().value(), 70_000);
        assert_eq!(manager.spent().value(), 30_000);
    }

    #[test]
    fn no_allocation_returns_none() {
        let ledger = make_ledger();
        let manager = ledger.budget_manager("unknown", "llm.groq").unwrap();
        assert!(manager.is_none());
    }

    #[test]
    fn zero_budget_returns_none() {
        let ledger = make_ledger();
        ledger
            .set_allocation("my-project", "llm.groq", 0, 0, 0)
            .unwrap();
        let manager = ledger.budget_manager("my-project", "llm.groq").unwrap();
        assert!(manager.is_none());
    }
}
