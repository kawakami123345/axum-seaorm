use crate::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use usecase::error::map_error;

#[utoipa::path(
    get,
    path = "/books",
    responses(
        (status = 200, description = "List all books", body = [usecase::book::ResponseDto])
    )
)]
pub async fn get_all(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.book_usecase.get_all().await {
        Ok(books) => (StatusCode::OK, Json(books)).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/books/{id}",
    responses(
        (status = 200, description = "Get book by id", body = usecase::book::ResponseDto),
        (status = 404, description = "Book not found")
    ),
    params(
        ("id" = i32, Path, description = "Book id")
    )
)]
pub async fn get(State(state): State<Arc<AppState>>, Path(id): Path<i32>) -> impl IntoResponse {
    match state.book_usecase.get(id).await {
        Ok(book) => (StatusCode::OK, Json(book)).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/books",
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
        Err(e) => map_error(e).into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/books/{id}",
    request_body = usecase::book::UpdateDto,
    responses(
        (status = 200, description = "Book updated successfully", body = usecase::book::ResponseDto),
        (status = 404, description = "Book not found")
    ),
    params(
        ("id" = i32, Path, description = "Book id")
    )
)]
pub async fn update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(mut payload): Json<usecase::book::UpdateDto>,
) -> impl IntoResponse {
    payload.id = id;
    match state.book_usecase.update(payload).await {
        Ok(book) => (StatusCode::OK, Json(book)).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/books/{id}",
    responses(
        (status = 204, description = "Book deleted successfully"),
        (status = 404, description = "Book not found")
    ),
    params(
        ("id" = i32, Path, description = "Book id")
    )
)]
pub async fn delete(State(state): State<Arc<AppState>>, Path(id): Path<i32>) -> impl IntoResponse {
    match state.book_usecase.delete(id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => map_error(e).into_response(),
    }
}
