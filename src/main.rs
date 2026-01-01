use api::{AppState, create_router};
use sea_orm::Database;
use std::sync::Arc;
mod test;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 0. Load .env
    dotenvy::dotenv().ok();

    // 1. Database Connection
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(db_url).await?;

    // 2. Setup Schema (Automated via infra)
    #[cfg(debug_assertions)]
    infra::init_db(&db).await?;

    // 3. Dependency Injection
    let book_repo =
        Arc::new(infra::book::SqlRepository::new(db.clone())) as Arc<dyn book::Repository>;
    let publisher_repo = Arc::new(infra::publisher::SqlRepository::new(db.clone()))
        as Arc<dyn publisher::Repository>;

    let book_usecase = usecase::book::Service::new(book_repo);
    let publisher_usecase = usecase::publisher::Service::new(publisher_repo);

    let state = Arc::new(AppState {
        book_usecase,
        publisher_usecase,
    });

    // 4. Start Server
    let router = create_router(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://localhost:3000");

    axum::serve(listener, router).await?;

    Ok(())
}
