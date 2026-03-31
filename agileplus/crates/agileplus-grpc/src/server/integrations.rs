use tonic::{Response, Status};

use agileplus_domain::domain::backlog::{BacklogFilters, BacklogItem, BacklogPriority, BacklogSort, BacklogStatus, Intent};
use agileplus_domain::ports::{AgentPort, ObservabilityPort, ReviewPort, StoragePort, VcsPort};
use agileplus_proto::agileplus::v1::{
    CreateBacklogItemRequest, CreateBacklogItemResponse, GetBacklogItemRequest,
    GetBacklogItemResponse, ImportBacklogRequest, ImportBacklogResponse, ListBacklogRequest,
    ListBacklogResponse, PopBacklogRequest, PopBacklogResponse, UpdateBacklogStatusRequest,
    UpdateBacklogStatusResponse,
};

use super::{domain_error_to_status, AgilePlusCoreServer};
use crate::conversions::backlog_item_to_proto;

impl<S, V, A, R, O> AgilePlusCoreServer<S, V, A, R, O>
where
    S: StoragePort + 'static,
    V: VcsPort + 'static,
    A: AgentPort + 'static,
    R: ReviewPort + 'static,
    O: ObservabilityPort + 'static,
{
    pub(super) async fn handle_create_backlog_item(
        &self,
        request: CreateBacklogItemRequest,
    ) -> Result<Response<CreateBacklogItemResponse>, Status> {
        let intent = parse_intent(&request.r#type)?;
        let mut item = BacklogItem::from_triage(
            request.title,
            request.description,
            intent,
            if request.source.is_empty() {
                "grpc".to_string()
            } else {
                request.source
            },
        )
        .with_feature_slug((!request.feature_slug.is_empty()).then_some(request.feature_slug))
        .with_tags(request.tags);

        if !request.priority.is_empty() {
            item.priority = parse_priority(&request.priority)?;
        }

        let id = self
            .storage
            .create_backlog_item(&item)
            .await
            .map_err(domain_error_to_status)?;

        let created = BacklogItem {
            id: Some(id),
            ..item
        };

        Ok(Response::new(CreateBacklogItemResponse {
            item: Some(backlog_item_to_proto(created)),
        }))
    }

    pub(super) async fn handle_import_backlog(
        &self,
        request: ImportBacklogRequest,
    ) -> Result<Response<ImportBacklogResponse>, Status> {
        let mut imported = Vec::with_capacity(request.items.len());

        for item in request.items {
            let created = self
                .handle_create_backlog_item(item)
                .await?
                .into_inner()
                .item
                .ok_or_else(|| Status::internal("backlog item missing in create response"))?;
            imported.push(created);
        }

        Ok(Response::new(ImportBacklogResponse { items: imported }))
    }

    pub(super) async fn handle_get_backlog_item(
        &self,
        request: GetBacklogItemRequest,
    ) -> Result<Response<GetBacklogItemResponse>, Status> {
        let item = self
            .storage
            .get_backlog_item(request.backlog_item_id)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| {
                Status::not_found(format!("backlog item {} not found", request.backlog_item_id))
            })?;

        Ok(Response::new(GetBacklogItemResponse {
            item: Some(backlog_item_to_proto(item)),
        }))
    }

    pub(super) async fn handle_list_backlog(
        &self,
        request: ListBacklogRequest,
    ) -> Result<Response<ListBacklogResponse>, Status> {
        let filters = BacklogFilters {
            intent: parse_intent_opt(request.r#type_filter.as_str())?,
            status: parse_status_opt(request.state_filter.as_str())?,
            priority: parse_priority_opt(request.priority_filter.as_str())?,
            feature_slug: (!request.feature_slug.is_empty()).then_some(request.feature_slug),
            source: (!request.source_filter.is_empty()).then_some(request.source_filter),
            limit: (request.limit > 0).then_some(request.limit as usize),
            sort: parse_sort_opt(request.sort.as_str())?,
        };

        let items = self
            .storage
            .list_backlog_items(&filters)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(ListBacklogResponse {
            items: items.into_iter().map(backlog_item_to_proto).collect(),
        }))
    }

    pub(super) async fn handle_update_backlog_status(
        &self,
        request: UpdateBacklogStatusRequest,
    ) -> Result<Response<UpdateBacklogStatusResponse>, Status> {
        let item = self
            .storage
            .get_backlog_item(request.backlog_item_id)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| {
                Status::not_found(format!("backlog item {} not found", request.backlog_item_id))
            })?;

        let target = parse_status(&request.target_status)?;
        self.storage
            .update_backlog_status(request.backlog_item_id, target)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(UpdateBacklogStatusResponse {
            backlog_item_id: request.backlog_item_id,
            from_status: item.status.to_string(),
            to_status: target.to_string(),
        }))
    }

    pub(super) async fn handle_pop_backlog(
        &self,
        request: PopBacklogRequest,
    ) -> Result<Response<PopBacklogResponse>, Status> {
        let mut items = Vec::new();
        let count = request.count;

        if count == 0 {
            return Ok(Response::new(PopBacklogResponse { items }));
        }

        for _ in 0..count {
            match self
                .storage
                .pop_next_backlog_item()
                .await
                .map_err(domain_error_to_status)?
            {
                Some(item) => items.push(backlog_item_to_proto(item)),
                None => break,
            }
        }

        Ok(Response::new(PopBacklogResponse { items }))
    }
}

fn parse_intent(value: &str) -> Result<Intent, Status> {
    let value = if value.is_empty() { "task" } else { value };
    value
        .parse::<Intent>()
        .map_err(Status::invalid_argument)
}

fn parse_intent_opt(value: &str) -> Result<Option<Intent>, Status> {
    if value.is_empty() {
        Ok(None)
    } else {
        parse_intent(value).map(Some)
    }
}

fn parse_priority(value: &str) -> Result<BacklogPriority, Status> {
    value
        .parse::<BacklogPriority>()
        .map_err(Status::invalid_argument)
}

fn parse_priority_opt(value: &str) -> Result<Option<BacklogPriority>, Status> {
    if value.is_empty() {
        Ok(None)
    } else {
        parse_priority(value).map(Some)
    }
}

fn parse_status(value: &str) -> Result<BacklogStatus, Status> {
    value
        .parse::<BacklogStatus>()
        .map_err(Status::invalid_argument)
}

fn parse_status_opt(value: &str) -> Result<Option<BacklogStatus>, Status> {
    if value.is_empty() {
        Ok(None)
    } else {
        parse_status(value).map(Some)
    }
}

fn parse_sort_opt(value: &str) -> Result<BacklogSort, Status> {
    if value.is_empty() {
        Ok(BacklogSort::default())
    } else {
        value
            .parse::<BacklogSort>()
            .map_err(Status::invalid_argument)
    }
}
