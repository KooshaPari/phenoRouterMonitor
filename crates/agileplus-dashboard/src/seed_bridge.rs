//! Bridges dashboard store construction from seeded feature data.
//! Traceability: WP12 (T074)

use std::collections::HashMap;

use agileplus_domain::domain::{
    cycle::Cycle, module::Module, project::Project,
};

use crate::app_state::DashboardStore;
use crate::seed::seed_dogfood_features;

/// Build a fully-populated [`DashboardStore`] from the dogfood seed data.
pub fn build_dashboard_store() -> crate::app_state::DashboardStore {
    crate::app_state::DashboardStore::seeded()
}
