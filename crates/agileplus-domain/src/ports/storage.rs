//! Storage port -- persistence abstraction for all domain entities.
//!
//! Traceability: FR-STORE-* / WP05-T025

use crate::domain::audit::AuditEntry;
use crate::domain::cycle::{Cycle, CycleFeature, CycleState, CycleWithFeatures};
use crate::domain::feature::Feature;
use crate::domain::governance::{Evidence, GovernanceContract, PolicyRule};
use crate::domain::metric::Metric;
use crate::domain::module::{Module, ModuleFeatureTag, ModuleWithFeatures};
use crate::domain::project::Project;
use crate::domain::state_machine::FeatureState;
use crate::domain::sync_mapping::SyncMapping;
use crate::domain::work_package::{WorkPackage, WpDependency, WpState};
use crate::error::DomainError;

/// Port for persistent storage operations.
///
/// Implementations provide CRUD access to all domain entities.
/// The SQLite adapter (WP06) is the primary implementation.
pub trait StoragePort: Send + Sync {
    // -- Feature CRUD --

    /// Create a new feature, returning its assigned ID.
    fn create_feature(
        &self,
        feature: &Feature,
    ) -> impl std::future::Future<Output = Result<i64, DomainError>> + Send;

    fn get_feature_by_slug(
        &self,
        slug: &str,
    ) -> impl std::future::Future<Output = Result<Option<Feature>, DomainError>> + Send;

    fn get_feature_by_id(
        &self,
        id: i64,
    ) -> impl std::future::Future<Output = Result<Option<Feature>, DomainError>> + Send;

    fn update_feature_state(
        &self,
        id: i64,
        state: FeatureState,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn list_features_by_state(
        &self,
        state: FeatureState,
    ) -> impl std::future::Future<Output = Result<Vec<Feature>, DomainError>> + Send;

    fn list_all_features(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<Feature>, DomainError>> + Send;

    fn create_work_package(
        &self,
        wp: &WorkPackage,
    ) -> impl std::future::Future<Output = Result<i64, DomainError>> + Send;

    fn get_work_package(
        &self,
        id: i64,
    ) -> impl std::future::Future<Output = Result<Option<WorkPackage>, DomainError>> + Send;

    fn update_wp_state(
        &self,
        id: i64,
        state: WpState,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn list_wps_by_feature(
        &self,
        feature_id: i64,
    ) -> impl std::future::Future<Output = Result<Vec<WorkPackage>, DomainError>> + Send;

    fn add_wp_dependency(
        &self,
        dep: &WpDependency,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn get_wp_dependencies(
        &self,
        wp_id: i64,
    ) -> impl std::future::Future<Output = Result<Vec<WpDependency>, DomainError>> + Send;

    fn get_ready_wps(
        &self,
        feature_id: i64,
    ) -> impl std::future::Future<Output = Result<Vec<WorkPackage>, DomainError>> + Send;

    fn append_audit_entry(
        &self,
        entry: &AuditEntry,
    ) -> impl std::future::Future<Output = Result<i64, DomainError>> + Send;

    fn get_audit_trail(
        &self,
        feature_id: i64,
    ) -> impl std::future::Future<Output = Result<Vec<AuditEntry>, DomainError>> + Send;

    fn get_latest_audit_entry(
        &self,
        feature_id: i64,
    ) -> impl std::future::Future<Output = Result<Option<AuditEntry>, DomainError>> + Send;

    fn create_evidence(
        &self,
        evidence: &Evidence,
    ) -> impl std::future::Future<Output = Result<i64, DomainError>> + Send;

    fn get_evidence_by_wp(
        &self,
        wp_id: i64,
    ) -> impl std::future::Future<Output = Result<Vec<Evidence>, DomainError>> + Send;

    fn get_evidence_by_fr(
        &self,
        fr_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Evidence>, DomainError>> + Send;

    fn create_policy_rule(
        &self,
        rule: &PolicyRule,
    ) -> impl std::future::Future<Output = Result<i64, DomainError>> + Send;

    fn list_active_policies(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<PolicyRule>, DomainError>> + Send;

    fn record_metric(
        &self,
        metric: &Metric,
    ) -> impl std::future::Future<Output = Result<i64, DomainError>> + Send;

    fn get_metrics_by_feature(
        &self,
        feature_id: i64,
    ) -> impl std::future::Future<Output = Result<Vec<Metric>, DomainError>> + Send;

    fn create_governance_contract(
        &self,
        contract: &GovernanceContract,
    ) -> impl std::future::Future<Output = Result<i64, DomainError>> + Send;

    fn get_governance_contract(
        &self,
        feature_id: i64,
        version: i32,
    ) -> impl std::future::Future<Output = Result<Option<GovernanceContract>, DomainError>> + Send;

    fn get_latest_governance_contract(
        &self,
        feature_id: i64,
    ) -> impl std::future::Future<Output = Result<Option<GovernanceContract>, DomainError>> + Send;

    fn create_module(
        &self,
        module: &Module,
    ) -> impl std::future::Future<Output = Result<i64, DomainError>> + Send;

    fn get_module(
        &self,
        id: i64,
    ) -> impl std::future::Future<Output = Result<Option<Module>, DomainError>> + Send;

    fn get_module_by_slug(
        &self,
        slug: &str,
    ) -> impl std::future::Future<Output = Result<Option<Module>, DomainError>> + Send;

    fn update_module(
        &self,
        id: i64,
        friendly_name: &str,
        description: Option<&str>,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn delete_module(
        &self,
        id: i64,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn list_root_modules(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<Module>, DomainError>> + Send;

    fn list_child_modules(
        &self,
        parent_id: i64,
    ) -> impl std::future::Future<Output = Result<Vec<Module>, DomainError>> + Send;

    fn get_module_with_features(
        &self,
        id: i64,
    ) -> impl std::future::Future<Output = Result<Option<ModuleWithFeatures>, DomainError>> + Send;

    fn create_cycle(
        &self,
        cycle: &Cycle,
    ) -> impl std::future::Future<Output = Result<i64, DomainError>> + Send;

    fn get_cycle(
        &self,
        id: i64,
    ) -> impl std::future::Future<Output = Result<Option<Cycle>, DomainError>> + Send;

    fn update_cycle_state(
        &self,
        id: i64,
        state: CycleState,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn list_cycles_by_state(
        &self,
        state: CycleState,
    ) -> impl std::future::Future<Output = Result<Vec<Cycle>, DomainError>> + Send;

    fn list_cycles_by_module(
        &self,
        module_id: i64,
    ) -> impl std::future::Future<Output = Result<Vec<Cycle>, DomainError>> + Send;

    fn list_all_cycles(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<Cycle>, DomainError>> + Send;

    fn get_cycle_with_features(
        &self,
        id: i64,
    ) -> impl std::future::Future<Output = Result<Option<CycleWithFeatures>, DomainError>> + Send;

    fn tag_feature_to_module(
        &self,
        tag: &ModuleFeatureTag,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn untag_feature_from_module(
        &self,
        module_id: i64,
        feature_id: i64,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn add_feature_to_cycle(
        &self,
        entry: &CycleFeature,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn remove_feature_from_cycle(
        &self,
        cycle_id: i64,
        feature_id: i64,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn get_sync_mapping(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> impl std::future::Future<Output = Result<Option<SyncMapping>, DomainError>> + Send;

    fn upsert_sync_mapping(
        &self,
        mapping: &SyncMapping,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn get_sync_mapping_by_plane_id(
        &self,
        entity_type: &str,
        plane_issue_id: &str,
    ) -> impl std::future::Future<Output = Result<Option<SyncMapping>, DomainError>> + Send;

    fn delete_sync_mapping(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    // -- Project CRUD --

    /// Create a new project, returning its assigned ID.
    fn create_project(
        &self,
        project: &Project,
    ) -> impl std::future::Future<Output = Result<i64, DomainError>> + Send;

    /// Look up a project by its slug. Returns None if not found.
    fn get_project_by_slug(
        &self,
        slug: &str,
    ) -> impl std::future::Future<Output = Result<Option<Project>, DomainError>> + Send;
}
