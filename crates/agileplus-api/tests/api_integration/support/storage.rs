use std::sync::{Arc, Mutex};

use agileplus_domain::domain::audit::{AuditEntry, hash_entry};
use agileplus_domain::domain::project::Project;
use agileplus_domain::domain::backlog::BacklogItem;
use agileplus_domain::domain::cycle::{Cycle, CycleFeature};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::governance::GovernanceContract;
use agileplus_domain::domain::module::{Module, ModuleFeatureTag};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{WorkPackage, WpState};
use chrono::Utc;

#[derive(Default, Clone)]
pub(crate) struct MockStorage {
    pub(crate) features: Arc<Mutex<Vec<Feature>>>,
    pub(crate) work_packages: Arc<Mutex<Vec<WorkPackage>>>,
    pub(crate) backlog: Arc<Mutex<Vec<BacklogItem>>>,
    pub(crate) modules: Arc<Mutex<Vec<Module>>>,
    pub(crate) cycles: Arc<Mutex<Vec<Cycle>>>,
    pub(crate) module_tags: Arc<Mutex<Vec<ModuleFeatureTag>>>,
    pub(crate) cycle_features: Arc<Mutex<Vec<CycleFeature>>>,
    pub(crate) governance: Arc<Mutex<Vec<GovernanceContract>>>,
    pub(crate) audit: Arc<Mutex<Vec<AuditEntry>>>,
    pub(crate) projects: Arc<Mutex<Vec<Project>>>,
}

impl MockStorage {
    pub(crate) fn with_test_data() -> Self {
        let s = MockStorage::default();
        let now = Utc::now();

        s.features
            .lock()
            .expect("features lock poisoned")
            .push(Feature {
                id: 1,
                slug: "test-feature".to_string(),
                friendly_name: "Test Feature".to_string(),
                state: FeatureState::Implementing,
                spec_hash: [0u8; 32],
                target_branch: "main".to_string(),
                plane_issue_id: None,
                plane_state_id: None,
                labels: vec![],
                module_id: None,
                project_id: None,
                created_at: now,
                updated_at: now,
            });

        s.work_packages
            .lock()
            .expect("work_packages lock poisoned")
            .push(WorkPackage {
                id: 1,
                feature_id: 1,
                title: "WP01".to_string(),
                state: WpState::Done,
                sequence: 1,
                file_scope: vec![],
                acceptance_criteria: "All tests pass".to_string(),
                agent_id: None,
                pr_url: Some("https://github.com/org/repo/pull/1".to_string()),
                pr_state: None,
                worktree_path: None,
                plane_sub_issue_id: None,
                created_at: now,
                updated_at: now,
            });

        // Build a valid 2-entry audit chain.
        let genesis = AuditEntry {
            id: 1,
            feature_id: 1,
            wp_id: None,
            timestamp: now,
            actor: "system".to_string(),
            transition: "created".to_string(),
            evidence_refs: vec![],
            prev_hash: [0u8; 32],
            hash: [0u8; 32], // fixed below
            event_id: None,
            archived_to: None,
        };
        let genesis_hash = hash_entry(&genesis);
        let genesis = AuditEntry {
            hash: genesis_hash,
            ..genesis
        };

        let second = AuditEntry {
            id: 2,
            feature_id: 1,
            wp_id: Some(1),
            timestamp: now,
            actor: "agent".to_string(),
            transition: "specified".to_string(),
            evidence_refs: vec![],
            prev_hash: genesis_hash,
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        };
        let second_hash = hash_entry(&second);
        let second = AuditEntry {
            hash: second_hash,
            ..second
        };
        s.audit
            .lock()
            .expect("audit lock poisoned")
            .extend([genesis, second]);

        s.governance
            .lock()
            .expect("governance lock poisoned")
            .push(GovernanceContract {
                id: 1,
                feature_id: 1,
                version: 1,
                rules: vec![],
                bound_at: now,
            });

        s
    }
}
