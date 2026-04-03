use agileplus_domain::domain::cycle::Cycle;
use agileplus_domain::domain::module::Module;
use agileplus_domain::domain::sync_mapping::SyncMapping;
use agileplus_domain::ports::StoragePort;
use anyhow::{Context, Result};
use chrono::Utc;
use crate::client::{PlaneClient, PlaneCreateCycleRequest, PlaneCreateModuleRequest};
// -- Module & Cycle outbound push (WP06-T031) --
/// Push a newly created or updated Module to Plane.so.
/// Stores a sync_mappings row with entity_type = "module".
pub async fn push_module<S: StoragePort>(
    client: &PlaneClient,
    storage: &S,
    module: &Module,
) -> Result<()> {
    let existing = storage
        .get_sync_mapping("module", module.id)
        .await
        .context("looking up module sync mapping")?;
    let req = PlaneCreateModuleRequest {
        name: module.friendly_name.clone(),
        description: module.description.clone(),
    };
    if let Some(mapping) = existing {
        client
            .update_module(&mapping.plane_issue_id, &req)
            .await
            .with_context(|| format!("updating Plane module {}", mapping.plane_issue_id))?;
        // Update last_synced_at
        let updated = SyncMapping {
            last_synced_at: Utc::now(),
            ..mapping
        };
        storage
            .upsert_sync_mapping(&updated)
            .await
            .context("updating sync mapping timestamp")?;
    } else {
        let resp = client
            .create_module(&req)
            .await
            .context("creating Plane module")?;
        let mapping = SyncMapping::new("module", module.id, &resp.id, "");
        storage
            .upsert_sync_mapping(&mapping)
            .await
            .context("storing module sync mapping")?;
        tracing::info!(
            module_id = module.id,
            plane_module_id = resp.id,
            "synced module to Plane.so"
        );
    }
    Ok(())
}
/// Push a newly created or updated Cycle to Plane.so.
/// Stores a sync_mappings row with entity_type = "cycle".
pub async fn push_cycle<S: StoragePort>(
    client: &PlaneClient,
    storage: &S,
    cycle: &Cycle,
) -> Result<()> {
    let existing = storage
        .get_sync_mapping("cycle", cycle.id)
        .await
        .context("looking up cycle sync mapping")?;
    let req = PlaneCreateCycleRequest {
        name: cycle.name.clone(),
        description: cycle.description.clone(),
        start_date: cycle.start_date.to_string(),
        end_date: cycle.end_date.to_string(),
    };
    if let Some(mapping) = existing {
        client
            .update_cycle(&mapping.plane_issue_id, &req)
            .await
            .with_context(|| format!("updating Plane cycle {}", mapping.plane_issue_id))?;
        let updated = SyncMapping {
            last_synced_at: Utc::now(),
            ..mapping
        };
        storage
            .upsert_sync_mapping(&updated)
            .await
            .context("updating sync mapping timestamp")?;
    } else {
        let resp = client
            .create_cycle(&req)
            .await
            .context("creating Plane cycle")?;
        let mapping = SyncMapping::new("cycle", cycle.id, &resp.id, "");
        storage
            .upsert_sync_mapping(&mapping)
            .await
            .context("storing cycle sync mapping")?;
        tracing::info!(
            cycle_id = cycle.id,
            plane_cycle_id = resp.id,
            "synced cycle to Plane.so"
        );
    }
    Ok(())
}
/// Delete a Module from Plane.so and remove its sync mapping.
pub async fn push_module_delete<S: StoragePort>(
    client: &PlaneClient,
    storage: &S,
    module_id: i64,
) -> Result<()> {
    let mapping = storage
        .get_sync_mapping("module", module_id)
        .await
        .context("looking up module sync mapping for delete")?;
    if let Some(m) = mapping {
        client
            .delete_module(&m.plane_issue_id)
            .await
            .with_context(|| format!("deleting Plane module {}", m.plane_issue_id))?;
        storage
            .delete_sync_mapping("module", module_id)
            .await
            .context("removing module sync mapping")?;
        tracing::info!(
            module_id,
            plane_module_id = m.plane_issue_id,
            "deleted Plane module"
        );
    }
    Ok(())
}
/// Delete a Cycle from Plane.so and remove its sync mapping.
pub async fn push_cycle_delete<S: StoragePort>(
    client: &PlaneClient,
    storage: &S,
    cycle_id: i64,
) -> Result<()> {
    let mapping = storage
        .get_sync_mapping("cycle", cycle_id)
        .await
        .context("looking up cycle sync mapping for delete")?;
    if let Some(m) = mapping {
        client
            .delete_cycle(&m.plane_issue_id)
            .await
            .with_context(|| format!("deleting Plane cycle {}", m.plane_issue_id))?;
        storage
            .delete_sync_mapping("cycle", cycle_id)
            .await
            .context("removing cycle sync mapping")?;
        tracing::info!(
            cycle_id,
            plane_cycle_id = m.plane_issue_id,
            "deleted Plane cycle"
        );
    }
    Ok(())
}
// -- Assignment sync (WP06-T033) --
/// When a Feature is assigned to a Module, sync the Plane work-item-to-module link.
pub async fn push_feature_module_assignment<S: StoragePort>(
    client: &PlaneClient,
    storage: &S,
    feature_id: i64,
    module_id: i64,
) -> Result<()> {
    let feature_mapping = storage
        .get_sync_mapping("feature", feature_id)
        .await
        .context("looking up feature sync mapping")?;
    let module_mapping = storage
        .get_sync_mapping("module", module_id)
        .await
        .context("looking up module sync mapping")?;
    match (feature_mapping, module_mapping) {
        (Some(fm), Some(mm)) => {
            client
                .add_work_item_to_module(&mm.plane_issue_id, &fm.plane_issue_id)
                .await
                .with_context(|| {
                    format!(
                        "adding Plane work item {} to module {}",
                        fm.plane_issue_id, mm.plane_issue_id
                    )
                })?;
            tracing::info!(
                feature_id,
                module_id,
                "synced feature-to-module assignment to Plane"
            );
        }
        _ => {
            tracing::debug!(
                feature_id,
                module_id,
                "skipping feature-module assignment sync: one or both sides not mapped"
            );
        }
    }
    Ok(())
}
/// When a Feature is unassigned from a Module, sync the Plane work-item-to-module unlink.
pub async fn push_feature_module_unassignment<S: StoragePort>(
    client: &PlaneClient,
    storage: &S,
    feature_id: i64,
    module_id: i64,
) -> Result<()> {
    let feature_mapping = storage
        .get_sync_mapping("feature", feature_id)
        .await
        .context("looking up feature sync mapping")?;
    let module_mapping = storage
        .get_sync_mapping("module", module_id)
        .await
        .context("looking up module sync mapping")?;
    match (feature_mapping, module_mapping) {
        (Some(fm), Some(mm)) => {
            client
                .delete_work_item_from_module(&mm.plane_issue_id, &fm.plane_issue_id)
                .await
                .with_context(|| {
                    format!(
                        "removing Plane work item {} from module {}",
                        fm.plane_issue_id, mm.plane_issue_id
                    )
                })?;
            tracing::info!(
                feature_id,
                module_id,
                "synced feature-to-module unassignment to Plane"
            );
        }
        _ => {
            tracing::debug!(
                feature_id,
                module_id,
                "skipping feature-module unassignment sync: one or both sides not mapped"
            );
        }
    }
    Ok(())
}
/// When a Feature is assigned to a Cycle, sync the Plane work-item-to-cycle link.
pub async fn push_feature_cycle_assignment<S: StoragePort>(
    client: &PlaneClient,
    storage: &S,
    feature_id: i64,
    cycle_id: i64,
) -> Result<()> {
    let feature_mapping = storage
        .get_sync_mapping("feature", feature_id)
        .await
        .context("looking up feature sync mapping")?;
    let cycle_mapping = storage
        .get_sync_mapping("cycle", cycle_id)
        .await
        .context("looking up cycle sync mapping")?;
    match (feature_mapping, cycle_mapping) {
        (Some(fm), Some(cm)) => {
            client
                .add_work_item_to_cycle(&cm.plane_issue_id, &fm.plane_issue_id)
                .await
                .with_context(|| {
                    format!(
                        "adding Plane work item {} to cycle {}",
                        fm.plane_issue_id, cm.plane_issue_id
                    )
                })?;
            tracing::info!(
                feature_id,
                cycle_id,
                "synced feature-to-cycle assignment to Plane"
            );
        }
        _ => {
            tracing::debug!(
                feature_id,
                cycle_id,
                "skipping feature-cycle assignment sync: one or both sides not mapped"
            );
        }
    }
    Ok(())
}
/// When a Feature is unassigned from a Cycle, sync the Plane work-item-to-cycle unlink.
pub async fn push_feature_cycle_unassignment<S: StoragePort>(
    client: &PlaneClient,
    storage: &S,
    feature_id: i64,
    cycle_id: i64,
) -> Result<()> {
    let feature_mapping = storage
        .get_sync_mapping("feature", feature_id)
        .await
        .context("looking up feature sync mapping")?;
    let cycle_mapping = storage
        .get_sync_mapping("cycle", cycle_id)
        .await
        .context("looking up cycle sync mapping")?;
    match (feature_mapping, cycle_mapping) {
        (Some(fm), Some(cm)) => {
            client
                .delete_work_item_from_cycle(&cm.plane_issue_id, &fm.plane_issue_id)
                .await
                .with_context(|| {
                    format!(
                        "removing Plane work item {} from cycle {}",
                        fm.plane_issue_id, cm.plane_issue_id
                    )
                })?;
            tracing::info!(
                feature_id,
                cycle_id,
                "synced feature-to-cycle unassignment to Plane"
            );
        }
        _ => {
            tracing::debug!(
                feature_id,
                cycle_id,
                "skipping feature-cycle unassignment sync: one or both sides not mapped"
            );
        }
    }
    Ok(())
}
