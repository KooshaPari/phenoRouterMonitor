//! T049: Inbound Sync — process Plane.so webhook events.
//!
//! Traceability: WP08-T049

mod handlers;
#[cfg(test)]
mod tests;
mod types;

pub use types::{InboundOutcome, LocalEntityStore};

use anyhow::Result;

use crate::{state_mapper::PlaneStateMapper, webhook::PlaneInboundEvent};

/// Inbound sync processor.
pub struct InboundSync {
    mapper: PlaneStateMapper,
    auto_import_enabled: bool,
}

impl InboundSync {
    pub fn new(mapper: PlaneStateMapper, auto_import_enabled: bool) -> Self {
        Self {
            mapper,
            auto_import_enabled,
        }
    }

    /// Process a webhook event and update local state via the store.
    pub fn process<S: LocalEntityStore>(
        &self,
        event: PlaneInboundEvent,
        store: &mut S,
    ) -> Result<InboundOutcome> {
        match event {
            PlaneInboundEvent::IssueCreated(issue) => {
                handlers::handle_create(&self.mapper, self.auto_import_enabled, issue, store)
            }
            PlaneInboundEvent::IssueUpdated(issue) => {
                handlers::handle_update(&self.mapper, issue, store)
            }
            PlaneInboundEvent::IssueDeleted { issue_id } => {
                handlers::handle_delete(issue_id, store)
            }
            // Module/Cycle events are handled at the webhook layer, not here.
            PlaneInboundEvent::ModuleUpdated(module) => Ok(InboundOutcome::NotTracked {
                issue_id: module.id,
            }),
            PlaneInboundEvent::ModuleDeleted { module_id } => Ok(InboundOutcome::NotTracked {
                issue_id: module_id,
            }),
            PlaneInboundEvent::CycleUpdated(cycle) => {
                Ok(InboundOutcome::NotTracked { issue_id: cycle.id })
            }
            PlaneInboundEvent::CycleDeleted { cycle_id } => {
                Ok(InboundOutcome::NotTracked { issue_id: cycle_id })
            }
        }
    }
}
