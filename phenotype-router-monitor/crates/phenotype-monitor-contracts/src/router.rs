//! Router domain contracts

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Core routing trait
pub trait Router: Send + Sync {
    fn route(&self, task: Task, context: RoutingContext) -> crate::error::Result<Target>;
    fn audit(&self) -> AuditLog;
    fn statistics(&self) -> RoutingStats;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub priority: u32,
    pub deadline_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub service: String,
    pub queue: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingContext {
    pub cost: f32,
    pub latency_ms: u64,
    pub reliability_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub entries: Vec<AuditEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub task_id: Uuid,
    pub target: Target,
    pub decision_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingStats {
    pub total_routed: u64,
    pub successful: u64,
    pub failed: u64,
}
