use api::{AppState, create_router};
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use std::sync::Arc;
mod test;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 0. Load .env
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    usecase::cedar::init()?;

    // 1. Database Connection
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(db_url).await?;

    // 2. Run Migrations
    Migrator::up(&db, None).await?;

    // 3. Dependency Injection
    let book_repo =
        Arc::new(infra::book::SqlRepository::new(db.clone())) as Arc<dyn book::Repository>;
    let publisher_repo = Arc::new(infra::publisher::SqlRepository::new(db.clone()))
        as Arc<dyn publisher::Repository>;
    let shop_repo =
        Arc::new(infra::shop::SqlRepository::new(db.clone())) as Arc<dyn shop::Repository>;

    let book_usecase =
        usecase::book::Service::new(book_repo.clone(), publisher_repo.clone(), shop_repo.clone());
    let publisher_usecase = usecase::publisher::Service::new(publisher_repo);
    let shop_usecase = usecase::shop::Service::new(shop_repo);
    let dashboard_usecase = usecase::dashboard::Service::new(book_repo.clone());

    // HTTP Client
    let http_client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()?;

    // OIDC Client
    let oidc_client = api::auth::create_oidc_client(&http_client).await?;

    let cookie_key = std::env::var("COOKIE_KEY")
        .map(|s| tower_cookies::Key::derive_from(s.as_bytes()))
        .unwrap_or_else(|_| tower_cookies::Key::generate());

    let state = Arc::new(AppState {
        book_usecase,
        publisher_usecase,
        shop_usecase,
        dashboard_usecase,
        oidc_client,
        cookie_key,
        http_client,
    });

    // 4. Start Server
    let router = create_router(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| format!("http://localhost:{}", port));
    println!("Server running on {}", app_url);

    axum::serve(listener, router).await?;

    Ok(())
}
