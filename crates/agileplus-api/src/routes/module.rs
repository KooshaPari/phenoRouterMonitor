//! Module route handlers.
//!
//! - GET /api/modules         -> list root modules (JSON)
//! - GET /api/modules/:id     -> get module with features (JSON)
//! - GET /api/modules/:id/tree -> recursive tree as flat list (JSON)
//! - GET /modules             -> module tree sidebar (HTML)
//!
//! Traces to: FR-D01, FR-D04

use askama::Template;
use axum::extract::{Path, State};
use axum::response::Html;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use agileplus_domain::domain::module::{Module, ModuleWithFeatures};
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::{
    observability::ObservabilityPort, storage::StoragePort, vcs::VcsPort,
};

use crate::error::ApiError;
use crate::state::AppState;

/// A flattened tree node for JSON and template rendering.
#[derive(Debug, Clone, Serialize)]
pub struct ModuleTreeNode {
    pub module: Module,
    pub depth: usize,
    pub owned_count: usize,
    pub tagged_count: usize,
}

/// Askama template for the module tree sidebar page.
#[derive(Template)]
#[template(path = "module_tree.html")]
pub struct ModuleTreeTemplate {
    pub nodes: Vec<ModuleTreeNode>,
}

/// Build the sub-router for module routes.
pub fn routes<S, V, O>() -> Router<AppState<S, V, O>>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    Router::new()
        .route("/", get(list_modules::<S, V, O>))
        .route("/{id}", get(get_module::<S, V, O>))
        .route("/{id}/tree", get(get_module_tree::<S, V, O>))
}

/// `GET /api/modules` - list all root modules.
/// Traces to: FR-D01, FR-D04
async fn list_modules<S, V, O>(
    State(app): State<AppState<S, V, O>>,
) -> Result<Json<Vec<Module>>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let modules = app
        .storage
        .list_root_modules()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(modules))
}

/// `GET /api/modules/:id` - get module with features.
async fn get_module<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Path(id): Path<i64>,
) -> Result<Json<ModuleWithFeatures>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let mwf = app
        .storage
        .get_module_with_features(id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("module {id} not found")))?;
    Ok(Json(mwf))
}

/// `GET /api/modules/:id/tree` - recursive module tree as flat list.
async fn get_module_tree<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<ModuleTreeNode>>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let mut nodes = Vec::new();
    flatten_tree(id, 0, app.storage.as_ref(), &mut nodes)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(nodes))
}

/// `GET /modules` - render module tree sidebar HTML.
/// Traces to: FR-D01, FR-D04
pub async fn module_tree_page<S, V, O>(
    State(app): State<AppState<S, V, O>>,
) -> Result<Html<String>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let roots = app
        .storage
        .list_root_modules()
        .await
        .map_err(ApiError::from)?;
    let mut nodes = Vec::new();
    for root in &roots {
        flatten_tree(root.id, 0, app.storage.as_ref(), &mut nodes)
            .await
            .map_err(ApiError::from)?;
    }
    let tmpl = ModuleTreeTemplate { nodes };
    let rendered = tmpl
        .render()
        .map_err(|e| ApiError::Template(e.to_string()))?;
    Ok(Html(rendered))
}

/// Recursively flatten a module tree into a depth-annotated list.
async fn flatten_tree<S: StoragePort>(
    module_id: i64,
    depth: usize,
    storage: &S,
    out: &mut Vec<ModuleTreeNode>,
) -> Result<(), DomainError> {
    let mwf = storage.get_module_with_features(module_id).await?;
    if let Some(mwf) = mwf {
        let children = mwf.child_modules.clone();
        out.push(ModuleTreeNode {
            owned_count: mwf.owned_features.len(),
            tagged_count: mwf.tagged_features.len(),
            depth,
            module: mwf.module,
        });
        for child in children {
            Box::pin(flatten_tree(child.id, depth + 1, storage, out)).await?;
        }
    }
    Ok(())
}
