use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use usecase::shop::{CreateDto, ResponseDto, UpdateDto};

use crate::AppState;
use crate::error::AppError;

#[utoipa::path(
    post,
    path = "/shops",
    tag = "Shop",
    request_body = CreateDto,
    responses(
        (status = 204, description = "Shop created successfully"),
        (status = 400, description = "Invalid or duplicate shop name"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_shop(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<usecase::UserContext>,
    Json(dto): Json<CreateDto>,
) -> impl IntoResponse {
    match state.shop_usecase.create(&ctx, dto).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => AppError(e).into_response(),
    }
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
pub async fn get_all_shops(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.shop_usecase.get_all().await {
        Ok(shops) => (StatusCode::OK, Json(shops)).into_response(),
        Err(e) => AppError(e).into_response(),
    }
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
) -> impl IntoResponse {
    match state.shop_usecase.get(pub_id).await {
        Ok(shop) => (StatusCode::OK, Json(shop)).into_response(),
        Err(e) => AppError(e).into_response(),
    }
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
        (status = 204, description = "Shop updated successfully"),
        (status = 400, description = "Invalid or duplicate shop name"),
        (status = 409, description = "Shop changed since it was loaded"),
        (status = 404, description = "Shop not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_shop(
    State(state): State<Arc<AppState>>,
    Path(pub_id): Path<uuid::Uuid>,
    Extension(ctx): Extension<usecase::UserContext>,
    Json(dto): Json<UpdateDto>,
) -> impl IntoResponse {
    match state.shop_usecase.update(&ctx, pub_id, dto).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => AppError(e).into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/shops/{pub_id}",
    tag = "Shop",
    params(
        ("pub_id" = Uuid, Path, description = "Shop ID"),
        ("revision" = i64, Query, minimum = 0, description = "Revision read by the client")
    ),
    responses(
        (status = 204, description = "Shop deleted successfully"),
        (status = 409, description = "Shop changed since it was loaded"),
        (status = 404, description = "Shop not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_shop(
    State(state): State<Arc<AppState>>,
    Path(pub_id): Path<uuid::Uuid>,
    Extension(ctx): Extension<usecase::UserContext>,
    Query(query): Query<crate::RevisionQuery>,
) -> impl IntoResponse {
    match state
        .shop_usecase
        .delete(&ctx, pub_id, query.revision)
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => AppError(e).into_response(),
    }
}
