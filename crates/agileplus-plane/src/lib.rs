//! AgilePlus Plane.so sync adapter.
//!
//! Bidirectional sync between AgilePlus entities and Plane.so issues.
//! Supports webhook ingestion, outbound push, state mapping, label sync,
//! content-hash conflict detection, and a bounded retry queue.
//!
//! Traceability: FR-051 / WP08

pub mod client;
pub mod content_hash;
pub mod inbound;
pub mod labels;
pub mod outbound;
pub mod runtime;
pub mod state_mapper;
pub mod sync;
pub mod sync_queue;
pub mod webhook;

pub use client::PlaneClient;
pub use client::{
    PlaneCreateCycleRequest, PlaneCreateModuleRequest, PlaneCycleResponse, PlaneModuleResponse,
};
pub use content_hash::{ConflictStatus, compute_content_hash, detect_conflict};
pub use inbound::{InboundOutcome, InboundSync, LocalEntityStore};
pub use labels::{LabelSync, PlaneLabel};
pub use outbound::{
    OutboundSync, push_cycle, push_cycle_delete, push_feature_cycle_assignment,
    push_feature_module_assignment, push_module, push_module_delete,
};
pub use runtime::*;
pub use state_mapper::{PlaneStateMapper, PlaneStateMapperConfig};
pub use sync::{PlaneSyncAdapter, SyncState};
pub use sync_queue::{SyncOpKind, SyncQueue, SyncQueueItem, SyncQueueStore};
pub use webhook::{
    PlaneInboundEvent, PlaneWebhookAction, PlaneWebhookCycle, PlaneWebhookModule,
    PlaneWebhookPayload, handle_plane_webhook, parse_webhook, verify_hmac_signature,
};
