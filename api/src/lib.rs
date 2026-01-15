pub mod auth;
pub mod book;
pub mod dashboard;
pub mod error;
pub mod publisher;
pub mod shop;

use axum::Router;
use std::sync::Arc;
use tower_cookies::CookieManagerLayer;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(info(title = "Rust Web App", version = "0.1.0"))]
pub struct ApiDoc;

pub struct AppState {
    pub book_usecase: usecase::book::Service,
    pub publisher_usecase: usecase::publisher::Service,
    pub shop_usecase: usecase::shop::Service,
    pub dashboard_usecase: usecase::dashboard::Service,
    pub oidc_client: crate::auth::AppClient,
    pub cookie_key: tower_cookies::Key,
    pub http_client: reqwest::Client,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let (api_router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        // routes!はPath毎に分ける必要あり
        .routes(routes!(book::get_all, book::create))
        .routes(routes!(book::get, book::update, book::delete))
        .routes(routes!(book::change_applied_at))
        .routes(routes!(book::get_year_applied_books))
        .routes(routes!(publisher::get_all, publisher::create))
        .routes(routes!(
            publisher::get,
            publisher::update,
            publisher::delete
        ))
        .routes(routes!(shop::get_all_shops, shop::create_shop))
        .routes(routes!(
            shop::get_shop,
            shop::update_shop,
            shop::delete_shop
        ))
        .routes(routes!(dashboard::get_annual_summary))
        .split_for_parts();

    #[cfg(debug_assertions)]
    {
        let openapi_json = api
            .to_pretty_json()
            .expect("Failed to generate OpenAPI JSON");
        std::fs::write("openapi.json", openapi_json).expect("Failed to write openapi.json");
    }

    // 認証が必要なルート
    let protected_routes = api_router.layer(axum::middleware::from_fn_with_state(
        state.clone(),
        auth::require_auth,
    ));

    // 認証不要のルート (login, callback, logout, swagger-ui)
    let public_routes = Router::new()
        .merge(auth::auth_router())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api));

    protected_routes
        .merge(public_routes)
        .layer(axum::middleware::from_fn(auth::csrf_layer))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(CookieManagerLayer::new())
        .with_state(state)
}
