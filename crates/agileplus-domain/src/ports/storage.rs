//! Storage port — persistence abstraction for all domain entities.
//!
//! Traceability: FR-STORE-* / WP05-T025

use std::future::Future;

use crate::domain::audit::AuditEntry;
use crate::domain::feature::Feature;
use crate::domain::governance::{Evidence, GovernanceContract, PolicyRule};
use crate::domain::metric::Metric;
use crate::domain::state_machine::FeatureState;
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
    ) -> impl Future<Output = Result<i64, DomainError>> + Send;

    /// Look up a feature by its unique slug.
    fn get_feature_by_slug(
        &self,
        slug: &str,
    ) -> impl Future<Output = Result<Option<Feature>, DomainError>> + Send;

    /// Look up a feature by its primary key.
    fn get_feature_by_id(
        &self,
        id: i64,
    ) -> impl Future<Output = Result<Option<Feature>, DomainError>> + Send;

    /// Update only the state field of an existing feature.
    fn update_feature_state(
        &self,
        id: i64,
        state: FeatureState,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// List all features currently in the given state.
    fn list_features_by_state(
        &self,
        state: FeatureState,
    ) -> impl Future<Output = Result<Vec<Feature>, DomainError>> + Send;

    /// List every feature in the system.
    fn list_all_features(&self) -> impl Future<Output = Result<Vec<Feature>, DomainError>> + Send;

    // -- Work Package CRUD --

    /// Create a new work package, returning its assigned ID.
    fn create_work_package(
        &self,
        wp: &WorkPackage,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send;

    /// Look up a work package by primary key.
    fn get_work_package(
        &self,
        id: i64,
    ) -> impl Future<Output = Result<Option<WorkPackage>, DomainError>> + Send;

    /// Update only the state field of a work package.
    fn update_wp_state(
        &self,
        id: i64,
        state: WpState,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// List all work packages belonging to a feature.
    fn list_wps_by_feature(
        &self,
        feature_id: i64,
    ) -> impl Future<Output = Result<Vec<WorkPackage>, DomainError>> + Send;

    /// Record a dependency between two work packages.
    fn add_wp_dependency(
        &self,
        dep: &WpDependency,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Get all dependencies for a given work package.
    fn get_wp_dependencies(
        &self,
        wp_id: i64,
    ) -> impl Future<Output = Result<Vec<WpDependency>, DomainError>> + Send;

    /// Get work packages whose dependencies are all in `Done` state.
    fn get_ready_wps(
        &self,
        feature_id: i64,
    ) -> impl Future<Output = Result<Vec<WorkPackage>, DomainError>> + Send;

    // -- Audit CRUD --

    /// Append an audit entry to the immutable log, returning its ID.
    fn append_audit_entry(
        &self,
        entry: &AuditEntry,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send;

    /// Retrieve the full audit trail for a feature, ordered by timestamp.
    fn get_audit_trail(
        &self,
        feature_id: i64,
    ) -> impl Future<Output = Result<Vec<AuditEntry>, DomainError>> + Send;

    /// Get the most recent audit entry for a feature.
    fn get_latest_audit_entry(
        &self,
        feature_id: i64,
    ) -> impl Future<Output = Result<Option<AuditEntry>, DomainError>> + Send;

    // -- Evidence + Policy + Metric CRUD --

    /// Store a new piece of evidence, returning its ID.
    fn create_evidence(
        &self,
        evidence: &Evidence,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send;

    /// Get all evidence associated with a work package.
    fn get_evidence_by_wp(
        &self,
        wp_id: i64,
    ) -> impl Future<Output = Result<Vec<Evidence>, DomainError>> + Send;

    /// Get all evidence satisfying a given functional requirement.
    fn get_evidence_by_fr(
        &self,
        fr_id: &str,
    ) -> impl Future<Output = Result<Vec<Evidence>, DomainError>> + Send;

    /// Create a new policy rule, returning its ID.
    fn create_policy_rule(
        &self,
        rule: &PolicyRule,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send;

    /// List all currently active policy rules.
    fn list_active_policies(
        &self,
    ) -> impl Future<Output = Result<Vec<PolicyRule>, DomainError>> + Send;

    /// Record a command-execution metric, returning its ID.
    fn record_metric(
        &self,
        metric: &Metric,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send;

    /// Get all metrics associated with a feature.
    fn get_metrics_by_feature(
        &self,
        feature_id: i64,
    ) -> impl Future<Output = Result<Vec<Metric>, DomainError>> + Send;

    // -- Governance --

    /// Store a governance contract, returning its ID.
    fn create_governance_contract(
        &self,
        contract: &GovernanceContract,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send;

    /// Look up a specific version of a governance contract for a feature.
    fn get_governance_contract(
        &self,
        feature_id: i64,
        version: i32,
    ) -> impl Future<Output = Result<Option<GovernanceContract>, DomainError>> + Send;

    /// Get the latest governance contract for a feature.
    fn get_latest_governance_contract(
        &self,
        feature_id: i64,
    ) -> impl Future<Output = Result<Option<GovernanceContract>, DomainError>> + Send;
}
