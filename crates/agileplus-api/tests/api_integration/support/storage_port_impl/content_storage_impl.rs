use std::future::Future;

use agileplus_domain::domain::backlog::{
    BacklogFilters, BacklogItem, BacklogPriority, BacklogStatus,
};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{WorkPackage, WpDependency, WpState};
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::storage::ContentStoragePort;

use super::super::storage::MockStorage;
use super::{backlog, feature, work_package};

impl ContentStoragePort for MockStorage {
    fn create_feature(&self, f: &Feature) -> impl Future<Output = Result<i64, DomainError>> + Send {
        feature::create_feature(self, f)
    }

    fn get_feature_by_slug(
        &self,
        slug: &str,
    ) -> impl Future<Output = Result<Option<Feature>, DomainError>> + Send {
        feature::get_feature_by_slug(self, slug)
    }

    fn get_feature_by_id(
        &self,
        id: i64,
    ) -> impl Future<Output = Result<Option<Feature>, DomainError>> + Send {
        feature::get_feature_by_id(self, id)
    }

    fn update_feature_state(
        &self,
        id: i64,
        state: FeatureState,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        feature::update_feature_state(self, id, state)
    }

    fn update_feature(
        &self,
        feature: &Feature,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        feature::update_feature(self, feature)
    }

    fn list_features_by_state(
        &self,
        state: FeatureState,
    ) -> impl Future<Output = Result<Vec<Feature>, DomainError>> + Send {
        feature::list_features_by_state(self, state)
    }

    fn list_all_features(&self) -> impl Future<Output = Result<Vec<Feature>, DomainError>> + Send {
        feature::list_all_features(self)
    }

    fn get_backlog_item(
        &self,
        id: i64,
    ) -> impl Future<Output = Result<Option<BacklogItem>, DomainError>> + Send {
        backlog::get_backlog_item(self, id)
    }

    fn list_backlog_items(
        &self,
        filters: &BacklogFilters,
    ) -> impl Future<Output = Result<Vec<BacklogItem>, DomainError>> + Send {
        backlog::list_backlog_items(self, filters)
    }

    fn create_backlog_item(
        &self,
        item: &BacklogItem,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send {
        backlog::create_backlog_item(self, item)
    }

    fn update_backlog_status(
        &self,
        id: i64,
        status: BacklogStatus,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        backlog::update_backlog_status(self, id, status)
    }

    fn update_backlog_priority(
        &self,
        id: i64,
        priority: BacklogPriority,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        backlog::update_backlog_priority(self, id, priority)
    }

    fn pop_next_backlog_item(
        &self,
    ) -> impl Future<Output = Result<Option<BacklogItem>, DomainError>> + Send {
        backlog::pop_next_backlog_item(self)
    }

    fn create_work_package(
        &self,
        wp: &WorkPackage,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send {
        work_package::create_work_package(self, wp)
    }

    fn get_work_package(
        &self,
        id: i64,
    ) -> impl Future<Output = Result<Option<WorkPackage>, DomainError>> + Send {
        work_package::get_work_package(self, id)
    }

    fn update_wp_state(
        &self,
        id: i64,
        state: WpState,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        work_package::update_wp_state(self, id, state)
    }

    fn update_work_package(
        &self,
        wp: &WorkPackage,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        work_package::update_work_package(self, wp)
    }

    fn list_wps_by_feature(
        &self,
        feature_id: i64,
    ) -> impl Future<Output = Result<Vec<WorkPackage>, DomainError>> + Send {
        work_package::list_wps_by_feature(self, feature_id)
    }

    fn add_wp_dependency(
        &self,
        dep: &WpDependency,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        work_package::add_wp_dependency(self, dep)
    }

    fn get_wp_dependencies(
        &self,
        wp_id: i64,
    ) -> impl Future<Output = Result<Vec<WpDependency>, DomainError>> + Send {
        work_package::get_wp_dependencies(self, wp_id)
    }

    fn get_ready_wps(
        &self,
        feature_id: i64,
    ) -> impl Future<Output = Result<Vec<WorkPackage>, DomainError>> + Send {
        work_package::get_ready_wps(self, feature_id)
    }
}
