//! AgilePlus hidden sub-commands registry.
//!
//! Defines ~25 sub-commands invocable via Claude Code's SlashCommand tool,
//! organized into 7 categories. Each invocation is logged to an append-only
//! JSONL audit trail.
//!
//! Traceability: FR-048, FR-049 / WP20

pub mod audit;
pub mod registry;

pub use audit::AuditLog;
pub use registry::{SubCommand, SubCommandCategory, SubCommandRegistry};
