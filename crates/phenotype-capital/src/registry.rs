//! Capital registry — parses `capital.toml` to build an inventory of all
//! organizational resources available to agents.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use phenotype_error_core::{PhenotypeError, Result};

// ── TOML Schema ─────────────────────────────────────────────────────────────

/// Top-level capital.toml structure.
#[derive(Debug, Clone, Deserialize)]
pub struct CapitalToml {
    pub org: OrgConfig,
    #[serde(default)]
    pub accounts: HashMap<String, HashMap<String, AccountDef>>,
    #[serde(default)]
    pub profiles: HashMap<String, HashMap<String, ProfileDef>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrgConfig {
    pub name: String,
    #[serde(default = "default_rotation_days")]
    pub default_rotation_days: u32,
    #[serde(default = "default_evidence_ledger")]
    pub evidence_ledger: String,
}

fn default_rotation_days() -> u32 {
    90
}

fn default_evidence_ledger() -> String {
    "evidence_ledger.jsonl".into()
}

/// Account definition from TOML (per-account).
#[derive(Debug, Clone, Deserialize)]
pub struct AccountDef {
    #[serde(rename = "type")]
    pub account_type: String,
    #[serde(default)]
    pub env_var: Option<String>,
    #[serde(default)]
    pub url_var: Option<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub plan: Option<String>,
    #[serde(default)]
    pub expires: Option<String>,
    #[serde(default)]
    pub credits_usd: Option<u64>,
    #[serde(default)]
    pub rate_limit: Option<RateLimitDef>,
    #[serde(default)]
    pub rotation: Option<RotationDef>,
    #[serde(default)]
    pub resources: Option<HashMap<String, u64>>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitDef {
    #[serde(default)]
    pub requests_per_day: Option<u64>,
    #[serde(default)]
    pub requests_per_minute: Option<u64>,
    #[serde(default)]
    pub tokens_per_minute: Option<u64>,
    #[serde(default)]
    pub events_per_month: Option<u64>,
    #[serde(default)]
    pub data_mb: Option<u64>,
    #[serde(default)]
    pub data_gb: Option<u64>,
    #[serde(default)]
    pub storage_gb: Option<u64>,
    #[serde(default)]
    pub mau: Option<u64>,
    #[serde(default)]
    pub db_mb: Option<u64>,
    #[serde(default)]
    pub reads: Option<u64>,
    #[serde(default)]
    pub writes: Option<u64>,
    #[serde(default)]
    pub emails_per_month: Option<u64>,
    #[serde(default)]
    pub workers_requests: Option<u64>,
    #[serde(default)]
    pub r2_storage_gb: Option<u64>,
    #[serde(default)]
    pub ec2_hours: Option<u64>,
    #[serde(default)]
    pub lambda_requests: Option<u64>,
    #[serde(default)]
    pub s3_gb: Option<u64>,
    #[serde(default)]
    pub dynamodb_gb: Option<u64>,
    #[serde(default)]
    pub arm_vms: Option<u64>,
    #[serde(default)]
    pub ocpus: Option<u64>,
    #[serde(default)]
    pub ram_gb: Option<u64>,
    #[serde(default)]
    pub bandwidth_tb: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RotationDef {
    #[serde(default)]
    pub interval_days: Option<u32>,
    #[serde(default)]
    pub validate_on_startup: bool,
}

/// Browser profile definition.
#[derive(Debug, Clone, Deserialize)]
pub struct ProfileDef {
    #[serde(rename = "type")]
    pub profile_type: String,
    #[serde(default)]
    pub managed_by: Option<String>,
    #[serde(default)]
    pub user_data_dir: Option<String>,
    #[serde(default)]
    pub auth_sessions: Vec<String>,
    #[serde(default)]
    pub refresh_strategy: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

// ── Resolved Models ──────────────────────────────────────────────────────────

/// A resolved account ready for use by the ledger.
#[derive(Debug, Clone)]
pub struct Account {
    pub id: String,
    pub category: String,
    pub account_type: AccountType,
    pub env_var: Option<String>,
    pub url_var: Option<String>,
    pub endpoint: Option<String>,
    pub plan: Option<String>,
    pub expires: Option<String>,
    pub credits_usd: Option<u64>,
    pub rate_limit: RateLimit,
    pub rotation: Rotation,
    pub resources: HashMap<String, u64>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AccountType {
    ApiKey,
    Subscription,
    AlwaysFree,
    Credits,
    FreeTier,
    ServiceKey,
    Dsn,
    GithubToken,
    ApiToken,
    Unknown(String),
}

impl From<&str> for AccountType {
    fn from(s: &str) -> Self {
        match s {
            "api_key" => AccountType::ApiKey,
            "subscription" => AccountType::Subscription,
            "always_free" => AccountType::AlwaysFree,
            "credits" => AccountType::Credits,
            "free_tier" => AccountType::FreeTier,
            "service_key" => AccountType::ServiceKey,
            "dsn" => AccountType::Dsn,
            "github_token" => AccountType::GithubToken,
            "api_token" => AccountType::ApiToken,
            other => AccountType::Unknown(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RateLimit {
    pub requests_per_day: Option<u64>,
    pub requests_per_minute: Option<u64>,
    pub tokens_per_minute: Option<u64>,
    pub events_per_month: Option<u64>,
    pub data_mb: Option<u64>,
    pub data_gb: Option<u64>,
    pub storage_gb: Option<u64>,
    pub mau: Option<u64>,
    pub db_mb: Option<u64>,
    pub reads: Option<u64>,
    pub writes: Option<u64>,
    pub emails_per_month: Option<u64>,
    pub workers_requests: Option<u64>,
    pub r2_storage_gb: Option<u64>,
    pub ec2_hours: Option<u64>,
    pub lambda_requests: Option<u64>,
    pub s3_gb: Option<u64>,
    pub dynamodb_gb: Option<u64>,
    pub arm_vms: Option<u64>,
    pub ocpus: Option<u64>,
    pub ram_gb: Option<u64>,
    pub bandwidth_tb: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Rotation {
    pub interval_days: u32,
    pub validate_on_startup: bool,
}

impl Default for Rotation {
    fn default() -> Self {
        Self {
            interval_days: 90,
            validate_on_startup: true,
        }
    }
}

// ── Registry ─────────────────────────────────────────────────────────────────

/// Resolved registry of all organizational resources.
pub struct Registry {
    pub org: OrgConfig,
    pub accounts: Vec<Account>,
    pub profiles: Vec<ProfileDef>,
}

impl Registry {
    /// Parse `capital.toml` from the given path.
    pub fn from_capital_toml(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| PhenotypeError::Config(format!("failed to read capital.toml: {e}")))?;
        let toml: CapitalToml = toml::from_str(&content)
            .map_err(|e| PhenotypeError::Config(format!("failed to parse capital.toml: {e}")))?;
        Self::from_toml(toml)
    }

    /// Build a registry from a parsed TOML structure.
    pub fn from_toml(toml: CapitalToml) -> Result<Self> {
        let mut accounts = Vec::new();
        let default_rotation = toml.org.default_rotation_days;

        for (category, defs) in &toml.accounts {
            for (name, def) in defs {
                let id = format!("{category}.{name}");
                let account = Account {
                    id,
                    category: category.clone(),
                    account_type: AccountType::from(def.account_type.as_str()),
                    env_var: def.env_var.clone(),
                    url_var: def.url_var.clone(),
                    endpoint: def.endpoint.clone(),
                    plan: def.plan.clone(),
                    expires: def.expires.clone(),
                    credits_usd: def.credits_usd,
                    rate_limit: RateLimit::from_def(def.rate_limit.as_ref()),
                    rotation: Rotation::from_def(def.rotation.as_ref(), default_rotation),
                    resources: def.resources.clone().unwrap_or_default(),
                    tags: def.tags.clone(),
                };
                accounts.push(account);
            }
        }

        let mut profiles = Vec::new();
        for (_section, defs) in &toml.profiles {
            for (_, def) in defs {
                profiles.push(def.clone());
            }
        }

        Ok(Self {
            org: toml.org,
            accounts,
            profiles,
        })
    }

    /// Get all accounts matching a given tag.
    pub fn accounts_by_tag(&self, tag: &str) -> Vec<&Account> {
        self.accounts
            .iter()
            .filter(|a| a.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Get all accounts in a category (e.g., "llm", "cloud", "database").
    pub fn accounts_by_category(&self, category: &str) -> Vec<&Account> {
        self.accounts
            .iter()
            .filter(|a| a.category == category)
            .collect()
    }

    /// Find an account by its full ID (e.g., "llm.groq").
    pub fn account_by_id(&self, id: &str) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == id)
    }

    /// Get all accounts that require startup validation.
    pub fn accounts_needing_validation(&self) -> Vec<&Account> {
        self.accounts
            .iter()
            .filter(|a| a.rotation.validate_on_startup)
            .collect()
    }

    /// Get all profiles.
    pub fn profiles(&self) -> &[ProfileDef] {
        &self.profiles
    }
}

// ── Conversions ──────────────────────────────────────────────────────────────

impl RateLimit {
    fn from_def(def: Option<&RateLimitDef>) -> Self {
        let Some(def) = def else {
            return Self::default();
        };
        Self {
            requests_per_day: def.requests_per_day,
            requests_per_minute: def.requests_per_minute,
            tokens_per_minute: def.tokens_per_minute,
            events_per_month: def.events_per_month,
            data_mb: def.data_mb,
            data_gb: def.data_gb,
            storage_gb: def.storage_gb,
            mau: def.mau,
            db_mb: def.db_mb,
            reads: def.reads,
            writes: def.writes,
            emails_per_month: def.emails_per_month,
            workers_requests: def.workers_requests,
            r2_storage_gb: def.r2_storage_gb,
            ec2_hours: def.ec2_hours,
            lambda_requests: def.lambda_requests,
            s3_gb: def.s3_gb,
            dynamodb_gb: def.dynamodb_gb,
            arm_vms: def.arm_vms,
            ocpus: def.ocpus,
            ram_gb: def.ram_gb,
            bandwidth_tb: def.bandwidth_tb,
        }
    }
}

impl Rotation {
    fn from_def(def: Option<&RotationDef>, default_days: u32) -> Self {
        let Some(def) = def else {
            return Self {
                interval_days: default_days,
                validate_on_startup: true,
            };
        };
        Self {
            interval_days: def.interval_days.unwrap_or(default_days),
            validate_on_startup: def.validate_on_startup,
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    // Traces to: FR-CAP-001
    use super::*;

    fn sample_toml() -> CapitalToml {
        let content = r#"
[org]
name = "Test"
default_rotation_days = 60

[accounts.llm.groq]
type = "api_key"
env_var = "GROQ_API_KEY"
endpoint = "https://api.groq.com/openai/v1/models"
rate_limit = { requests_per_day = 1000, tokens_per_minute = 6000 }
rotation = { interval_days = 90, validate_on_startup = true }
tags = ["inference", "free-tier"]
"#;
        toml::from_str(content).unwrap()
    }

    #[test]
    fn parses_toml_to_registry() {
        let toml = sample_toml();
        let registry = Registry::from_toml(toml).unwrap();
        assert_eq!(registry.org.name, "Test");
        assert_eq!(registry.accounts.len(), 1);
        assert_eq!(registry.accounts[0].id, "llm.groq");
        assert_eq!(registry.accounts[0].category, "llm");
    }

    #[test]
    fn resolves_account_type() {
        let toml = sample_toml();
        let registry = Registry::from_toml(toml).unwrap();
        assert_eq!(registry.accounts[0].account_type, AccountType::ApiKey);
    }

    #[test]
    fn resolves_rate_limit() {
        let toml = sample_toml();
        let registry = Registry::from_toml(toml).unwrap();
        let rl = &registry.accounts[0].rate_limit;
        assert_eq!(rl.requests_per_day, Some(1000));
        assert_eq!(rl.tokens_per_minute, Some(6000));
    }

    #[test]
    fn resolves_rotation() {
        let toml = sample_toml();
        let registry = Registry::from_toml(toml).unwrap();
        let rot = &registry.accounts[0].rotation;
        assert_eq!(rot.interval_days, 90);
        assert!(rot.validate_on_startup);
    }

    #[test]
    fn accounts_by_tag() {
        let toml = sample_toml();
        let registry = Registry::from_toml(toml).unwrap();
        let free = registry.accounts_by_tag("free-tier");
        assert_eq!(free.len(), 1);
        let paid = registry.accounts_by_tag("paid");
        assert_eq!(paid.len(), 0);
    }

    #[test]
    fn accounts_by_category() {
        let toml = sample_toml();
        let registry = Registry::from_toml(toml).unwrap();
        let llm = registry.accounts_by_category("llm");
        assert_eq!(llm.len(), 1);
        let cloud = registry.accounts_by_category("cloud");
        assert_eq!(cloud.len(), 0);
    }

    #[test]
    fn account_by_id() {
        let toml = sample_toml();
        let registry = Registry::from_toml(toml).unwrap();
        assert!(registry.account_by_id("llm.groq").is_some());
        assert!(registry.account_by_id("llm.missing").is_none());
    }

    #[test]
    fn accounts_needing_validation() {
        let toml = sample_toml();
        let registry = Registry::from_toml(toml).unwrap();
        let needs = registry.accounts_needing_validation();
        assert_eq!(needs.len(), 1);
    }

    #[test]
    fn default_rotation_uses_org_value() {
        let content = r#"
[org]
name = "Test"
default_rotation_days = 45

[accounts.llm.test]
type = "api_key"
env_var = "TEST_KEY"
"#;
        let toml: CapitalToml = toml::from_str(content).unwrap();
        let registry = Registry::from_toml(toml).unwrap();
        assert_eq!(registry.accounts[0].rotation.interval_days, 45);
    }

    #[test]
    fn unknown_account_type() {
        let content = r#"
[org]
name = "Test"

[accounts.misc.unknown]
type = "custom_type"
"#;
        let toml: CapitalToml = toml::from_str(content).unwrap();
        let registry = Registry::from_toml(toml).unwrap();
        assert_eq!(
            registry.accounts[0].account_type,
            AccountType::Unknown("custom_type".into())
        );
    }
}
