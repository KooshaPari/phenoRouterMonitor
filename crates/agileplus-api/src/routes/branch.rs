//! Branch route handlers.
//!
//! - GET    /api/v1/branches         -> list
//! - POST   /api/v1/branches         -> create
//! - POST   /api/v1/branches/checkout -> checkout
//! - POST   /api/v1/branches/delete   -> delete
//! - POST   /api/v1/branches/sync     -> sync source into target

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use agileplus_domain::ports::{
    BranchInfo, observability::ObservabilityPort, storage::StoragePort, vcs::VcsPort,
};

use crate::error::ApiError;
use crate::state::AppState;

pub fn routes<S, V, O>() -> Router<AppState<S, V, O>>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    Router::new()
        .route(
            "/",
            get(list_branches::<S, V, O>).post(create_branch::<S, V, O>),
        )
        .route("/checkout", post(checkout_branch::<S, V, O>))
        .route("/delete", post(delete_branch::<S, V, O>))
        .route("/sync", post(sync_branches::<S, V, O>))
}

#[derive(Debug, Deserialize)]
pub struct BranchListParams {
    pub pattern: Option<String>,
    pub remote: Option<bool>,
}

pub async fn list_branches<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Query(params): Query<BranchListParams>,
) -> Result<Json<Vec<BranchInfo>>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let branches = state
        .vcs
        .list_branches(params.pattern.as_deref(), params.remote.unwrap_or(false))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(branches))
}

#[derive(Debug, Deserialize)]
pub struct CreateBranchRequest {
    pub name: String,
    pub base: Option<String>,
}

pub async fn create_branch<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Json(body): Json<CreateBranchRequest>,
) -> Result<(StatusCode, Json<ActionResponse>), ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let base = body.base.unwrap_or_else(|| "main".to_string());
    state
        .vcs
        .create_branch(&body.name, &base)
        .await
        .map_err(ApiError::from)?;
    Ok((
        StatusCode::CREATED,
        Json(ActionResponse {
            message: format!("Created branch {} from {}", body.name, base),
        }),
    ))
}

#[derive(Debug, Deserialize)]
pub struct CheckoutBranchRequest {
    pub name: String,
}

pub async fn checkout_branch<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Json(body): Json<CheckoutBranchRequest>,
) -> Result<Json<ActionResponse>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    state
        .vcs
        .checkout_branch(&body.name)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ActionResponse {
        message: format!("Checked out branch {}", body.name),
    }))
}

#[derive(Debug, Deserialize)]
pub struct DeleteBranchRequest {
    pub name: String,
    pub force: Option<bool>,
    pub remote: Option<String>,
}

pub async fn delete_branch<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Json(body): Json<DeleteBranchRequest>,
) -> Result<Json<ActionResponse>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    state
        .vcs
        .delete_branch(
            &body.name,
            body.force.unwrap_or(false),
            body.remote.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ActionResponse {
        message: if let Some(remote) = body.remote {
            format!("Deleted remote branch {remote}/{}", body.name)
        } else {
            format!("Deleted branch {}", body.name)
        },
    }))
}

#[derive(Debug, Deserialize)]
pub struct SyncBranchRequest {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Serialize)]
pub struct SyncBranchResponse {
    pub source: String,
    pub target: String,
    pub success: bool,
    pub merged_commit: Option<String>,
}

pub async fn sync_branches<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Json(body): Json<SyncBranchRequest>,
) -> Result<Json<SyncBranchResponse>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let result = state
        .vcs
        .merge_to_target(&body.source, &body.target)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(SyncBranchResponse {
        source: body.source,
        target: body.target,
        success: result.success,
        merged_commit: result.merged_commit,
    }))
}

#[derive(Debug, Serialize)]
pub struct ActionResponse {
    pub message: String,
}
