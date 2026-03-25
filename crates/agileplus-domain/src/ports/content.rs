use std::future::Future;

use crate::domain::backlog::{BacklogFilters, BacklogItem, BacklogPriority, BacklogStatus};
use crate::domain::feature::Feature;
use crate::domain::state_machine::FeatureState;
use crate::domain::work_package::{WorkPackage, WpDependency, WpState};
use crate::error::DomainError;

/// Content-storage operations for features, backlog, and work packages.
pub trait ContentStoragePort: Send + Sync {
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

    /// Persist the mutable feature fields for an existing feature.
    fn update_feature(
        &self,
        feature: &Feature,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// List all features currently in the given state.
    fn list_features_by_state(
        &self,
        state: FeatureState,
    ) -> impl Future<Output = Result<Vec<Feature>, DomainError>> + Send;

    /// List every feature in the system.
    fn list_all_features(&self) -> impl Future<Output = Result<Vec<Feature>, DomainError>> + Send;

    /// Create a new backlog item, returning its assigned ID.
    fn create_backlog_item(
        &self,
        item: &BacklogItem,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send;

    /// Look up a backlog item by primary key.
    fn get_backlog_item(
        &self,
        id: i64,
    ) -> impl Future<Output = Result<Option<BacklogItem>, DomainError>> + Send;

    /// List backlog items using the shared queue filter model.
    fn list_backlog_items(
        &self,
        filters: &BacklogFilters,
    ) -> impl Future<Output = Result<Vec<BacklogItem>, DomainError>> + Send;

    /// Update a backlog item's status.
    fn update_backlog_status(
        &self,
        id: i64,
        status: BacklogStatus,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Update a backlog item's priority.
    fn update_backlog_priority(
        &self,
        id: i64,
        priority: BacklogPriority,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Pop the next highest-priority backlog item, marking it triaged.
    fn pop_next_backlog_item(
        &self,
    ) -> impl Future<Output = Result<Option<BacklogItem>, DomainError>> + Send;

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

    /// Persist the mutable work-package fields for an existing work package.
    fn update_work_package(
        &self,
        wp: &WorkPackage,
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
}
