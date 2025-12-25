use std::sync::Arc;

use axum::Router;
use usecase::{BookUseCase, PublisherUseCase};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

pub mod book;
pub mod publisher;

#[derive(OpenApi)]
pub struct ApiDoc;

pub struct AppState {
    pub book_usecase: BookUseCase,
    pub publisher_usecase: PublisherUseCase,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(book::get_books, book::create_book))
        .routes(routes!(
            book::get_book,
            book::update_book,
            book::delete_book
        ))
        .routes(routes!(
            publisher::get_publishers,
            publisher::create_publisher
        ))
        .routes(routes!(
            publisher::get_publisher,
            publisher::update_publisher,
            publisher::delete_publisher
        ))
        .split_for_parts();

    #[cfg(debug_assertions)]
    {
        let openapi_json = api
            .to_pretty_json()
            .expect("Failed to generate OpenAPI JSON");
        std::fs::write("openapi.json", openapi_json).expect("Failed to write openapi.json");
    }

    router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}
