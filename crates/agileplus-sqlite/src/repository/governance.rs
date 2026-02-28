//! Governance repository — CRUD for `governance_contracts` and `policy_rules`.

use rusqlite::{Connection, Row, params};

use agileplus_domain::{
    domain::governance::{GovernanceContract, GovernanceRule, PolicyDefinition, PolicyDomain, PolicyRule},
    error::DomainError,
};

fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

fn policy_domain_str(d: PolicyDomain) -> &'static str {
    match d {
        PolicyDomain::Security => "security",
        PolicyDomain::Quality => "quality",
        PolicyDomain::Compliance => "compliance",
        PolicyDomain::Performance => "performance",
        PolicyDomain::Custom => "custom",
    }
}

fn policy_domain_from_str(s: &str) -> Result<PolicyDomain, DomainError> {
    match s {
        "security" => Ok(PolicyDomain::Security),
        "quality" => Ok(PolicyDomain::Quality),
        "compliance" => Ok(PolicyDomain::Compliance),
        "performance" => Ok(PolicyDomain::Performance),
        "custom" => Ok(PolicyDomain::Custom),
        _ => Err(DomainError::Storage(format!("invalid policy domain: {s}"))),
    }
}

// -- Governance Contracts --

pub fn create_governance_contract(
    conn: &Connection,
    contract: &GovernanceContract,
) -> Result<i64, DomainError> {
    let rules_json = serde_json::to_string(&contract.rules)
        .map_err(|e| DomainError::Storage(e.to_string()))?;

    conn.execute(
        "INSERT INTO governance_contracts (feature_id, version, rules, bound_at)
         VALUES (?1,?2,?3,?4)",
        params![
            contract.feature_id,
            contract.version,
            rules_json,
            contract.bound_at.to_rfc3339(),
        ],
    )
    .map_err(map_err)?;

    Ok(conn.last_insert_rowid())
}

fn row_to_contract_parts(row: &Row<'_>) -> rusqlite::Result<(i64, i64, i32, String, String)> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
    ))
}

fn parse_contract(
    parts: (i64, i64, i32, String, String),
) -> Result<GovernanceContract, DomainError> {
    let (id, feature_id, version, rules_json, bound_at_s) = parts;

    let rules: Vec<GovernanceRule> = serde_json::from_str(&rules_json)
        .map_err(|e| DomainError::Storage(e.to_string()))?;
    let bound_at = bound_at_s
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| DomainError::Storage(e.to_string()))?;

    Ok(GovernanceContract {
        id,
        feature_id,
        version,
        rules,
        bound_at,
    })
}

pub fn get_governance_contract(
    conn: &Connection,
    feature_id: i64,
    version: i32,
) -> Result<Option<GovernanceContract>, DomainError> {
    conn.query_row(
        "SELECT id,feature_id,version,rules,bound_at
         FROM governance_contracts WHERE feature_id=?1 AND version=?2",
        params![feature_id, version],
        row_to_contract_parts,
    )
    .optional()
    .map_err(map_err)?
    .map(parse_contract)
    .transpose()
}

pub fn get_latest_governance_contract(
    conn: &Connection,
    feature_id: i64,
) -> Result<Option<GovernanceContract>, DomainError> {
    conn.query_row(
        "SELECT id,feature_id,version,rules,bound_at
         FROM governance_contracts WHERE feature_id=?1 ORDER BY version DESC LIMIT 1",
        params![feature_id],
        row_to_contract_parts,
    )
    .optional()
    .map_err(map_err)?
    .map(parse_contract)
    .transpose()
}

// -- Policy Rules --

pub fn create_policy_rule(conn: &Connection, rule: &PolicyRule) -> Result<i64, DomainError> {
    let rule_json = serde_json::to_string(&rule.rule)
        .map_err(|e| DomainError::Storage(e.to_string()))?;

    conn.execute(
        "INSERT INTO policy_rules (domain, rule, active, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5)",
        params![
            policy_domain_str(rule.domain),
            rule_json,
            rule.active as i32,
            rule.created_at.to_rfc3339(),
            rule.updated_at.to_rfc3339(),
        ],
    )
    .map_err(map_err)?;

    Ok(conn.last_insert_rowid())
}

pub fn list_active_policies(conn: &Connection) -> Result<Vec<PolicyRule>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id,domain,rule,active,created_at,updated_at
             FROM policy_rules WHERE active = 1",
        )
        .map_err(map_err)?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .map_err(map_err)?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(map_err)?
        .into_iter()
        .map(|(id, domain_s, rule_json, active, created_at_s, updated_at_s)| {
            let domain = policy_domain_from_str(&domain_s)?;
            let rule: PolicyDefinition = serde_json::from_str(&rule_json)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            let created_at = created_at_s
                .parse::<chrono::DateTime<chrono::Utc>>()
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            let updated_at = updated_at_s
                .parse::<chrono::DateTime<chrono::Utc>>()
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            Ok(PolicyRule {
                id,
                domain,
                rule,
                active: active != 0,
                created_at,
                updated_at,
            })
        })
        .collect()
}

/// Extension trait to add `.optional()` on rusqlite query results.
trait OptionalExt<T> {
    fn optional(self) -> rusqlite::Result<Option<T>>;
}

impl<T> OptionalExt<T> for rusqlite::Result<T> {
    fn optional(self) -> rusqlite::Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
