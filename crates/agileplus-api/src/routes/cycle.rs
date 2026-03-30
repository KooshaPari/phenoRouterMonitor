//! Cycle route handlers.
//!
//! - GET /api/cycles?state=X  -> list cycles (JSON)
//! - GET /api/cycles/:id      -> get cycle with features (JSON)
//! - GET /cycles              -> cycle kanban board (HTML)
//! - GET /cycles/:id          -> cycle detail page (HTML)
//!
//! Traces to: FR-D02, FR-D03

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::response::Html;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use agileplus_domain::domain::cycle::{Cycle, CycleState, CycleWithFeatures};
use agileplus_domain::ports::{
    observability::ObservabilityPort, storage::StoragePort, vcs::VcsPort,
};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CycleListParams {
    pub state: Option<String>,
}

/// Entry for a single cycle in the kanban board.
#[derive(Debug)]
pub struct CycleColumnEntry {
    pub cycle: Cycle,
    pub feature_count: usize,
}

/// Askama template for the cycle kanban board page.
#[derive(Template)]
#[template(path = "cycle_kanban.html")]
pub struct CycleKanbanTemplate {
    pub draft: Vec<CycleColumnEntry>,
    pub active: Vec<CycleColumnEntry>,
    pub review: Vec<CycleColumnEntry>,
    pub shipped: Vec<CycleColumnEntry>,
    pub archived: Vec<CycleColumnEntry>,
}

/// Askama template for the cycle detail page.
#[derive(Template)]
#[template(path = "cycle_detail.html")]
pub struct CycleDetailTemplate {
    pub cwf: CycleWithFeatures,
    pub scope_module_name: Option<String>,
    pub days_remaining: i64,
}

/// Build the sub-router for cycle API routes.
pub fn routes<S, V, O>() -> Router<AppState<S, V, O>>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    Router::new()
        .route("/", get(list_cycles::<S, V, O>))
        .route("/{id}", get(get_cycle::<S, V, O>))
}

/// `GET /api/cycles[?state=Draft|Active|...]`
/// Traces to: FR-D02
async fn list_cycles<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Query(params): Query<CycleListParams>,
) -> Result<Json<Vec<Cycle>>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let cycles = if let Some(ref state_str) = params.state {
        let cs: CycleState =
            state_str
                .parse()
                .map_err(|e: agileplus_domain::error::DomainError| {
                    ApiError::BadRequest(e.to_string())
                })?;
        app.storage
            .list_cycles_by_state(cs)
            .await
            .map_err(ApiError::from)?
    } else {
        app.storage
            .list_all_cycles()
            .await
            .map_err(ApiError::from)?
    };
    Ok(Json(cycles))
}

/// `GET /api/cycles/:id`
/// Traces to: FR-D03
async fn get_cycle<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Path(id): Path<i64>,
) -> Result<Json<CycleWithFeatures>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let cwf = app
        .storage
        .get_cycle_with_features(id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("cycle {id} not found")))?;
    Ok(Json(cwf))
}

/// `GET /cycles` - render cycle kanban board HTML.
/// Traces to: FR-D02
pub async fn cycle_kanban_page<S, V, O>(
    State(app): State<AppState<S, V, O>>,
) -> Result<Html<String>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let all_cycles = app
        .storage
        .list_all_cycles()
        .await
        .map_err(ApiError::from)?;

    let mut draft = Vec::new();
    let mut active = Vec::new();
    let mut review = Vec::new();
    let mut shipped = Vec::new();
    let mut archived = Vec::new();

    for cycle in all_cycles {
        let cwf = app
            .storage
            .get_cycle_with_features(cycle.id)
            .await
            .map_err(ApiError::from)?;
        let feature_count = cwf.as_ref().map_or(0, |c| c.features.len());
        let entry = CycleColumnEntry {
            cycle: cycle.clone(),
            feature_count,
        };
        match cycle.state {
            CycleState::Draft => draft.push(entry),
            CycleState::Active => active.push(entry),
            CycleState::Review => review.push(entry),
            CycleState::Shipped => shipped.push(entry),
            CycleState::Archived => archived.push(entry),
        }
    }

    let tmpl = CycleKanbanTemplate {
        draft,
        active,
        review,
        shipped,
        archived,
    };
    let rendered = tmpl
        .render()
        .map_err(|e| ApiError::Template(e.to_string()))?;
    Ok(Html(rendered))
}

/// `GET /cycles/:id` - render cycle detail page with WP burndown.
/// Traces to: FR-D03
pub async fn cycle_detail_page<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Path(id): Path<i64>,
) -> Result<Html<String>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let cwf = app
        .storage
        .get_cycle_with_features(id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("cycle {id} not found")))?;

    let today = chrono::Utc::now().date_naive();
    let days_remaining = (cwf.cycle.end_date - today).num_days();

    let scope_module_name = if let Some(module_id) = cwf.cycle.module_scope_id {
        app.storage
            .get_module(module_id)
            .await
            .map_err(ApiError::from)?
            .map(|m| m.friendly_name)
    } else {
        None
    };

    let tmpl = CycleDetailTemplate {
        cwf,
        scope_module_name,
        days_remaining,
    };
    let rendered = tmpl
        .render()
        .map_err(|e| ApiError::Template(e.to_string()))?;
    Ok(Html(rendered))
}
