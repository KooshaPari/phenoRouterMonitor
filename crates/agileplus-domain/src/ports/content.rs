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

    fn update_feature(
        &self,
        feature: &Feature,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn list_features_by_state(
        &self,
        state: FeatureState,
    ) -> impl std::future::Future<Output = Result<Vec<Feature>, DomainError>> + Send;

    fn list_all_features(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<Feature>, DomainError>> + Send;

    fn create_backlog_item(
        &self,
        item: &BacklogItem,
    ) -> impl std::future::Future<Output = Result<i64, DomainError>> + Send;

    fn get_backlog_item(
        &self,
        id: i64,
    ) -> impl std::future::Future<Output = Result<Option<BacklogItem>, DomainError>> + Send;

    fn list_backlog_items(
        &self,
        filters: &BacklogFilters,
    ) -> impl std::future::Future<Output = Result<Vec<BacklogItem>, DomainError>> + Send;

    fn update_backlog_status(
        &self,
        id: i64,
        status: BacklogStatus,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn update_backlog_priority(
        &self,
        id: i64,
        priority: BacklogPriority,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn pop_next_backlog_item(
        &self,
    ) -> impl std::future::Future<Output = Result<Option<BacklogItem>, DomainError>> + Send;

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

    fn update_work_package(
        &self,
        wp: &WorkPackage,
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
}
