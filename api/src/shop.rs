use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use usecase::shop::{CreateDto, ResponseDto, UpdateDto};

use crate::AppState;

#[utoipa::path(
    post,
    path = "/shops",
    tag = "Shop",
    request_body = CreateDto,
    responses(
        (status = 201, description = "Shop created successfully", body = ResponseDto),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_shop(
    State(state): State<Arc<AppState>>,
    Json(dto): Json<CreateDto>,
) -> Result<impl IntoResponse, StatusCode> {
    state
        .shop_usecase
        .create(dto)
        .await
        .map(|dto| (StatusCode::CREATED, Json(dto)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    get,
    path = "/shops",
    tag = "Shop",
    responses(
        (status = 200, description = "List of all shops", body = [ResponseDto]),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_all_shops(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    state
        .shop_usecase
        .get_all()
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    get,
    path = "/shops/{pub_id}",
    tag = "Shop",
    params(
        ("pub_id" = Uuid, Path, description = "Shop ID")
    ),
    responses(
        (status = 200, description = "Shop found", body = ResponseDto),
        (status = 404, description = "Shop not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_shop(
    State(state): State<Arc<AppState>>,
    Path(pub_id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    state
        .shop_usecase
        .get(pub_id)
        .await
        .map(Json)
        .map_err(|e| match e {
            usecase::error::UseCaseError::NotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })
}

#[utoipa::path(
    put,
    path = "/shops/{pub_id}",
    tag = "Shop",
    request_body = UpdateDto,
    params(
        ("pub_id" = Uuid, Path, description = "Shop ID")
    ),
    responses(
        (status = 200, description = "Shop updated successfully", body = ResponseDto),
        (status = 404, description = "Shop not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_shop(
    State(state): State<Arc<AppState>>,
    Path(pub_id): Path<uuid::Uuid>,
    Json(dto): Json<UpdateDto>,
) -> Result<impl IntoResponse, StatusCode> {
    state
        .shop_usecase
        .update(pub_id, dto)
        .await
        .map(|dto| (StatusCode::OK, Json(dto)))
        .map_err(|e| match e {
            usecase::error::UseCaseError::NotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })
}

#[utoipa::path(
    delete,
    path = "/shops/{pub_id}",
    tag = "Shop",
    params(
        ("pub_id" = Uuid, Path, description = "Shop ID")
    ),
    responses(
        (status = 204, description = "Shop deleted successfully"),
        (status = 404, description = "Shop not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_shop(
    State(state): State<Arc<AppState>>,
    Path(pub_id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    state
        .shop_usecase
        .delete(pub_id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| match e {
            usecase::error::UseCaseError::NotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })
}
