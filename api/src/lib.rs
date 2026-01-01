pub mod book;
pub mod publisher;

use axum::Router;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
pub struct ApiDoc;

pub struct AppState {
    pub book_usecase: usecase::book::Service,
    pub publisher_usecase: usecase::publisher::Service,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        // routes!はPath毎に分ける必要あり
        .routes(routes!(book::get_all, book::create))
        .routes(routes!(book::get, book::update, book::delete))
        .routes(routes!(book::switch_status))
        .routes(routes!(publisher::get_all, publisher::create))
        .routes(routes!(
            publisher::get,
            publisher::update,
            publisher::delete
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
