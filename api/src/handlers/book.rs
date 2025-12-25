use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use usecase::dtos::book::{BookCreateDto, BookResponseDto, BookUpdateDto};
use usecase::map_error;

use crate::handlers::AppState;

#[utoipa::path(
    get,
    path = "/books",
    responses(
        (status = 200, description = "List all books", body = [BookResponseDto])
    )
)]
pub async fn get_books(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.book_usecase.get_all_books().await {
        Ok(books) => (StatusCode::OK, Json(books)).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/books/{id}",
    responses(
        (status = 200, description = "Get book by id", body = BookResponseDto),
        (status = 404, description = "Book not found")
    ),
    params(
        ("id" = i32, Path, description = "Book id")
    )
)]
pub async fn get_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state.book_usecase.get_book(id).await {
        Ok(book) => (StatusCode::OK, Json(book)).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/books",
    request_body = BookCreateDto,
    responses(
        (status = 201, description = "Book created successfully", body = BookResponseDto)
    )
)]
pub async fn create_book(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<BookCreateDto>,
) -> impl IntoResponse {
    match state.book_usecase.create_book(payload).await {
        Ok(book) => (StatusCode::CREATED, Json(book)).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/books/{id}",
    request_body = BookUpdateDto,
    responses(
        (status = 200, description = "Book updated successfully", body = BookResponseDto),
        (status = 404, description = "Book not found")
    ),
    params(
        ("id" = i32, Path, description = "Book id")
    )
)]
pub async fn update_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(mut payload): Json<BookUpdateDto>,
) -> impl IntoResponse {
    payload.id = id;
    match state.book_usecase.update_book(payload).await {
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
pub async fn delete_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state.book_usecase.delete_book(id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => map_error(e).into_response(),
    }
}
