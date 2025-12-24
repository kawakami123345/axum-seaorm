use std::sync::Arc;

use axum::{Router, routing::get};
use usecase::{BookUseCase, PublisherUseCase};

pub mod book;
pub mod publisher;
pub struct AppState {
    pub book_usecase: BookUseCase,
    pub publisher_usecase: PublisherUseCase,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/books", get(book::get_books).post(book::create_book))
        .route(
            "/books/:id",
            get(book::get_book)
                .put(book::update_book)
                .delete(book::delete_book),
        )
        .route(
            "/publishers",
            get(publisher::get_publishers).post(publisher::create_publisher),
        )
        .route(
            "/publishers/:id",
            get(publisher::get_publisher)
                .put(publisher::update_publisher)
                .delete(publisher::delete_publisher),
        )
        .with_state(state)
}
