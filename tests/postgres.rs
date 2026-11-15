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
        assert!(publisher_repo.update(publisher).await?);
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
        assert!(shop_repo.update(shop).await?);
        let shop = shop_repo.find_by_pub_id(shop_id).await?.unwrap();
        assert_eq!(shop.name(), "Renamed Shop");
        assert!(shop_repo.delete(shop, updated_by).await?);
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
        assert!(!repo.update(stale_edit.clone()).await?);
        assert!(!repo.delete(stale_edit, user_id).await?);
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

        Migrator::down(&db, Some(2)).await?;
        Migrator::up(&db, None).await?;
        assert!(repo.find_by_pub_id(pub_id).await?.is_some());
        Ok(())
    }).await
}

#[tokio::test]
#[ignore = "requires DATABASE_URL pointing to PostgreSQL"]
async fn master_repositories_reject_stale_updates_and_deletes() -> anyhow::Result<()> {
    with_database(|db| async move {
        Migrator::up(&db, None).await?;
        let publisher = create_publisher(&db).await?;
        let user_id = *publisher.created_by();
        let publisher_repo = infra::publisher::SqlRepository::new(db.clone());
        let mut winner = publisher.clone();
        winner.update(publisher::vo::PublisherName::new("Winner".into())?, user_id)?;
        assert!(publisher_repo.update(winner).await?);
        assert!(!publisher_repo.update(publisher.clone()).await?);
        assert!(!publisher_repo.delete(publisher.clone(), user_id).await?);
        let current = publisher_repo
            .find_by_pub_id(publisher.pub_id())
            .await?
            .unwrap();
        assert_eq!(current.id(), publisher.id());
        assert_eq!(current.name(), "Winner");
        assert_eq!(current.revision(), 1);
        assert_eq!(
            infra::publisher_history::Entity::find()
                .all(&db)
                .await?
                .len(),
            2
        );

        let shop_repo = infra::shop::SqlRepository::new(db.clone());
        let shop_id = Uuid::new_v4();
        shop_repo
            .create(shop::Shop::new(
                shop_id,
                shop::vo::ShopName::new("Shop".into())?,
                user_id,
            ))
            .await?;
        let shop = shop_repo.find_by_pub_id(shop_id).await?.unwrap();
        let mut winner = shop.clone();
        winner.update(shop::vo::ShopName::new("Winner".into())?, user_id)?;
        assert!(shop_repo.update(winner).await?);
        assert!(!shop_repo.update(shop.clone()).await?);
        assert!(!shop_repo.delete(shop.clone(), user_id).await?);
        let current = shop_repo.find_by_pub_id(shop_id).await?.unwrap();
        assert_eq!(current.id(), shop.id());
        assert_eq!(current.pub_id(), shop_id);
        assert_eq!(current.name(), "Winner");
        assert_eq!(current.revision(), 1);
        assert!(shop_repo.delete(current, user_id).await?);
        let current = publisher_repo
            .find_by_pub_id(publisher.pub_id())
            .await?
            .unwrap();
        assert!(publisher_repo.delete(current, user_id).await?);
        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore = "requires DATABASE_URL pointing to PostgreSQL"]
async fn simultaneous_book_edits_have_one_winner() -> anyhow::Result<()> {
    with_database(|db| async move {
        Migrator::up(&db, None).await?;
        let publisher = create_publisher(&db).await?;
        let user_id = *publisher.created_by();
        let repo = Arc::new(infra::book::SqlRepository::new(db.clone()));
        let pub_id = Uuid::new_v4();
        repo.create(book::Book::new(
            pub_id,
            book::vo::BookTitle::new("Original".into())?,
            book::vo::BookAuthor::new("Author".into())?,
            publisher.clone(),
            None,
            book::vo::BookFormat::Real,
            book::vo::BookPrice::new(100)?,
            user_id,
            user_id,
        ))
        .await?;
        let mut left = repo.find_by_pub_id(pub_id).await?.unwrap();
        let mut right = left.clone();
        left.update(
            book::vo::BookTitle::new("Left".into())?,
            book::vo::BookAuthor::new("Author".into())?,
            publisher.clone(),
            None,
            book::vo::BookFormat::Real,
            book::vo::BookPrice::new(200)?,
            user_id,
        )?;
        right.update(
            book::vo::BookTitle::new("Right".into())?,
            book::vo::BookAuthor::new("Author".into())?,
            publisher,
            None,
            book::vo::BookFormat::Real,
            book::vo::BookPrice::new(300)?,
            user_id,
        )?;
        let (left, right) = tokio::join!(repo.update(left), repo.update(right));
        let (left, right) = (left?, right?);
        assert_ne!(left, right);
        let winner = repo.find_by_pub_id(pub_id).await?.unwrap();
        assert_eq!(winner.revision(), 1);
        assert_eq!(winner.title(), if left { "Left" } else { "Right" });
        assert_eq!(winner.price(), if left { 200 } else { 300 });
        assert_eq!(infra::book_history::Entity::find().all(&db).await?.len(), 2);
        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore = "requires DATABASE_URL pointing to PostgreSQL"]
async fn database_rejects_identity_changes_and_controls_revisions() -> anyhow::Result<()> {
    with_database(|db| async move {
        Migrator::up(&db, None).await?;
        let publisher = create_publisher(&db).await?;
        let user_id = *publisher.created_by();
        let shop_id = Uuid::new_v4();
        infra::shop::SqlRepository::new(db.clone())
            .create(shop::Shop::new(
                shop_id,
                shop::vo::ShopName::new("Shop".into())?,
                user_id,
            ))
            .await?;
        let book_id = Uuid::new_v4();
        infra::book::SqlRepository::new(db.clone())
            .create(book::Book::new(
                book_id,
                book::vo::BookTitle::new("Book".into())?,
                book::vo::BookAuthor::new("Author".into())?,
                publisher.clone(),
                None,
                book::vo::BookFormat::Real,
                book::vo::BookPrice::new(100)?,
                user_id,
                user_id,
            ))
            .await?;

        for (table, pub_id) in [
            ("book", book_id),
            ("publisher", publisher.pub_id()),
            ("shop", shop_id),
        ] {
            let before = db
                .query_one_raw(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    format!("SELECT id FROM {table} WHERE pub_id = $1"),
                    [pub_id.into()],
                ))
                .await?
                .unwrap();
            let original_id: i32 = before.try_get("", "id")?;
            for assignment in [
                "id = id + 100000".to_string(),
                format!("pub_id = '{}'", Uuid::new_v4()),
            ] {
                let error = db
                    .execute_raw(Statement::from_sql_and_values(
                        DbBackend::Postgres,
                        format!("UPDATE {table} SET {assignment} WHERE pub_id = $1"),
                        [pub_id.into()],
                    ))
                    .await
                    .unwrap_err();
                assert!(
                    error.to_string().contains("id and pub_id are immutable"),
                    "{error}"
                );
            }

            let txn = db.begin_with_user(&user_id).await?;
            // Writing the same identifiers is safe; the caller cannot choose the new revision.
            txn.execute_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                format!(
                    "UPDATE {table} SET id = id, pub_id = pub_id, revision = 900 WHERE pub_id = $1"
                ),
                [pub_id.into()],
            ))
            .await?;
            txn.commit().await?;
            let after = db
                .query_one_raw(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    format!("SELECT id, pub_id, revision FROM {table} WHERE pub_id = $1"),
                    [pub_id.into()],
                ))
                .await?
                .unwrap();
            assert_eq!(after.try_get::<i32>("", "id")?, original_id);
            assert_eq!(after.try_get::<Uuid>("", "pub_id")?, pub_id);
            assert_eq!(after.try_get::<i64>("", "revision")?, 1);
        }
        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore = "requires DATABASE_URL pointing to PostgreSQL"]
async fn migration_upgrades_existing_masters_and_preserves_their_history() -> anyhow::Result<()> {
    with_database(|db| async move {
        Migrator::up(&db, Some(3)).await?;
        // Historical migrations currently use the live entities. Restore the
        // actual pre-upgrade schema before seeding existing data.
        for table in ["publisher", "publisher_history", "shop"] {
            db.execute_unprepared(&format!("ALTER TABLE {table} DROP COLUMN revision")).await?;
        }
        let user_id = Uuid::new_v4();
        let publisher_id = Uuid::new_v4();
        let shop_id = Uuid::new_v4();
        let txn = db.begin_with_user(&user_id).await?;
        for (table, pub_id) in [("publisher", publisher_id), ("shop", shop_id)] {
            txn.execute_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                format!("INSERT INTO {table} (pub_id, name, created_at, updated_at, created_by, updated_by) VALUES ($1, 'Existing', NOW(), NOW(), $2, $2)"),
                [pub_id.into(), user_id.into()],
            )).await?;
        }
        txn.commit().await?;
        Migrator::up(&db, None).await?;
        let publisher_repo = infra::publisher::SqlRepository::new(db.clone());
        let shop_repo = infra::shop::SqlRepository::new(db.clone());
        let mut publisher = publisher_repo.find_by_pub_id(publisher_id).await?.unwrap();
        let publisher_internal_id = publisher.id();
        assert_eq!(publisher.name(), "Existing");
        assert_eq!(publisher.revision(), 0);
        assert_eq!(shop_repo.find_by_pub_id(shop_id).await?.unwrap().revision(), 0);
        publisher.update(publisher::vo::PublisherName::new("Updated".into())?, user_id)?;
        assert!(publisher_repo.update(publisher).await?);
        let history = infra::publisher_history::Entity::find().all(&db).await?;
        assert_eq!(history.len(), 2);
        assert_eq!(history.iter().map(|row| row.revision).max(), Some(1));

        Migrator::down(&db, Some(1)).await?;
        Migrator::up(&db, None).await?;
        let mut publisher = publisher_repo.find_by_pub_id(publisher_id).await?.unwrap();
        assert_eq!(publisher.id(), publisher_internal_id);
        assert_eq!(publisher.name(), "Updated");
        publisher.update(publisher::vo::PublisherName::new("After re-upgrade".into())?, user_id)?;
        assert!(publisher_repo.update(publisher).await?);
        assert_eq!(infra::publisher_history::Entity::find().all(&db).await?.len(), 3);
        Ok(())
    }).await
}
