use std::env;

use agileplus_domain::ports::StoragePort;
use anyhow::Context;

use crate::client::PlaneClient;
use crate::outbound::{
    push_cycle, push_cycle_delete, push_feature_cycle_assignment, push_feature_cycle_unassignment,
    push_feature_module_assignment, push_feature_module_unassignment, push_module,
    push_module_delete,
};

const DEFAULT_PLANE_API_URL: &str = "https://app.plane.so";

fn plane_client_from_env() -> Option<PlaneClient> {
    let api_key = env::var("PLANE_API_KEY").ok()?;
    let workspace_slug = env::var("PLANE_WORKSPACE").ok()?;
    let project_id = env::var("PLANE_PROJECT").ok()?;
    let base_url = env::var("PLANE_API_URL").unwrap_or_else(|_| DEFAULT_PLANE_API_URL.to_string());

    Some(PlaneClient::new(
        base_url,
        api_key,
        workspace_slug,
        project_id,
    ))
}

pub async fn maybe_sync_module_from_env<S: StoragePort>(
    storage: &S,
    module_id: i64,
) -> anyhow::Result<()> {
    let Some(client) = plane_client_from_env() else {
        return Ok(());
    };
    let module = storage
        .get_module(module_id)
        .await
        .context("loading module for Plane sync")?
        .ok_or_else(|| anyhow::anyhow!("module {module_id} not found for Plane sync"))?;
    push_module(&client, storage, &module).await
}

pub async fn maybe_delete_module_from_env<S: StoragePort>(
    storage: &S,
    module_id: i64,
) -> anyhow::Result<()> {
    let Some(client) = plane_client_from_env() else {
        return Ok(());
    };
    push_module_delete(&client, storage, module_id).await
}

pub async fn maybe_sync_cycle_from_env<S: StoragePort>(
    storage: &S,
    cycle_id: i64,
) -> anyhow::Result<()> {
    let Some(client) = plane_client_from_env() else {
        return Ok(());
    };
    let cycle = storage
        .get_cycle(cycle_id)
        .await
        .context("loading cycle for Plane sync")?
        .ok_or_else(|| anyhow::anyhow!("cycle {cycle_id} not found for Plane sync"))?;
    push_cycle(&client, storage, &cycle).await
}

pub async fn maybe_delete_cycle_from_env<S: StoragePort>(
    storage: &S,
    cycle_id: i64,
) -> anyhow::Result<()> {
    let Some(client) = plane_client_from_env() else {
        return Ok(());
    };
    push_cycle_delete(&client, storage, cycle_id).await
}

pub async fn maybe_sync_feature_module_assignment_from_env<S: StoragePort>(
    storage: &S,
    feature_id: i64,
    module_id: i64,
) -> anyhow::Result<()> {
    let Some(client) = plane_client_from_env() else {
        return Ok(());
    };
    push_feature_module_assignment(&client, storage, feature_id, module_id).await
}

pub async fn maybe_sync_feature_module_unassignment_from_env<S: StoragePort>(
    storage: &S,
    feature_id: i64,
    module_id: i64,
) -> anyhow::Result<()> {
    let Some(client) = plane_client_from_env() else {
        return Ok(());
    };
    push_feature_module_unassignment(&client, storage, feature_id, module_id).await
}

pub async fn maybe_sync_feature_cycle_assignment_from_env<S: StoragePort>(
    storage: &S,
    feature_id: i64,
    cycle_id: i64,
) -> anyhow::Result<()> {
    let Some(client) = plane_client_from_env() else {
        return Ok(());
    };
    push_feature_cycle_assignment(&client, storage, feature_id, cycle_id).await
}

pub async fn maybe_sync_feature_cycle_unassignment_from_env<S: StoragePort>(
    storage: &S,
    feature_id: i64,
    cycle_id: i64,
) -> anyhow::Result<()> {
    let Some(client) = plane_client_from_env() else {
        return Ok(());
    };
    push_feature_cycle_unassignment(&client, storage, feature_id, cycle_id).await
}
