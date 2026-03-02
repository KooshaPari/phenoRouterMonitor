//! Governance route handlers.
//!
//! - GET /api/v1/features/:slug/governance
//! - POST /api/v1/features/:slug/validate  (trigger governance evaluation)
//!
//! Traceability: WP15-T086

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};

use agileplus_domain::ports::{observability::ObservabilityPort, storage::StoragePort, vcs::VcsPort};

use crate::error::ApiError;
use crate::responses::GovernanceResponse;
use crate::state::AppState;

pub fn routes<S, V, O>() -> Router<AppState<S, V, O>>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    Router::new()
        .route("/{slug}/governance", get(get_governance::<S, V, O>))
        .route("/{slug}/validate", post(trigger_validate::<S, V, O>))
}

/// `GET /api/v1/features/:slug/governance`
pub async fn get_governance<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Path(slug): Path<String>,
) -> Result<Json<GovernanceResponse>, ApiError>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    let feature = state
        .storage
        .get_feature_by_slug(&slug)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Feature '{slug}' not found")))?;

    let contract = state
        .storage
        .get_latest_governance_contract(feature.id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| {
            ApiError::NotFound(format!("No governance contract for feature '{slug}'"))
        })?;

    Ok(Json(GovernanceResponse::from(contract)))
}

/// `POST /api/v1/features/:slug/validate`
///
/// Triggers governance validation and returns a summary report.
/// Full evaluator integration is handled by the GovernanceEvaluator from WP11.
pub async fn trigger_validate<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, ApiError>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    let feature = state
        .storage
        .get_feature_by_slug(&slug)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Feature '{slug}' not found")))?;

    let contract = state
        .storage
        .get_latest_governance_contract(feature.id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| {
            ApiError::NotFound(format!("No governance contract for feature '{slug}'"))
        })?;

    // Get evidence for all WPs in this feature
    let wps = state
        .storage
        .list_wps_by_feature(feature.id)
        .await
        .map_err(ApiError::from)?;

    let mut total_rules = 0usize;
    let mut satisfied_rules = 0usize;

    for rule in &contract.rules {
        total_rules += 1;
        // Check if at least one piece of evidence satisfies each required evidence type
        let mut rule_satisfied = rule.required_evidence.is_empty();
        for req in &rule.required_evidence {
            let evidence = state
                .storage
                .get_evidence_by_fr(&req.fr_id)
                .await
                .map_err(ApiError::from)?;
            let wp_ids: std::collections::HashSet<i64> = wps.iter().map(|w| w.id).collect();
            if evidence.iter().any(|e| wp_ids.contains(&e.wp_id)
                && e.evidence_type == req.evidence_type)
            {
                rule_satisfied = true;
            }
        }
        if rule_satisfied {
            satisfied_rules += 1;
        }
    }

    let compliant = satisfied_rules == total_rules;
    Ok(Json(json!({
        "feature_slug": slug,
        "governance_version": contract.version,
        "total_rules": total_rules,
        "satisfied_rules": satisfied_rules,
        "compliant": compliant,
    })))
}
