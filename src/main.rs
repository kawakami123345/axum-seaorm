use api::handlers::{AppState, create_router};
use infrastructure::repositories::{BookRepositoryImpl, PublisherRepositoryImpl};
use sea_orm::Database;
use std::sync::Arc;
use usecase::{BookUseCase, PublisherUseCase};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 0. Load .env
    dotenvy::dotenv().ok();

    // 1. Database Connection
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(db_url).await?;

    // 2. Setup Schema (Automated via infrastructure)
    #[cfg(debug_assertions)]
    infrastructure::init_db(&db).await?;

    // 3. Dependency Injection
    let book_repo = Arc::new(BookRepositoryImpl::new(db.clone()))
        as Arc<dyn domain::interfaces::BookRepository>;
    let publisher_repo = Arc::new(PublisherRepositoryImpl::new(db.clone()))
        as Arc<dyn domain::interfaces::PublisherRepository>;

    let book_usecase = BookUseCase::new(book_repo);
    let publisher_usecase = PublisherUseCase::new(publisher_repo);

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
