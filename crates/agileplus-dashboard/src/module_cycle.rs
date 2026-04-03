//! Native modules and cycles dashboard views.

use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use chrono::Utc;

use agileplus_domain::domain::{
    cycle::{Cycle, CycleState},
    module::Module,
};

use crate::app_state::SharedState;

#[derive(Debug, Clone)]
pub struct ModuleCardView {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub parent_name: String,
    pub feature_count: usize,
    pub active_feature_count: usize,
    pub shipped_feature_count: usize,
    pub work_package_count: usize,
    pub cycle_count: usize,
    pub status_label: String,
}

#[derive(Debug, Clone)]
pub struct CycleCardView {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub state: String,
    pub module_scope: String,
    pub feature_count: usize,
    pub work_package_count: usize,
    pub days_remaining: i64,
    pub shippable: bool,
}

#[derive(Template)]
#[template(path = "pages/modules.html")]
pub struct ModulesPage {
    pub modules: Vec<ModuleCardView>,
    pub selected_module_id: i64,
    pub total_modules: usize,
    pub total_features: usize,
    pub total_cycles: usize,
    pub total_work_packages: usize,
}

#[derive(Template)]
#[template(path = "pages/cycles.html")]
pub struct CyclesPage {
    pub cycles: Vec<CycleCardView>,
    pub selected_cycle_id: i64,
    pub total_cycles: usize,
    pub active_cycles: usize,
    pub total_features: usize,
    pub total_work_packages: usize,
}

fn render<T: Template>(tpl: T) -> Response {
    match tpl.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template error: {err}"),
        )
            .into_response(),
    }
}

fn feature_counts_for_module(
    store: &crate::app_state::DashboardStore,
    module_id: i64,
) -> (usize, usize, usize) {
    store.feature_counts_for_module(module_id)
}

fn module_parent_name(store: &crate::app_state::DashboardStore, module: &Module) -> String {
    module
        .parent_module_id
        .and_then(|parent_id| {
            store
                .modules
                .iter()
                .find(|candidate| candidate.id == parent_id)
        })
        .map(|parent| parent.friendly_name.clone())
        .unwrap_or_default()
}

fn build_module_view(store: &crate::app_state::DashboardStore, module: &Module) -> ModuleCardView {
    let (feature_count, active_feature_count, shipped_feature_count) =
        feature_counts_for_module(store, module.id);
    let work_package_count = store.work_package_count_for_module(module.id);
    let cycle_count = store
        .cycles
        .iter()
        .filter(|cycle| cycle.module_scope_id == Some(module.id))
        .count();
    let status_label = if shipped_feature_count > 0 && active_feature_count == 0 {
        "Shipped-ready".to_string()
    } else if active_feature_count > 0 {
        "In progress".to_string()
    } else {
        "Planned".to_string()
    };

    ModuleCardView {
        id: module.id,
        slug: module.slug.clone(),
        name: module.friendly_name.clone(),
        description: module
            .description
            .clone()
            .unwrap_or_else(|| "No description".to_string()),
        parent_name: module_parent_name(store, module),
        feature_count,
        active_feature_count,
        shipped_feature_count,
        work_package_count,
        cycle_count,
        status_label,
    }
}

fn build_cycle_view(store: &crate::app_state::DashboardStore, cycle: &Cycle) -> CycleCardView {
    let feature_ids = store.cycle_feature_ids(cycle.id);
    let feature_count = feature_ids.len();
    let work_package_count = store.cycle_work_package_count(cycle.id);
    let days_remaining = (cycle.end_date - Utc::now().date_naive()).num_days();
    let module_scope = cycle
        .module_scope_id
        .and_then(|module_id| store.modules.iter().find(|module| module.id == module_id))
        .map(|module| module.friendly_name.clone())
        .unwrap_or_else(|| "Unscoped".to_string());

    CycleCardView {
        id: cycle.id,
        name: cycle.name.clone(),
        description: cycle
            .description
            .clone()
            .unwrap_or_else(|| "No description".to_string()),
        state: cycle.state.to_string(),
        module_scope,
        feature_count,
        work_package_count,
        days_remaining,
        shippable: store.cycle_is_shippable(cycle.id),
    }
}

async fn render_modules_page(state: SharedState, selected_module_id: i64) -> Response {
    let store = state.read().await;
    let modules = store
        .modules
        .iter()
        .map(|module| build_module_view(&store, module))
        .collect::<Vec<_>>();
    let total_features = store.features.len();
    let total_cycles = store.cycles.len();
    let total_work_packages = store.work_packages.values().map(Vec::len).sum();
    render(ModulesPage {
        modules,
        selected_module_id,
        total_modules: store.modules.len(),
        total_features,
        total_cycles,
        total_work_packages,
    })
}

async fn render_cycles_page(state: SharedState, selected_cycle_id: i64) -> Response {
    let store = state.read().await;
    let cycles = store
        .cycles
        .iter()
        .map(|cycle| build_cycle_view(&store, cycle))
        .collect::<Vec<_>>();
    let active_cycles = store
        .cycles
        .iter()
        .filter(|cycle| {
            matches!(
                cycle.state,
                CycleState::Draft | CycleState::Active | CycleState::Review
            )
        })
        .count();
    let total_features = store.features.len();
    let total_work_packages = store.work_packages.values().map(Vec::len).sum();
    render(CyclesPage {
        cycles,
        selected_cycle_id,
        total_cycles: store.cycles.len(),
        active_cycles,
        total_features,
        total_work_packages,
    })
}

pub async fn modules_page(State(state): State<SharedState>) -> Response {
    render_modules_page(state, 0).await
}

pub async fn module_detail_page(State(state): State<SharedState>, Path(id): Path<i64>) -> Response {
    let has_module = {
        let store = state.read().await;
        store.modules.iter().any(|module| module.id == id)
    };
    if !has_module {
        return (StatusCode::NOT_FOUND, "Module not found").into_response();
    }
    render_modules_page(state, id).await
}

pub async fn cycles_page(State(state): State<SharedState>) -> Response {
    render_cycles_page(state, 0).await
}

pub async fn cycle_detail_page(State(state): State<SharedState>, Path(id): Path<i64>) -> Response {
    let has_cycle = {
        let store = state.read().await;
        store.cycles.iter().any(|cycle| cycle.id == id)
    };
    if !has_cycle {
        return (StatusCode::NOT_FOUND, "Cycle not found").into_response();
    }
    render_cycles_page(state, id).await
}
