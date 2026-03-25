//! Backlog / queue route handlers.
//!
//! Traceability: FR-049 / WP11-T071

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use agileplus_domain::domain::backlog::{
    BacklogFilters, BacklogItem, BacklogPriority, BacklogSort, BacklogStatus, Intent,
};
use agileplus_domain::ports::{
    observability::ObservabilityPort, storage::StoragePort, vcs::VcsPort,
};

use crate::error::ApiError;
use crate::responses::BacklogItemResponse;
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
            get(list_backlog::<S, V, O>).post(create_backlog::<S, V, O>),
        )
        .route("/import", post(import_backlog::<S, V, O>))
        .route("/{id}", get(get_backlog_item::<S, V, O>))
        .route("/{id}/transition", post(transition_backlog_item::<S, V, O>))
        .route("/pop", post(pop_backlog::<S, V, O>))
}

#[derive(Debug, Deserialize)]
pub struct BacklogListParams {
    pub r#type: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub feature_slug: Option<String>,
    pub source: Option<String>,
    pub sort: Option<String>,
    pub limit: Option<usize>,
}

/// `GET /api/v1/backlog`
pub async fn list_backlog<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Query(params): Query<BacklogListParams>,
) -> Result<Json<Vec<BacklogItemResponse>>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let filters = BacklogFilters {
        intent: parse_intent_opt(params.r#type)?,
        status: parse_status_opt(params.status)?,
        priority: parse_priority_opt(params.priority)?,
        feature_slug: params.feature_slug,
        source: params.source,
        limit: Some(params.limit.unwrap_or(20)),
        sort: params
            .sort
            .as_deref()
            .map(parse_sort)
            .transpose()?
            .unwrap_or_default(),
    };

    let items = state
        .storage
        .list_backlog_items(&filters)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(
        items.into_iter().map(BacklogItemResponse::from).collect(),
    ))
}

#[derive(Debug, Deserialize)]
pub struct CreateBacklogRequest {
    pub title: String,
    pub description: Option<String>,
    pub r#type: Option<String>,
    pub priority: Option<String>,
    pub source: Option<String>,
    pub feature_slug: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImportBacklogRequest {
    pub items: Vec<CreateBacklogRequest>,
}

#[derive(Debug, Serialize)]
pub struct ImportBacklogResponse {
    pub imported: Vec<BacklogItemResponse>,
}

/// `POST /api/v1/backlog`
pub async fn create_backlog<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Json(body): Json<CreateBacklogRequest>,
) -> Result<(StatusCode, Json<BacklogItemResponse>), ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let intent = parse_intent(body.r#type)?;
    let mut item = BacklogItem::from_triage(
        body.title,
        body.description.unwrap_or_default(),
        intent,
        body.source.unwrap_or_else(|| "api".to_string()),
    )
    .with_feature_slug(body.feature_slug)
    .with_tags(body.tags);

    if let Some(priority) = body.priority {
        item.priority = parse_priority(priority)?;
    }

    let id = state
        .storage
        .create_backlog_item(&item)
        .await
        .map_err(ApiError::from)?;
    let created = BacklogItem {
        id: Some(id),
        ..item
    };

    Ok((
        StatusCode::CREATED,
        Json(BacklogItemResponse::from(created)),
    ))
}

/// `POST /api/v1/backlog/import`
pub async fn import_backlog<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Json(body): Json<ImportBacklogRequest>,
) -> Result<(StatusCode, Json<ImportBacklogResponse>), ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let mut imported = Vec::with_capacity(body.items.len());
    for item in body.items {
        let intent = parse_intent(item.r#type)?;
        let mut backlog_item = BacklogItem::from_triage(
            item.title,
            item.description.unwrap_or_default(),
            intent,
            item.source.unwrap_or_else(|| "api".to_string()),
        )
        .with_feature_slug(item.feature_slug)
        .with_tags(item.tags);

        if let Some(priority) = item.priority {
            backlog_item.priority = parse_priority(priority)?;
        }

        let id = state
            .storage
            .create_backlog_item(&backlog_item)
            .await
            .map_err(ApiError::from)?;
        imported.push(BacklogItem {
            id: Some(id),
            ..backlog_item
        });
    }

    Ok((
        StatusCode::CREATED,
        Json(ImportBacklogResponse {
            imported: imported
                .into_iter()
                .map(BacklogItemResponse::from)
                .collect(),
        }),
    ))
}

/// `GET /api/v1/backlog/:id`
pub async fn get_backlog_item<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Path(id): Path<i64>,
) -> Result<Json<BacklogItemResponse>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let item = state
        .storage
        .get_backlog_item(id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Backlog item {id} not found")))?;
    Ok(Json(BacklogItemResponse::from(item)))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransitionBacklogRequest {
    pub target_status: String,
}

#[derive(Debug, Serialize)]
pub struct TransitionBacklogResponse {
    pub backlog_item_id: i64,
    pub from_status: String,
    pub to_status: String,
}

/// `POST /api/v1/backlog/:id/transition`
pub async fn transition_backlog_item<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Path(id): Path<i64>,
    Json(body): Json<TransitionBacklogRequest>,
) -> Result<Json<TransitionBacklogResponse>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let item = state
        .storage
        .get_backlog_item(id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Backlog item {id} not found")))?;

    let target = parse_status(&body.target_status)?;
    state
        .storage
        .update_backlog_status(id, target)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(TransitionBacklogResponse {
        backlog_item_id: id,
        from_status: item.status.to_string(),
        to_status: target.to_string(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct PopBacklogParams {
    pub count: Option<usize>,
}

/// `POST /api/v1/backlog/pop`
pub async fn pop_backlog<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Query(params): Query<PopBacklogParams>,
) -> Result<Json<Vec<BacklogItemResponse>>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let count = params.count.unwrap_or(1);
    let mut items = Vec::new();
    for _ in 0..count {
        match state
            .storage
            .pop_next_backlog_item()
            .await
            .map_err(ApiError::from)?
        {
            Some(item) => items.push(BacklogItemResponse::from(item)),
            None => break,
        }
    }

    Ok(Json(items))
}

fn parse_intent(value: Option<String>) -> Result<Intent, ApiError> {
    let value = value.unwrap_or_else(|| "task".to_string());
    value
        .parse::<Intent>()
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

fn parse_intent_opt(value: Option<String>) -> Result<Option<Intent>, ApiError> {
    value.map(|v| parse_intent(Some(v))).transpose()
}

fn parse_priority(value: String) -> Result<BacklogPriority, ApiError> {
    value
        .parse::<BacklogPriority>()
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

fn parse_priority_opt(value: Option<String>) -> Result<Option<BacklogPriority>, ApiError> {
    value.map(parse_priority).transpose()
}

fn parse_status(value: &str) -> Result<BacklogStatus, ApiError> {
    value
        .parse::<BacklogStatus>()
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

fn parse_status_opt(value: Option<String>) -> Result<Option<BacklogStatus>, ApiError> {
    value.as_deref().map(parse_status).transpose()
}

fn parse_sort(value: &str) -> Result<BacklogSort, ApiError> {
    value
        .parse::<BacklogSort>()
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}
