//! Security Alert Aggregation

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSeverity { Critical, High, Medium, Low, Info }

impl AlertSeverity {
    pub fn score(&self) -> f32 {
        match self {
            AlertSeverity::Critical => 100.0,
            AlertSeverity::High => 75.0,
            AlertSeverity::Medium => 50.0,
            AlertSeverity::Low => 25.0,
            AlertSeverity::Info => 10.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SecurityAlert {
    pub id: String,
    pub severity: AlertSeverity,
    pub title: String,
    pub repository: String,
    pub created_at: DateTime<Utc>,
}

impl SecurityAlert {
    pub fn new(id: String, severity: AlertSeverity, title: String, repository: String) -> Self {
        Self { id, severity, title, repository, created_at: Utc::now() }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SecurityAggregator { total_alerts: usize, critical_count: usize }

impl SecurityAggregator {
    pub fn new() -> Self { Self::default() }
    pub fn add_critical(&mut self) { self.total_alerts += 1; self.critical_count += 1; }
    pub fn add_alert(&mut self) { self.total_alerts += 1; }
    pub fn calculate_score(&self) -> f32 { (100.0 - (self.critical_count as f32 * 10.0)).max(0.0).min(100.0) }
    pub fn total(&self) -> usize { self.total_alerts }
}
