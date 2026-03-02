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
pub mod state_mapper;
pub mod sync;
pub mod sync_queue;
pub mod webhook;

pub use client::PlaneClient;
pub use content_hash::{compute_content_hash, detect_conflict, ConflictStatus};
pub use inbound::{InboundOutcome, InboundSync, LocalEntityStore};
pub use labels::{LabelSync, PlaneLabel};
pub use outbound::OutboundSync;
pub use state_mapper::{PlaneStateMapper, PlaneStateMapperConfig};
pub use sync::{PlaneSyncAdapter, SyncState};
pub use sync_queue::{SyncQueue, SyncQueueItem, SyncQueueStore, SyncOpKind};
pub use webhook::{
    handle_plane_webhook, parse_webhook, verify_hmac_signature, PlaneInboundEvent,
    PlaneWebhookAction, PlaneWebhookPayload,
};
