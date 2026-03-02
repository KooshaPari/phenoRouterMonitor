//! API response types — stable JSON shapes for all endpoints.
//!
//! These types are separate from domain entities so internal representations
//! can change without breaking the public API contract.
//!
//! Traceability: WP15-T086

use serde::{Deserialize, Serialize};

use agileplus_domain::domain::audit::AuditEntry;
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::governance::GovernanceContract;
use agileplus_domain::domain::work_package::WorkPackage;

// ----- Features -----

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureResponse {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub state: String,
    pub target_branch: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Feature> for FeatureResponse {
    fn from(f: Feature) -> Self {
        Self {
            id: f.id,
            slug: f.slug,
            name: f.friendly_name,
            state: format!("{:?}", f.state).to_lowercase(),
            target_branch: f.target_branch,
            created_at: f.created_at.to_rfc3339(),
            updated_at: f.updated_at.to_rfc3339(),
        }
    }
}

// ----- Work Packages -----

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkPackageResponse {
    pub id: i64,
    pub feature_id: i64,
    pub title: String,
    pub state: String,
    pub sequence: i32,
    pub acceptance_criteria: String,
    pub pr_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<WorkPackage> for WorkPackageResponse {
    fn from(wp: WorkPackage) -> Self {
        Self {
            id: wp.id,
            feature_id: wp.feature_id,
            title: wp.title,
            state: format!("{:?}", wp.state).to_lowercase(),
            sequence: wp.sequence,
            acceptance_criteria: wp.acceptance_criteria,
            pr_url: wp.pr_url,
            created_at: wp.created_at.to_rfc3339(),
            updated_at: wp.updated_at.to_rfc3339(),
        }
    }
}

// ----- Governance -----

#[derive(Debug, Serialize, Deserialize)]
pub struct GovernanceResponse {
    pub id: i64,
    pub feature_id: i64,
    pub version: i32,
    pub rules_count: usize,
    pub bound_at: String,
}

impl From<GovernanceContract> for GovernanceResponse {
    fn from(c: GovernanceContract) -> Self {
        Self {
            id: c.id,
            feature_id: c.feature_id,
            version: c.version,
            rules_count: c.rules.len(),
            bound_at: c.bound_at.to_rfc3339(),
        }
    }
}

// ----- Audit -----

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditEntryResponse {
    pub id: i64,
    pub feature_id: i64,
    pub wp_id: Option<i64>,
    pub timestamp: String,
    pub actor: String,
    pub transition: String,
    pub hash: String,
}

impl From<AuditEntry> for AuditEntryResponse {
    fn from(e: AuditEntry) -> Self {
        Self {
            id: e.id,
            feature_id: e.feature_id,
            wp_id: e.wp_id,
            timestamp: e.timestamp.to_rfc3339(),
            actor: e.actor,
            transition: e.transition,
            hash: e.hash.iter().map(|b| format!("{b:02x}")).collect(),
        }
    }
}

// ----- Health -----

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

impl HealthResponse {
    pub fn ok() -> Self {
        Self {
            status: "ok",
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}
