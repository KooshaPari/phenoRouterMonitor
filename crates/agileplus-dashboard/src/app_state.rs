//! Shared application state threaded through Axum handlers.

use std::collections::HashMap;
use std::sync::Arc;

use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::WorkPackage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// A lightweight health snapshot for one service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub name: String,
    pub healthy: bool,
    pub degraded: bool,
    pub latency_ms: Option<u64>,
    pub last_check: DateTime<Utc>,
}

/// In-memory store used by dashboard handlers.
/// In production this would delegate to repositories.
#[derive(Default)]
pub struct DashboardStore {
    pub features: Vec<Feature>,
    pub work_packages: HashMap<i64, Vec<WorkPackage>>,
    pub health: Vec<ServiceHealth>,
}

pub type SharedState = Arc<RwLock<DashboardStore>>;

impl DashboardStore {
    pub fn features_by_state(&self) -> HashMap<FeatureState, Vec<&Feature>> {
        let mut map: HashMap<FeatureState, Vec<&Feature>> = HashMap::new();
        for f in &self.features {
            map.entry(f.state).or_default().push(f);
        }
        map
    }
}

pub fn default_health() -> Vec<ServiceHealth> {
    let now = Utc::now();
    vec![
        ServiceHealth { name: "NATS".into(), healthy: true, degraded: false, latency_ms: Some(2), last_check: now },
        ServiceHealth { name: "Dragonfly".into(), healthy: true, degraded: false, latency_ms: Some(1), last_check: now },
        ServiceHealth { name: "Neo4j".into(), healthy: true, degraded: false, latency_ms: Some(8), last_check: now },
        ServiceHealth { name: "MinIO".into(), healthy: true, degraded: false, latency_ms: Some(5), last_check: now },
        ServiceHealth { name: "SQLite".into(), healthy: true, degraded: false, latency_ms: Some(0), last_check: now },
        ServiceHealth { name: "API".into(), healthy: true, degraded: false, latency_ms: Some(3), last_check: now },
    ]
}
