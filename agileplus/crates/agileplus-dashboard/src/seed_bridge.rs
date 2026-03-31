//! Bridges dashboard store construction from seeded feature data.
//! Traceability: WP12 (T074)

use std::collections::HashMap;

use agileplus_domain::domain::{
    cycle::Cycle, module::Module, project::Project,
};

use crate::app_state::DashboardStore;
use crate::seed::seed_dogfood_features;

/// Build a fully-populated [`DashboardStore`] from the dogfood seed data.
///
/// Called by [`DashboardStore::seeded()`](crate::app_state::DashboardStore::seeded).
pub fn build_dashboard_store() -> DashboardStore {
    let (features, work_packages) = seed_dogfood_features();

    let now = chrono::Utc::now();

    // ---- Seeded modules (native dashboard structure) ----
    let modules = vec![
        Module {
            id: 1,
            slug: "core".to_string(),
            friendly_name: "Core".to_string(),
            description: Some("Core platform functionality".to_string()),
            parent_module_id: None,
            created_at: now,
            updated_at: now,
        },
        Module {
            id: 2,
            slug: "kitty-specs".to_string(),
            friendly_name: "Kitty Specs".to_string(),
            description: Some("Kitty specification suite".to_string()),
            parent_module_id: None,
            created_at: now,
            updated_at: now,
        },
        Module {
            id: 3,
            slug: "agents".to_string(),
            friendly_name: "Agents".to_string(),
            description: Some("Agent infrastructure".to_string()),
            parent_module_id: None,
            created_at: now,
            updated_at: now,
        },
    ];

    // ---- Seeded cycles (native dashboard structure) ----
    let cycles = vec![Cycle {
        id: 1,
        name: "Sprint 1".to_string(),
        description: Some("Initial development sprint".to_string()),
        start_date: now.date_naive(),
        end_date: now.date_naive(),
        state: agileplus_domain::domain::cycle::CycleState::Active,
        module_scope_id: None,
        created_at: now,
        updated_at: now,
    }];

    // Map feature IDs to cycle 1 (dogfood association)
    let cycle_features: HashMap<i64, Vec<i64>> = {
        let mut m = HashMap::new();
        m.insert(1, features.iter().map(|f| f.id).collect());
        m
    };

    // ---- Seeded projects ----
    let projects = vec![Project {
        id: 1,
        slug: "agileplus-internal".to_string(),
        name: "AgilePlus Internal".to_string(),
        description: "Internal AgilePlus development project".to_string(),
        created_at: now,
        updated_at: now,
    }];

    // ---- Default service health ----
    let health = crate::app_state::default_health();

    DashboardStore {
        features,
        work_packages,
        modules,
        cycles,
        cycle_features,
        health,
        projects,
        active_project_id: Some(1),
    }
}
