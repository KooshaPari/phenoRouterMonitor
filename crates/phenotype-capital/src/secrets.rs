//! Secret lifecycle management — validation, rotation, and .env export.

use chrono::Utc;
use reqwest::Client;
use std::collections::HashMap;
use std::path::Path;

use crate::ledger::CapitalLedger;
use crate::registry::{Account, AccountType, Registry};
use phenotype_error_core::{PhenotypeError, Result};

// ── Secret Status ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum SecretStatus {
    Valid,
    Stale,
    Invalid,
    Unknown,
}

impl std::fmt::Display for SecretStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretStatus::Valid => write!(f, "valid"),
            SecretStatus::Stale => write!(f, "stale"),
            SecretStatus::Invalid => write!(f, "invalid"),
            SecretStatus::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<&str> for SecretStatus {
    fn from(s: &str) -> Self {
        match s {
            "valid" => SecretStatus::Valid,
            "stale" => SecretStatus::Stale,
            "invalid" => SecretStatus::Invalid,
            _ => SecretStatus::Unknown,
        }
    }
}

/// A secret entry with its current validation state.
#[derive(Debug, Clone)]
pub struct SecretEntry {
    pub account_id: String,
    pub env_var: String,
    pub status: SecretStatus,
    pub last_validated: Option<i64>,
    pub rotation_interval_days: u32,
    pub next_rotation: Option<i64>,
}

// ── SecretManager ────────────────────────────────────────────────────────────

/// Manages secret validation, rotation, and propagation.
pub struct SecretManager {
    ledger: CapitalLedger,
    registry: Registry,
    client: Client,
}

impl SecretManager {
    pub fn new(ledger: CapitalLedger, registry: Registry) -> Self {
        Self {
            ledger,
            registry,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(3))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Validate a single secret by pinging its service endpoint.
    pub async fn validate_secret(&self, account: &Account, value: &str) -> SecretStatus {
        let Some(endpoint) = &account.endpoint else {
            return SecretStatus::Unknown;
        };

        let result = match account.account_type {
            AccountType::ApiKey | AccountType::ServiceKey | AccountType::ApiToken => {
                self.ping_api_key(endpoint, value, &account.account_id).await
            }
            AccountType::GithubToken => {
                self.ping_github_token(value).await
            }
            AccountType::Dsn => {
                self.ping_dsn(value).await
            }
            _ => SecretStatus::Unknown,
        };

        let status_str = result.to_string();
        let _ = self.ledger.update_account_status(&account.id, &status_str);
        let _ = self.ledger.update_secret_status(&account.id, &status_str);

        result
    }

    /// Validate all secrets that require startup validation.
    pub async fn validate_all(&self, env: &HashMap<String, String>) -> Vec<SecretEntry> {
        let accounts = self.registry.accounts_needing_validation();
        let mut results = Vec::new();

        for account in accounts {
            let Some(env_var) = &account.env_var else {
                continue;
            };
            let Some(value) = env.get(env_var) else {
                results.push(SecretEntry {
                    account_id: account.id.clone(),
                    env_var: env_var.clone(),
                    status: SecretStatus::Invalid,
                    last_validated: Some(Utc::now().timestamp()),
                    rotation_interval_days: account.rotation.interval_days,
                    next_rotation: None,
                });
                continue;
            };

            let status = self.validate_secret(account, value).await;
            results.push(SecretEntry {
                account_id: account.id.clone(),
                env_var: env_var.clone(),
                status: status.clone(),
                last_validated: Some(Utc::now().timestamp()),
                rotation_interval_days: account.rotation.interval_days,
                next_rotation: None,
            });
        }

        results
    }

    /// Check all secrets for staleness based on rotation intervals.
    pub fn check_freshness(&self) -> Result<Vec<SecretEntry>> {
        let stale = self.ledger.stale_secrets()?;
        Ok(stale
            .into_iter()
            .map(|s| SecretEntry {
                account_id: s.account_id,
                env_var: s.env_var,
                status: SecretStatus::Stale,
                last_validated: s.last_validated,
                rotation_interval_days: s.rotation_interval_days,
                next_rotation: s.next_rotation,
            })
            .collect())
    }

    /// Export secrets from the ledger to a `.env` file for a project.
    /// The file is generated (gitignored), not hand-maintained.
    pub fn export_env(&self, project_path: impl AsRef<Path>, env: &HashMap<String, String>) -> Result<()> {
        let secrets = self.ledger.list_secrets()?;
        let env_path = project_path.as_ref().join(".env");

        let mut content = String::new();
        content.push_str("# Generated by phenotype-capital — DO NOT EDIT MANUALLY\n");
        content.push_str("# Run `agileplus capital export-env` to regenerate\n\n");

        for secret in &secrets {
            if secret.status == "invalid" {
                continue;
            }
            let Some(value) = env.get(&secret.env_var) else {
                continue;
            };
            content.push_str(&format!("{}={}\n", secret.env_var, value));
        }

        std::fs::write(&env_path, content).map_err(|e| {
            PhenotypeError::Storage(format!("failed to write .env: {e}"))
        })?;

        Ok(())
    }

    /// Import existing `.env` values into the ledger (migration path).
    pub fn import_env(&self, project_path: impl AsRef<Path>) -> Result<usize> {
        let env_path = project_path.as_ref().join(".env");
        if !env_path.exists() {
            return Ok(0);
        }

        let content = std::fs::read_to_string(&env_path)?;
        let mut count = 0;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();

            if let Some(account) = self.registry.accounts.iter().find(|a| a.env_var.as_deref() == Some(key)) {
                let _ = self.ledger.upsert_secret(
                    &account.id,
                    key,
                    value,
                    account.rotation.interval_days,
                );
                count += 1;
            }
        }

        Ok(count)
    }

    /// Get all secret entries for display.
    pub fn list_all(&self) -> Result<Vec<SecretEntry>> {
        let secrets = self.ledger.list_secrets()?;
        Ok(secrets
            .into_iter()
            .map(|s| SecretEntry {
                account_id: s.account_id,
                env_var: s.env_var,
                status: SecretStatus::from(s.status.as_str()),
                last_validated: s.last_validated,
                rotation_interval_days: s.rotation_interval_days,
                next_rotation: s.next_rotation,
            })
            .collect())
    }
}

// ── Validation Helpers ───────────────────────────────────────────────────────

impl SecretManager {
    async fn ping_api_key(&self, endpoint: &str, key: &str, _account_id: &str) -> SecretStatus {
        match self.client
            .get(endpoint)
            .header("Authorization", format!("Bearer {key}"))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => SecretStatus::Valid,
            Ok(resp) if resp.status() == 401 || resp.status() == 403 => SecretStatus::Invalid,
            Ok(_) => SecretStatus::Valid,
            Err(_) => {
                // Network error — check if it's a timeout vs auth failure
                // For now, mark as stale if we can't reach the endpoint
                SecretStatus::Stale
            }
        }
    }

    async fn ping_github_token(&self, token: &str) -> SecretStatus {
        match self.client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {token}"))
            .header("User-Agent", "phenotype-capital")
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => SecretStatus::Valid,
            Ok(resp) if resp.status() == 401 => SecretStatus::Invalid,
            _ => SecretStatus::Stale,
        }
    }

    async fn ping_dsn(&self, dsn: &str) -> SecretStatus {
        // DSN validation is complex — for now, just check format
        if dsn.starts_with("https://") && dsn.contains("@") {
            SecretStatus::Valid
        } else {
            SecretStatus::Invalid
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    // Traces to: FR-SEC-001
    use super::*;
    use crate::registry::Registry;
    use std::collections::HashMap;

    fn make_manager() -> (SecretManager, HashMap<String, String>) {
        let content = r#"
[org]
name = "Test"

[accounts.llm.groq]
type = "api_key"
env_var = "GROQ_API_KEY"
endpoint = "https://api.groq.com/openai/v1/models"
rotation = { interval_days = 90, validate_on_startup = true }
tags = ["inference", "free-tier"]

[accounts.llm.claude]
type = "subscription"
env_var = "ANTHROPIC_API_KEY"
tags = ["inference", "paid"]
"#;
        let toml = toml::from_str(content).unwrap();
        let registry = Registry::from_toml(toml).unwrap();
        let ledger = CapitalLedger::in_memory().unwrap();

        // Register accounts
        for account in &registry.accounts {
            let _ = ledger.upsert_account(&account.id, &account.category, &format!("{:?}", account.account_type));
        }

        let manager = SecretManager::new(ledger, registry);
        let mut env = HashMap::new();
        env.insert("GROQ_API_KEY".to_string(), "gsk-test-key".to_string());
        env.insert("ANTHROPIC_API_KEY".to_string(), "sk-ant-test".to_string());
        (manager, env)
    }

    #[test]
    fn export_env_creates_file() {
        let (manager, env) = make_manager();

        // Seed secrets
        for account in &manager.registry.accounts {
            if let Some(env_var) = &account.env_var {
                if let Some(value) = env.get(env_var) {
                    let _ = manager.ledger.upsert_secret(&account.id, env_var, value, 90);
                }
            }
        }

        let tmp = std::env::temp_dir().join("test-export-env");
        std::fs::create_dir_all(&tmp).unwrap();

        manager.export_env(&tmp, &env).unwrap();
        let content = std::fs::read_to_string(tmp.join(".env")).unwrap();

        assert!(content.contains("GROQ_API_KEY=gsk-test-key"));
        assert!(content.contains("ANTHROPIC_API_KEY=sk-ant-test"));
        assert!(content.contains("DO NOT EDIT MANUALLY"));

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn export_env_skips_invalid_secrets() {
        let (manager, env) = make_manager();
        manager.ledger.upsert_account("llm.groq", "llm", "api_key").unwrap();
        manager.ledger.upsert_secret("llm.groq", "GROQ_API_KEY", "key", 90).unwrap();
        manager.ledger.update_secret_status("llm.groq", "invalid").unwrap();

        let tmp = std::env::temp_dir().join("test-skip-invalid");
        std::fs::create_dir_all(&tmp).unwrap();

        manager.export_env(&tmp, &env).unwrap();
        let content = std::fs::read_to_string(tmp.join(".env")).unwrap();

        assert!(!content.contains("GROQ_API_KEY"));

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn import_env_reads_existing_file() {
        let (manager, _) = make_manager();
        let tmp = std::env::temp_dir().join("test-import-env");
        std::fs::create_dir_all(&tmp).unwrap();

        let env_content = "# My env file\nGROQ_API_KEY=my-imported-key\nANTHROPIC_API_KEY=sk-ant-imported\n";
        std::fs::write(tmp.join(".env"), env_content).unwrap();

        let count = manager.import_env(&tmp).unwrap();
        assert_eq!(count, 2);

        let secrets = manager.ledger.list_secrets().unwrap();
        assert_eq!(secrets.len(), 2);

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn import_env_returns_zero_for_missing_file() {
        let (manager, _) = make_manager();
        let tmp = std::env::temp_dir().join("test-no-env");
        let count = manager.import_env(&tmp).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn list_all_returns_entries() {
        let (manager, env) = make_manager();
        for account in &manager.registry.accounts {
            if let Some(env_var) = &account.env_var {
                if let Some(value) = env.get(env_var) {
                    let _ = manager.ledger.upsert_secret(&account.id, env_var, value, 90);
                }
            }
        }

        let entries = manager.list_all().unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn secret_status_display() {
        assert_eq!(SecretStatus::Valid.to_string(), "valid");
        assert_eq!(SecretStatus::Stale.to_string(), "stale");
        assert_eq!(SecretStatus::Invalid.to_string(), "invalid");
        assert_eq!(SecretStatus::Unknown.to_string(), "unknown");
    }

    #[test]
    fn secret_status_from_str() {
        assert_eq!(SecretStatus::from("valid"), SecretStatus::Valid);
        assert_eq!(SecretStatus::from("stale"), SecretStatus::Stale);
        assert_eq!(SecretStatus::from("invalid"), SecretStatus::Invalid);
        assert_eq!(SecretStatus::from("other"), SecretStatus::Unknown);
    }

    #[test]
    fn check_freshness_returns_stale() {
        let (manager, _) = make_manager();
        manager.ledger.upsert_account("llm.groq", "llm", "api_key").unwrap();
        manager.ledger.upsert_secret("llm.groq", "GROQ_API_KEY", "key", 90).unwrap();

        // Force stale by setting past rotation
        let past = Utc::now().timestamp() - 86400;
        manager.ledger.conn.execute(
            "UPDATE secrets SET next_rotation = ?1 WHERE account_id = ?2",
            rusqlite::params![past, "llm.groq"],
        ).unwrap();

        let stale = manager.check_freshness().unwrap();
        assert_eq!(stale.len(), 1);
        assert_eq!(stale[0].status, SecretStatus::Stale);
    }
}
