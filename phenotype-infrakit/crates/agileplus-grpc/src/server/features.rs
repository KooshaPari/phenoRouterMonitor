use tonic::{Response, Status};

use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::ports::{AgentPort, ObservabilityPort, ReviewPort, StoragePort, VcsPort};
use agileplus_proto::agileplus::v1::{
    GetFeatureRequest, GetFeatureResponse, GetFeatureStateRequest, GetFeatureStateResponse,
    ListFeaturesRequest, ListFeaturesResponse,
};

use super::{AgilePlusCoreServer, domain_error_to_status};
use crate::conversions::feature_to_proto;

impl<S, V, A, R, O> AgilePlusCoreServer<S, V, A, R, O>
where
    S: StoragePort + 'static,
    V: VcsPort + 'static,
    A: AgentPort + 'static,
    R: ReviewPort + 'static,
    O: ObservabilityPort + 'static,
{
    pub(super) async fn handle_get_feature(
        &self,
        request: GetFeatureRequest,
    ) -> Result<Response<GetFeatureResponse>, Status> {
        let slug = request.slug;
        match self.storage.get_feature_by_slug(&slug).await {
            Ok(Some(feature)) => Ok(Response::new(GetFeatureResponse {
                feature: Some(feature_to_proto(feature)),
            })),
            Ok(None) => Err(Status::not_found(format!("feature '{slug}' not found"))),
            Err(e) => Err(domain_error_to_status(e)),
        }
    }

    pub(super) async fn handle_list_features(
        &self,
        request: ListFeaturesRequest,
    ) -> Result<Response<ListFeaturesResponse>, Status> {
        let features = if request.state_filter.is_empty() {
            self.storage
                .list_all_features()
                .await
                .map_err(domain_error_to_status)?
        } else {
            let state: FeatureState = request
                .state_filter
                .parse()
                .map_err(|e: String| Status::invalid_argument(e))?;
            self.storage
                .list_features_by_state(state)
                .await
                .map_err(domain_error_to_status)?
        };

        Ok(Response::new(ListFeaturesResponse {
            features: features.into_iter().map(feature_to_proto).collect(),
        }))
    }

    pub(super) async fn handle_get_feature_state(
        &self,
        request: GetFeatureStateRequest,
    ) -> Result<Response<GetFeatureStateResponse>, Status> {
        let slug = request.slug;
        let feature = self
            .storage
            .get_feature_by_slug(&slug)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| Status::not_found(format!("feature '{slug}' not found")))?;

        let next_command = match feature.state {
            FeatureState::Created => "specify",
            FeatureState::Specified => "research",
            FeatureState::Researched => "plan",
            FeatureState::Planned => "implement",
            FeatureState::Implementing => "validate",
            FeatureState::Validated => "ship",
            FeatureState::Shipped => "retrospective",
            FeatureState::Retrospected => "",
        };

        Ok(Response::new(GetFeatureStateResponse {
            feature_state: Some(agileplus_proto::agileplus::v1::FeatureState {
                state: feature.state.to_string(),
                next_command: next_command.to_string(),
                blockers: Vec::new(),
                governance: None,
            }),
        }))
    }
}
