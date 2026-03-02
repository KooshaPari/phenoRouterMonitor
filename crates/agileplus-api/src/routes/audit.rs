//! Audit trail route handlers.
//!
//! - GET /api/v1/features/:slug/audit
//! - POST /api/v1/features/:slug/audit/verify
//!
//! Traceability: WP15-T086

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};

use agileplus_domain::domain::audit::AuditChain;
use agileplus_domain::ports::{observability::ObservabilityPort, storage::StoragePort, vcs::VcsPort};

use crate::error::ApiError;
use crate::responses::AuditEntryResponse;
use crate::state::AppState;

pub fn routes<S, V, O>() -> Router<AppState<S, V, O>>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    Router::new()
        .route("/{slug}/audit", get(get_audit_trail::<S, V, O>))
        .route("/{slug}/audit/verify", post(verify_audit_chain::<S, V, O>))
}

/// `GET /api/v1/features/:slug/audit`
pub async fn get_audit_trail<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Path(slug): Path<String>,
) -> Result<Json<Vec<AuditEntryResponse>>, ApiError>
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

    let trail = state
        .storage
        .get_audit_trail(feature.id)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(
        trail.into_iter().map(AuditEntryResponse::from).collect(),
    ))
}

/// `POST /api/v1/features/:slug/audit/verify`
///
/// Verifies the integrity of the audit hash chain for a feature.
pub async fn verify_audit_chain<S, V, O>(
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

    let trail = state
        .storage
        .get_audit_trail(feature.id)
        .await
        .map_err(ApiError::from)?;

    let chain = AuditChain { entries: trail.clone() };
    match chain.verify_chain() {
        Ok(()) => Ok(Json(json!({
            "feature_slug": slug,
            "chain_valid": true,
            "entries_verified": trail.len(),
        }))),
        Err(e) => Ok(Json(json!({
            "feature_slug": slug,
            "chain_valid": false,
            "error": e.to_string(),
        }))),
    }
}
