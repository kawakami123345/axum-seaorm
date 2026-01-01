use crate::AppState;
use crate::error::AppError;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/books",
    tag = "Book",
    operation_id = "get_all_books",
    responses(
        (status = 200, description = "List all books", body = [usecase::book::ResponseDto])
    )
)]
pub async fn get_all(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.book_usecase.get_all().await {
        Ok(books) => (StatusCode::OK, Json(books)).into_response(),
        Err(e) => AppError(e).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/books/{pub_id}",
    tag = "Book",
    operation_id = "get_book",
    responses(
        (status = 200, description = "Get book by pub_id", body = usecase::book::ResponseDto),
        (status = 404, description = "Book not found")
    ),
    params(
        ("pub_id" = uuid::Uuid, Path, description = "Book pub_id")
    )
)]
pub async fn get(
    State(state): State<Arc<AppState>>,
    Path(pub_id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    match state.book_usecase.get(pub_id).await {
        Ok(book) => (StatusCode::OK, Json(book)).into_response(),
        Err(e) => AppError(e).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/books",
    tag = "Book",
    operation_id = "create_book",
    request_body = usecase::book::CreateDto,
    responses(
        (status = 201, description = "Book created successfully", body = usecase::book::ResponseDto)
    )
)]
pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<usecase::book::CreateDto>,
) -> impl IntoResponse {
    match state.book_usecase.create(payload).await {
        Ok(book) => (StatusCode::CREATED, Json(book)).into_response(),
        Err(e) => AppError(e).into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/books/{pub_id}",
    tag = "Book",
    operation_id = "update_book",
    request_body = usecase::book::UpdateDto,
    responses(
        (status = 200, description = "Book updated successfully", body = usecase::book::ResponseDto),
        (status = 404, description = "Book not found")
    ),
    params(
        ("pub_id" = uuid::Uuid, Path, description = "Book pub_id")
    )
)]
pub async fn update(
    State(state): State<Arc<AppState>>,
    Path(pub_id): Path<uuid::Uuid>,
    Json(payload): Json<usecase::book::UpdateDto>,
) -> impl IntoResponse {
    match state.book_usecase.update(pub_id, payload).await {
        Ok(book) => (StatusCode::OK, Json(book)).into_response(),
        Err(e) => AppError(e).into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/books/{pub_id}",
    tag = "Book",
    operation_id = "delete_book",
    responses(
        (status = 204, description = "Book deleted successfully"),
        (status = 404, description = "Book not found")
    ),
    params(
        ("pub_id" = uuid::Uuid, Path, description = "Book pub_id")
    )
)]
pub async fn delete(
    State(state): State<Arc<AppState>>,
    Path(pub_id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    match state.book_usecase.delete(pub_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => AppError(e).into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/books/{pub_id}/switch_status",
    tag = "Book",
    operation_id = "switch_book_status",
    responses(
        (status = 200, description = "Book status switched successfully", body = usecase::book::ResponseDto),
        (status = 404, description = "Book not found")
    ),
    params(
        ("pub_id" = uuid::Uuid, Path, description = "Book pub_id")
    )
)]
pub async fn switch_status(
    State(state): State<Arc<AppState>>,
    Path(pub_id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    match state.book_usecase.switch_status(pub_id).await {
        Ok(book) => (StatusCode::OK, Json(book)).into_response(),
        Err(e) => AppError(e).into_response(),
    }
}
