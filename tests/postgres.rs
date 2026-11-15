use std::{future::Future, sync::Arc, time::Duration};

use book::Repository as _;
use infra::BeginWithUser;
use migration::{Migrator, MigratorTrait};
use publisher::Repository as _;
use sea_orm::{
    ColumnTrait, ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend,
    EntityTrait, QueryFilter, Statement,
};
use shop::Repository as _;
use time::OffsetDateTime;
use usecase::{
    UserContext,
    session::{Repository as _, Session},
};
use uuid::Uuid;

async fn with_database<F, Fut>(test: F) -> anyhow::Result<()>
where
    F: FnOnce(DatabaseConnection) -> Fut,
    Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    let mut options = ConnectOptions::new(std::env::var("DATABASE_URL")?);
    options
        .max_connections(1)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(3))
        .sqlx_logging(false);
    let db = Database::connect(options).await?;
    let schema = format!("test_{}", Uuid::new_v4().simple());
    db.execute_unprepared(&format!("CREATE SCHEMA {schema}"))
        .await?;
    // Exclude public so migration metadata and application queries cannot
    // resolve to an existing development table when a test table is absent.
    db.execute_unprepared(&format!("SET search_path TO {schema}"))
        .await?;

    let result = tokio::spawn(test(db.clone())).await;
    let cleanup = db
        .execute_unprepared(&format!("DROP SCHEMA {schema} CASCADE"))
        .await;
    db.close().await?;
    cleanup?;
    result??;
    Ok(())
}

async fn create_publisher(db: &DatabaseConnection) -> anyhow::Result<publisher::Publisher> {
    let repo = infra::publisher::SqlRepository::new(db.clone());
    let item = publisher::Publisher::new(
        Uuid::new_v4(),
        publisher::vo::PublisherName::new("Test Publisher".into())?,
        Uuid::new_v4(),
    );
    let pub_id = item.pub_id();
    repo.create(item).await?;
    Ok(repo.find_by_pub_id(pub_id).await?.unwrap())
}

#[tokio::test]
#[ignore = "requires DATABASE_URL pointing to PostgreSQL"]
async fn repositories_update_and_delete_with_one_connection() -> anyhow::Result<()> {
    with_database(|db| async move {
        Migrator::up(&db, None).await?;
        let updated_by = Uuid::new_v4();
        let mut publisher = create_publisher(&db).await?;
        let publisher_repo = infra::publisher::SqlRepository::new(db.clone());
        publisher.update(
            publisher::vo::PublisherName::new("Renamed Publisher".into())?,
            updated_by,
        )?;
        let publisher_id = publisher.pub_id();
        publisher_repo.update(publisher).await?;
        let stored = publisher_repo.find_by_pub_id(publisher_id).await?.unwrap();
        assert_eq!(stored.name(), "Renamed Publisher");
        assert_eq!(*stored.updated_by(), updated_by);
        let history = infra::publisher_history::Entity::find()
            .filter(infra::publisher_history::Column::OperationType.eq("UPDATE"))
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(history.operation_by, updated_by);
        assert_eq!(history.name, "Renamed Publisher");

        let shop_repo = infra::shop::SqlRepository::new(db.clone());
        let shop = shop::Shop::new(
            Uuid::new_v4(),
            shop::vo::ShopName::new("Test Shop".into())?,
            updated_by,
        );
        let shop_id = shop.pub_id();
        shop_repo.create(shop).await?;
        let mut shop = shop_repo.find_by_pub_id(shop_id).await?.unwrap();
        shop.update(shop::vo::ShopName::new("Renamed Shop".into())?, updated_by)?;
        shop_repo.update(shop).await?;
        let shop = shop_repo.find_by_pub_id(shop_id).await?.unwrap();
        assert_eq!(shop.name(), "Renamed Shop");
        shop_repo.delete(shop, updated_by).await?;
        assert!(shop_repo.find_by_pub_id(shop_id).await?.is_none());
        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore = "requires DATABASE_URL pointing to PostgreSQL"]
async fn stale_book_writes_preserve_the_winning_edit_and_history() -> anyhow::Result<()> {
    with_database(|db| async move {
        Migrator::up(&db, None).await?;
        let publisher = create_publisher(&db).await?;
        let user_id = *publisher.created_by();
        let repo = infra::book::SqlRepository::new(db.clone());
        let book = book::Book::new(
            Uuid::new_v4(),
            book::vo::BookTitle::new("Original Title".into())?,
            book::vo::BookAuthor::new("Author".into())?,
            publisher.clone(),
            None,
            book::vo::BookFormat::Real,
            book::vo::BookPrice::new(1000)?,
            user_id,
            user_id,
        );
        let pub_id = book.pub_id();
        repo.create(book).await?;
        let mut stale_edit = repo.find_by_pub_id(pub_id).await?.unwrap();
        let mut applying = stale_edit.clone();
        let applied_at = OffsetDateTime::now_utc().replace_nanosecond(0)?;
        applying.change_applied_at(Some(applied_at), user_id)?;
        assert!(repo.update(applying).await?);
        stale_edit.update(
            book::vo::BookTitle::new("Stale Title".into())?,
            book::vo::BookAuthor::new("Author".into())?,
            publisher.clone(),
            None,
            book::vo::BookFormat::Real,
            book::vo::BookPrice::new(2000)?,
            user_id,
        )?;
        assert!(!repo.update(stale_edit).await?);
        let mut stored = repo.find_by_pub_id(pub_id).await?.unwrap();
        assert_eq!(stored.applied_at(), Some(applied_at));
        assert_eq!(stored.title(), "Original Title");
        assert_eq!(stored.price(), 1000);
        assert_eq!(stored.revision(), 1);

        stored.change_applied_at(None, user_id)?;
        assert!(repo.update(stored).await?);
        let mut editing = repo.find_by_pub_id(pub_id).await?.unwrap();
        let mut stale_application = editing.clone();
        editing.update(
            book::vo::BookTitle::new("Winning Title".into())?,
            book::vo::BookAuthor::new("Author".into())?,
            publisher,
            None,
            book::vo::BookFormat::EBook,
            book::vo::BookPrice::new(3000)?,
            user_id,
        )?;
        assert!(repo.update(editing).await?);
        stale_application.change_applied_at(Some(applied_at), user_id)?;
        assert!(!repo.update(stale_application).await?);
        let stored = repo.find_by_pub_id(pub_id).await?.unwrap();
        assert_eq!(stored.title(), "Winning Title");
        assert_eq!(stored.price(), 3000);
        assert_eq!(stored.applied_at(), None);
        assert_eq!(stored.revision(), 3);
        let history = infra::book_history::Entity::find().all(&db).await?;
        assert_eq!(history.len(), 4);
        assert!(history.iter().all(|entry| entry.operation_by == user_id));
        assert_eq!(history.iter().map(|entry| entry.revision).max(), Some(3));
        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore = "requires DATABASE_URL pointing to PostgreSQL"]
async fn sessions_persist_expire_revoke_and_clean_up() -> anyhow::Result<()> {
    with_database(|db| async move {
        Migrator::up(&db, None).await?;
        let repo = Arc::new(infra::session::SqlRepository::new(db.clone()));
        let service = usecase::session::Service::new(repo.clone());
        let user_id = Uuid::new_v4();
        let user = UserContext::new(user_id, vec!["admin".into()]);
        let before = OffsetDateTime::now_utc();
        let session = service.create(user.clone()).await?;
        assert!(session.expires_at >= before + usecase::session::SESSION_LIFETIME);
        let another_service = usecase::session::Service::new(Arc::new(
            infra::session::SqlRepository::new(db.clone()),
        ));
        let authenticated = another_service.authenticate(session.id).await?.unwrap();
        assert_eq!(*authenticated.user_id(), user_id);
        assert!(authenticated.is_admin());
        service.revoke(session.id).await?;
        assert!(another_service.authenticate(session.id).await?.is_none());

        let expired = Session {
            id: Uuid::new_v4(),
            user: user.clone(),
            expires_at: before - time::Duration::seconds(1),
        };
        let expired_id = expired.id;
        repo.create(expired).await?;
        assert!(service.authenticate(expired_id).await?.is_none());
        service.create(user).await?;
        assert!(repo.find(expired_id).await?.is_none());
        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore = "requires DATABASE_URL pointing to PostgreSQL"]
async fn migrations_upgrade_existing_books_and_preserve_history() -> anyhow::Result<()> {
    with_database(|db| async move {
        Migrator::up(&db, Some(2)).await?;
        // Emulate the deployed schema before revisions, including existing rows.
        db.execute_unprepared("ALTER TABLE book DROP COLUMN revision").await?;
        db.execute_unprepared("ALTER TABLE book_history DROP COLUMN revision").await?;
        let publisher = create_publisher(&db).await?;
        let user_id = *publisher.created_by();
        let pub_id = Uuid::new_v4();
        let txn = db.begin_with_user(&user_id).await?;
        txn.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,
            "INSERT INTO book (pub_id, title, author, publisher_id, format, price, created_at, updated_at, created_by, updated_by, user_id) VALUES ($1, 'Existing Title', 'Author', $2, 'Real', 1000, NOW(), NOW(), $3, $3, $3)",
            [pub_id.into(), publisher.id().into(), user_id.into()],
        )).await?;
        txn.commit().await?;

        Migrator::up(&db, None).await?;
        let repo = infra::book::SqlRepository::new(db.clone());
        let mut book = repo.find_by_pub_id(pub_id).await?.unwrap();
        assert_eq!(book.revision(), 0);
        assert_eq!(book.title(), "Existing Title");
        book.change_applied_at(Some(OffsetDateTime::now_utc()), user_id)?;
        assert!(repo.update(book).await?);
        let history = infra::book_history::Entity::find().all(&db).await?;
        assert_eq!(history.len(), 2);
        assert_eq!(history.iter().map(|entry| entry.revision).max(), Some(1));

        Migrator::down(&db, Some(1)).await?;
        Migrator::up(&db, None).await?;
        assert!(repo.find_by_pub_id(pub_id).await?.is_some());
        Ok(())
    }).await
}
