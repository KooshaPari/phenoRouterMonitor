use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::application::{EntityService, CreateEntityDto, UpdateEntityDto};
use crate::domain::{Entity, DomainError};

pub struct AppState<S: EntityService> {
    pub entity_service: S,
}

pub fn create_routes<S: EntityService + 'static>(service: S) -> Router {
    let state = Arc::new(AppState {
        entity_service: service,
    });

    Router::new()
        .route("/health", get(health_check))
        .route("/entities", post(create_entity).get(list_entities))
        .route("/entities/:id", get(get_entity).put(update_entity).delete(delete_entity))
        .with_state(state)
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn create_entity<S: EntityService>(
    State(state): State<Arc<AppState<S>>>,
    Json(dto): Json<CreateEntityDto>,
) -> Result<Json<Entity>, (StatusCode, String)> {
    state.entity_service.create(dto).await
        .map(Json)
        .map_err(map_domain_error)
}

async fn get_entity<S: EntityService>(
    State(state): State<Arc<AppState<S>>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Entity>, (StatusCode, String)> {
    state.entity_service.get_by_id(id).await
        .map(Json)
        .map_err(map_domain_error)
}

async fn update_entity<S: EntityService>(
    State(state): State<Arc<AppState<S>>>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateEntityDto>,
) -> Result<Json<Entity>, (StatusCode, String)> {
    state.entity_service.update(id, dto).await
        .map(Json)
        .map_err(map_domain_error)
}

async fn delete_entity<S: EntityService>(
    State(state): State<Arc<AppState<S>>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.entity_service.delete(id).await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(map_domain_error)
}

async fn list_entities<S: EntityService>(
    State(state): State<Arc<AppState<S>>>,
) -> Result<Json<Vec<Entity>>, (StatusCode, String)> {
    state.entity_service.list().await
        .map(Json)
        .map_err(map_domain_error)
}

fn map_domain_error(e: DomainError) -> (StatusCode, String) {
    match e {
        DomainError::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
        DomainError::InvalidInput(_) => (StatusCode::BAD_REQUEST, e.to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}
