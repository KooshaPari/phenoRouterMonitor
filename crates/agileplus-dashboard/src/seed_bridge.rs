use std::collections::HashMap;

use agileplus_domain::domain::cycle::Cycle;
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::module::Module;
use agileplus_domain::domain::project::Project;
use agileplus_domain::domain::work_package::WorkPackage;
use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::app_state::{DashboardStore, default_health};

pub fn build_dashboard_store() -> DashboardStore {
    let bundle = crate::seed::seed_import_bundle();
    let now = Utc::now();

    let mut module_ids = HashMap::new();
    for (index, module_spec) in bundle.modules.iter().enumerate() {
        let id = (index + 1) as i64;
        let slug = module_spec
            .slug
            .clone()
            .unwrap_or_else(|| format!("module-{id}"));
        module_ids.insert(slug, id);
    }

    let modules = bundle
        .modules
        .into_iter()
        .enumerate()
        .map(|(index, module_spec)| {
            let id = (index + 1) as i64;
            let slug = module_spec.slug.unwrap_or_else(|| format!("module-{id}"));
            Module {
                id,
                slug,
                friendly_name: module_spec.friendly_name,
                description: module_spec.description,
                parent_module_id: module_spec
                    .parent_slug
                    .and_then(|parent_slug| module_ids.get(&parent_slug).copied()),
                created_at: now,
                updated_at: now,
            }
        })
        .collect::<Vec<_>>();

    let mut feature_ids = HashMap::new();
    let mut features = Vec::with_capacity(bundle.features.len());
    let mut work_packages = HashMap::new();
    for (index, feature_spec) in bundle.features.into_iter().enumerate() {
        let id = (index + 1) as i64;
        let slug = feature_spec
            .slug
            .clone()
            .unwrap_or_else(|| Feature::slug_from_name(&feature_spec.friendly_name));
        feature_ids.insert(slug.clone(), id);

        let mut feature = Feature::new(
            &slug,
            &feature_spec.friendly_name,
            hash_spec_content(&feature_spec.spec_content),
            feature_spec.target_branch.as_deref(),
        );
        feature.id = id;
        feature.state = feature_spec.state;
        feature.labels = feature_spec.labels;
        feature.module_id = feature_spec
            .module_slug
            .and_then(|module_slug| module_ids.get(&module_slug).copied());
        feature.project_id = feature_spec.project_id;
        feature.plane_issue_id = feature_spec.plane_issue_id;
        feature.plane_state_id = feature_spec.plane_state_id;
        features.push(feature);

        let mut wp_list = Vec::with_capacity(feature_spec.work_packages.len());
        for (wp_index, wp_spec) in feature_spec.work_packages.into_iter().enumerate() {
            wp_list.push(WorkPackage {
                id: (index as i64) * 100 + wp_index as i64 + 1,
                feature_id: id,
                title: wp_spec.title,
                state: wp_spec.state,
                sequence: wp_spec.sequence.unwrap_or((wp_index as i32) + 1),
                file_scope: wp_spec.file_scope,
                acceptance_criteria: wp_spec.acceptance_criteria.unwrap_or_default(),
                agent_id: wp_spec.agent_id,
                pr_url: wp_spec.pr_url,
                pr_state: wp_spec.pr_state,
                worktree_path: wp_spec.worktree_path,
                plane_sub_issue_id: wp_spec.plane_sub_issue_id,
                created_at: now,
                updated_at: now,
            });
        }
        work_packages.insert(id, wp_list);
    }

    let mut cycles = Vec::with_capacity(bundle.cycles.len());
    let mut cycle_features = HashMap::new();
    for (index, cycle_spec) in bundle.cycles.into_iter().enumerate() {
        let id = (index + 1) as i64;
        cycles.push(Cycle {
            id,
            name: cycle_spec.name,
            description: cycle_spec.description,
            state: cycle_spec.state,
            start_date: cycle_spec.start_date,
            end_date: cycle_spec.end_date,
            module_scope_id: cycle_spec
                .module_scope_slug
                .and_then(|module_slug| module_ids.get(&module_slug).copied()),
            created_at: now,
            updated_at: now,
        });
        cycle_features.insert(
            id,
            cycle_spec
                .feature_slugs
                .into_iter()
                .filter_map(|slug| feature_ids.get(&slug).copied())
                .collect(),
        );
    }

    DashboardStore {
        features,
        work_packages,
        modules,
        cycles,
        cycle_features,
        health: default_health(),
        projects: default_projects(),
        active_project_id: None,
    }
}

fn default_projects() -> Vec<Project> {
    vec![
        Project::with_id(
            1,
            "agileplus",
            "AgilePlus",
            "Spec-driven development platform",
        ),
        Project::with_id(
            2,
            "bifrost-extensions",
            "Bifrost Extensions",
            "Clean extension layer for Bifrost LLM gateway",
        ),
        Project::with_id(
            3,
            "cliproxyapi-plusplus",
            "CLIProxy API++",
            "CLI proxy with third-party provider support",
        ),
        Project::with_id(
            4,
            "agentapi-plusplus",
            "AgentAPI++",
            "AgentAPI fork with provider support and OAuth",
        ),
        Project::with_id(
            5,
            "colab",
            "Colab",
            "Hybrid web browser and local code editor",
        ),
        Project::with_id(6, "helios", "Helios", "Helios application and CLI"),
        Project::with_id(
            7,
            "thegent",
            "TheGent",
            "Agent orchestration, governance, and lifecycle framework",
        ),
        Project::with_id(
            8,
            "tokenledger",
            "TokenLedger",
            "Token management and pricing governance for AI agents",
        ),
        Project::with_id(
            9,
            "trace",
            "Trace",
            "Multi-view requirements traceability system",
        ),
        Project::with_id(
            10,
            "phenotype-config",
            "Phenotype Config",
            "Local-first config, feature flags, and secrets",
        ),
        Project::with_id(
            11,
            "phenotype-infra",
            "Phenotype Infra",
            "Shared actions, design tokens, and doc federation",
        ),
        Project::with_id(
            12,
            "portage",
            "Portage",
            "Agent and LLM evaluation framework",
        ),
        Project::with_id(
            13,
            "civ",
            "Civ",
            "Deterministic simulation and policy architecture",
        ),
    ]
}

fn hash_spec_content(spec_content: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(spec_content.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&digest);
    bytes
}
