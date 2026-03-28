//! Bridges dashboard store construction from seeded feature data.
//! Traceability: WP12 (T074)

/// Build a fully-populated [`DashboardStore`] from the dogfood seed data.
pub fn build_dashboard_store() -> crate::app_state::DashboardStore {
    crate::app_state::DashboardStore::seeded()
}
