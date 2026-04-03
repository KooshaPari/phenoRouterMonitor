use tonic::{Response, Status};

use agileplus_domain::ports::{AgentPort, ObservabilityPort, ReviewPort, StoragePort, VcsPort};
use agileplus_proto::agileplus::v1::{
    GetWorkPackageStatusRequest, GetWorkPackageStatusResponse, ListWorkPackagesRequest,
    ListWorkPackagesResponse,
};

use super::{AgilePlusCoreServer, domain_error_to_status};
use crate::conversions::wp_to_proto;

impl<S, V, A, R, O> AgilePlusCoreServer<S, V, A, R, O>
where
    S: StoragePort + 'static,
    V: VcsPort + 'static,
    A: AgentPort + 'static,
    R: ReviewPort + 'static,
    O: ObservabilityPort + 'static,
{
    pub(super) async fn handle_list_work_packages(
        &self,
        request: ListWorkPackagesRequest,
    ) -> Result<Response<ListWorkPackagesResponse>, Status> {
        let feature = self
            .storage
            .get_feature_by_slug(&request.feature_slug)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| {
                Status::not_found(format!("feature '{}' not found", request.feature_slug))
            })?;

        let work_packages = self
            .storage
            .list_wps_by_feature(feature.id)
            .await
            .map_err(domain_error_to_status)?;

        let packages = if request.state_filter.is_empty() {
            work_packages
        } else {
            work_packages
                .into_iter()
                .filter(|wp| {
                    format!("{:?}", wp.state).to_lowercase() == request.state_filter.to_lowercase()
                })
                .collect()
        };

        Ok(Response::new(ListWorkPackagesResponse {
            packages: packages.into_iter().map(wp_to_proto).collect(),
        }))
    }

    pub(super) async fn handle_get_work_package_status(
        &self,
        request: GetWorkPackageStatusRequest,
    ) -> Result<Response<GetWorkPackageStatusResponse>, Status> {
        let feature = self
            .storage
            .get_feature_by_slug(&request.feature_slug)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| {
                Status::not_found(format!("feature '{}' not found", request.feature_slug))
            })?;

        let work_package = self
            .storage
            .list_wps_by_feature(feature.id)
            .await
            .map_err(domain_error_to_status)?
            .into_iter()
            .find(|wp| wp.sequence == request.wp_sequence)
            .ok_or_else(|| {
                Status::not_found(format!("WP sequence {} not found", request.wp_sequence))
            })?;

        Ok(Response::new(GetWorkPackageStatusResponse {
            work_package_status: Some(wp_to_proto(work_package)),
        }))
    }
}
