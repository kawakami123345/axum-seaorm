use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use usecase::dtos::publisher::{PublisherCreateDto, PublisherUpdateDto};
use usecase::map_error;

use crate::handlers::AppState;

pub async fn get_publishers(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.publisher_usecase.get_all_publishers().await {
        Ok(publishers) => (StatusCode::OK, Json(publishers)).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

pub async fn get_publisher(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state.publisher_usecase.get_publisher(id).await {
        Ok(publisher) => (StatusCode::OK, Json(publisher)).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

pub async fn create_publisher(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PublisherCreateDto>,
) -> impl IntoResponse {
    match state.publisher_usecase.create_publisher(payload).await {
        Ok(publisher) => (StatusCode::CREATED, Json(publisher)).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

pub async fn update_publisher(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(mut payload): Json<PublisherUpdateDto>,
) -> impl IntoResponse {
    payload.id = id;
    match state.publisher_usecase.update_publisher(payload).await {
        Ok(publisher) => (StatusCode::OK, Json(publisher)).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

pub async fn delete_publisher(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state.publisher_usecase.delete_publisher(id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => map_error(e).into_response(),
    }
}
