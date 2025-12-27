use crate::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/publishers",
    tag = "Publisher",
    operation_id = "get_all_publishers",
    responses(
        (status = 200, description = "List all publishers", body = [usecase::publisher::ResponseDto])
    )
)]
pub async fn get_all(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.publisher_usecase.get_all().await {
        Ok(publishers) => (StatusCode::OK, Json(publishers)).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/publishers/{id}",
    tag = "Publisher",
    operation_id = "get_publisher",
    responses(
        (status = 200, description = "Get publisher by id", body = usecase::publisher::ResponseDto),
        (status = 404, description = "Publisher not found")
    ),
    params(
        ("id" = i32, Path, description = "Publisher id")
    )
)]
pub async fn get(State(state): State<Arc<AppState>>, Path(id): Path<i32>) -> impl IntoResponse {
    match state.publisher_usecase.get(id).await {
        Ok(publisher) => (StatusCode::OK, Json(publisher)).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/publishers",
    tag = "Publisher",
    operation_id = "create_publisher",
    request_body =  usecase::publisher::CreateDto,
    responses(
        (status = 201, description = "Publisher created successfully", body = usecase::publisher::ResponseDto)
    )
)]
pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<usecase::publisher::CreateDto>,
) -> impl IntoResponse {
    match state.publisher_usecase.create(payload).await {
        Ok(publisher) => (StatusCode::CREATED, Json(publisher)).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/publishers/{id}",
    tag = "Publisher",
    operation_id = "update_publisher",
    request_body = usecase::publisher::UpdateDto,
    responses(
        (status = 200, description = "Publisher updated successfully", body = usecase::publisher::ResponseDto),
        (status = 404, description = "Publisher not found")
    ),
    params(
        ("id" = i32, Path, description = "Publisher id")
    )
)]
pub async fn update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(mut payload): Json<usecase::publisher::UpdateDto>,
) -> impl IntoResponse {
    payload.id = id;
    match state.publisher_usecase.update(payload).await {
        Ok(publisher) => (StatusCode::OK, Json(publisher)).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/publishers/{id}",
    tag = "Publisher",
    operation_id = "delete_publisher",
    responses(
        (status = 204, description = "Publisher deleted successfully"),
        (status = 404, description = "Publisher not found")
    ),
    params(
        ("id" = i32, Path, description = "Publisher id")
    )
)]
pub async fn delete(State(state): State<Arc<AppState>>, Path(id): Path<i32>) -> impl IntoResponse {
    match state.publisher_usecase.delete(id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => e.into_response(),
    }
}
