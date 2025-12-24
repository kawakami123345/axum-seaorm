use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use usecase::dtos::book::{BookCreateDto, BookUpdateDto};
use usecase::map_error;

use crate::handlers::AppState;

pub async fn get_books(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.book_usecase.get_all_books().await {
        Ok(books) => (StatusCode::OK, Json(books)).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

pub async fn get_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state.book_usecase.get_book(id).await {
        Ok(book) => (StatusCode::OK, Json(book)).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

pub async fn create_book(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<BookCreateDto>,
) -> impl IntoResponse {
    match state.book_usecase.create_book(payload).await {
        Ok(book) => (StatusCode::CREATED, Json(book)).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

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

pub async fn delete_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state.book_usecase.delete_book(id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => map_error(e).into_response(),
    }
}
