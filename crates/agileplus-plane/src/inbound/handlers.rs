use agileplus_domain::domain::state_machine::FeatureState;
use anyhow::Result;

use crate::{
    content_hash::compute_content_hash, state_mapper::PlaneStateMapper, webhook::PlaneWebhookIssue,
};

use super::{InboundOutcome, LocalEntityStore};

pub(super) fn handle_create<S: LocalEntityStore>(
    mapper: &PlaneStateMapper,
    auto_import_enabled: bool,
    webhook_issue: PlaneWebhookIssue,
    store: &mut S,
) -> Result<InboundOutcome> {
    if store.get_content_hash(&webhook_issue.id).is_some() {
        return handle_update(mapper, webhook_issue, store);
    }

    if auto_import_enabled {
        let state = mapped_state(mapper, &webhook_issue);
        store.auto_import(&webhook_issue, state)?;
        tracing::info!(
            plane_issue_id = webhook_issue.id,
            title = webhook_issue.name,
            "auto-imported new Plane.so work item"
        );
        Ok(InboundOutcome::AutoImported {
            issue_id: webhook_issue.id,
            title: webhook_issue.name,
        })
    } else {
        tracing::debug!(
            plane_issue_id = webhook_issue.id,
            "work item not tracked; auto-import disabled"
        );
        Ok(InboundOutcome::NotTracked {
            issue_id: webhook_issue.id,
        })
    }
}

pub(super) fn handle_update<S: LocalEntityStore>(
    mapper: &PlaneStateMapper,
    webhook_issue: PlaneWebhookIssue,
    store: &mut S,
) -> Result<InboundOutcome> {
    let Some(existing_hash) = store.get_content_hash(&webhook_issue.id) else {
        return Ok(InboundOutcome::NotTracked {
            issue_id: webhook_issue.id,
        });
    };

    let new_state = mapped_state(mapper, &webhook_issue);
    let new_hash = compute_content_hash(
        &webhook_issue.name,
        webhook_issue.state.as_deref().unwrap_or(""),
        &new_state.to_string(),
        &webhook_issue.labels,
    );

    if new_hash == existing_hash {
        tracing::debug!(
            plane_issue_id = webhook_issue.id,
            "work item content hash unchanged; skipping"
        );
        return Ok(InboundOutcome::Unchanged {
            issue_id: webhook_issue.id,
        });
    }

    store.apply_update(&webhook_issue.id, new_state, new_hash.clone())?;
    tracing::info!(
        plane_issue_id = webhook_issue.id,
        new_state = ?new_state,
        "applied inbound work item update from Plane.so"
    );
    Ok(InboundOutcome::Updated {
        issue_id: webhook_issue.id,
        new_hash,
        new_state,
    })
}

pub(super) fn handle_delete<S: LocalEntityStore>(
    issue_id: String,
    store: &mut S,
) -> Result<InboundOutcome> {
    if store.get_content_hash(&issue_id).is_none() {
        return Ok(InboundOutcome::NotTracked {
            issue_id: issue_id.clone(),
        });
    }

    store.mark_archived(&issue_id)?;
    tracing::info!(
        plane_issue_id = issue_id,
        "archived deleted Plane.so work item"
    );
    Ok(InboundOutcome::Archived { issue_id })
}

fn mapped_state(mapper: &PlaneStateMapper, webhook_issue: &PlaneWebhookIssue) -> FeatureState {
    let state_group = webhook_issue.state.as_deref().unwrap_or("backlog");
    mapper.map_plane_state(state_group, state_group)
}
