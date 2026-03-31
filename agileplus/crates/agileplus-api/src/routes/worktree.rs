//! Worktree route handlers.
//!
//! - GET    /api/v1/worktrees -> list
//! - POST   /api/v1/worktrees -> add
//! - DELETE /api/v1/worktrees -> remove

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{delete, get};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use agileplus_domain::ports::{
    WorktreeInfo, observability::ObservabilityPort, storage::StoragePort, vcs::VcsPort,
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
            get(list_worktrees::<S, V, O>).post(add_worktree::<S, V, O>),
        )
        .route("/", delete(remove_worktree::<S, V, O>))
}

pub async fn list_worktrees<S, V, O>(
    State(state): State<AppState<S, V, O>>,
) -> Result<Json<Vec<WorktreeInfo>>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let worktrees = state.vcs.list_worktrees().await.map_err(ApiError::from)?;
    Ok(Json(worktrees))
}

#[derive(Debug, Deserialize)]
pub struct AddWorktreeRequest {
    pub feature_slug: String,
    pub wp_id: String,
}

pub async fn add_worktree<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Json(body): Json<AddWorktreeRequest>,
) -> Result<(StatusCode, Json<WorktreeInfo>), ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let path = state
        .vcs
        .create_worktree(&body.feature_slug, &body.wp_id)
        .await
        .map_err(ApiError::from)?;
    let branch = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string();
    let worktree = WorktreeInfo {
        path,
        branch,
        feature_slug: body.feature_slug,
        wp_id: body.wp_id,
    };
    Ok((StatusCode::CREATED, Json(worktree)))
}

#[derive(Debug, Deserialize)]
pub struct RemoveWorktreeRequest {
    pub path: String,
}

pub async fn remove_worktree<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Json(body): Json<RemoveWorktreeRequest>,
) -> Result<Json<ActionResponse>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    state
        .vcs
        .cleanup_worktree(std::path::Path::new(&body.path))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ActionResponse {
        message: format!("Removed worktree {}", body.path),
    }))
}

#[derive(Debug, Serialize)]
pub struct ActionResponse {
    pub message: String,
}
