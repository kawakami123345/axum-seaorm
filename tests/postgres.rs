use std::{future::Future, sync::Arc, time::Duration};

use book::Repository as _;
use infra::BeginWithUser;
use migration::{Migrator, MigratorTrait};
use publisher::Repository as _;
use sea_orm::{
    ColumnTrait, ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend,
    EntityTrait, QueryFilter, Statement, TransactionTrait,
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
    with_connections(1, test).await
}

async fn with_connections<F, Fut>(connections: u32, test: F) -> anyhow::Result<()>
where
    F: FnOnce(DatabaseConnection) -> Fut,
    Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    let schema = format!("test_{}", Uuid::new_v4().simple());
    let mut options = ConnectOptions::new(std::env::var("DATABASE_URL")?);
    options
        .max_connections(connections)
        .min_connections(connections)
        .set_schema_search_path(schema.clone())
        .acquire_timeout(Duration::from_secs(3))
        .sqlx_logging(false);
    let db = Database::connect(options).await?;
    db.execute_unprepared(&format!("CREATE SCHEMA {schema}"))
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
async fn upgrades_original_books_and_publishers() -> anyhow::Result<()> {
    with_database(|db| async move {
        db.execute_unprepared(include_str!("fixtures/591ee80.sql"))
            .await?;
        db.execute_unprepared(
            "INSERT INTO publisher (pub_id, name) VALUES
                 (gen_random_uuid(), 'Original'), (gen_random_uuid(), 'Original');
             INSERT INTO book (pub_id, title, author, publisher_id, status, price) VALUES
                 (gen_random_uuid(), repeat('本', 128), 'Author', 2, 'Applied', 3000),
                 (gen_random_uuid(), 'Pending', 'Author', 1, 'Unapplied', 1000)",
        )
        .await?;
        Migrator::up(&db, None).await?;
        let books = infra::book::SqlRepository::new(db.clone());
        let owner = Uuid::new_v4();
        for id in [owner, Uuid::nil()] {
            let visible = books.find_all(book::ListFilter::OwnedBy(id)).await?;
            assert!(visible.is_empty());
        }
        let all = books.find_all(book::ListFilter::All).await?;
        assert_eq!(all.len(), 2);
        assert!(!all[0].applied_without_date());
        let mut applied = all[1].clone();
        assert_eq!(applied.title(), "本".repeat(128));
        assert_eq!(applied.publisher().id(), 2);
        assert!(applied.applied_without_date());
        assert!(applied.applied_at().is_none());
        assert!(books.annual_totals(book::ListFilter::All).await?.is_empty());
        applied.change_applied_at(Some(OffsetDateTime::now_utc()), owner)?;
        assert!(books.update(applied).await?);
        let history = infra::book_history::Entity::find().one(&db).await?.unwrap();
        assert_eq!(history.title, "本".repeat(128));
        assert_eq!(history.operation_by, owner);
        assert!(!history.applied_without_date);
        assert_eq!(history.revision, 1);

        let masters = infra::publisher::SqlRepository::new(db.clone());
        let mut originals = masters.find_all().await?;
        assert_eq!(originals.len(), 2);
        let mut first = originals.pop().unwrap();
        first.update(publisher::vo::PublisherName::new("Renamed".into())?, owner)?;
        assert!(masters.update(first).await?);
        let duplicate = publisher::Publisher::new(
            Uuid::new_v4(),
            publisher::vo::PublisherName::new("Original".into())?,
            owner,
        );
        let error = masters.create(duplicate).await.unwrap_err();
        assert!(error.is::<usecase::error::DuplicateName>());
        let renamed = infra::publisher_history::Entity::find()
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(renamed.name, "Renamed");
        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore = "requires DATABASE_URL pointing to PostgreSQL"]
async fn duplicate_master_names_are_client_errors_for_create_and_rename() -> anyhow::Result<()> {
    use axum::{http::StatusCode, response::IntoResponse};
    use usecase::{error::UseCaseError, publisher as publishers, shop as shops};

    with_database(|db| async move {
        Migrator::up(&db, None).await?;
        let ctx = UserContext::new(Uuid::new_v4(), vec![]);
        let publisher_repo = infra::publisher::SqlRepository::new(db.clone());
        let publishers = publishers::Service::new(Arc::new(publisher_repo));
        let shops = shops::Service::new(Arc::new(infra::shop::SqlRepository::new(db)));
        for name in ["First", "Second"] {
            publishers
                .create(&ctx, publishers::CreateDto { name: name.into() })
                .await?;
            shops.create(&ctx, shops::CreateDto { name: name.into() })
                .await?;
        }
        let publisher = publishers.get_all().await?.remove(0);
        let shop = shops.get_all().await?.remove(0);
        let publisher_rename = publishers::UpdateDto {
            name: "First".into(),
            revision: publisher.revision,
        };
        let shop_rename = shops::UpdateDto {
            name: "First".into(),
            revision: shop.revision,
        };
        let failures = [
            publishers.create(&ctx, publishers::CreateDto { name: "First".into() }).await,
            publishers.update(&ctx, publisher.pub_id, publisher_rename).await,
            shops.create(&ctx, shops::CreateDto { name: "First".into() }).await,
            shops.update(&ctx, shop.pub_id, shop_rename).await,
        ];
        for failure in failures {
            let error = failure.unwrap_err();
            assert!(matches!(&error, UseCaseError::DomainRuleViolation(msg) if msg.contains("すでに登録")));
            let response = api::error::AppError(error).into_response();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }
        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore = "requires DATABASE_URL pointing to PostgreSQL"]
async fn preserves_pre_oidc_history_and_actor_names() -> anyhow::Result<()> {
    with_database(|db| async move {
        db.execute_unprepared(include_str!("fixtures/591ee80.sql"))
            .await?;
        db.execute_unprepared(include_str!("fixtures/pre_oidc_history.sql"))
            .await?;
        Migrator::up(&db, None).await?;

        let original = infra::book_history::Entity::find().one(&db).await?.unwrap();
        assert_eq!(original.history_id, 1);
        assert!(original.applied_without_date);
        assert_eq!(original.created_by, Uuid::from_u128(1));
        assert!(original.user_id.is_nil());
        let row = db
            .query_one_raw(Statement::from_string(
                DbBackend::Postgres,
                "SELECT legacy_updated_by FROM book_history WHERE history_id = 1",
            ))
            .await?
            .unwrap();
        assert_eq!(
            row.try_get::<String>("", "legacy_updated_by")?,
            "old operator"
        );

        let books = infra::book::SqlRepository::new(db.clone());
        let mut book = books.find_by_pub_id(original.pub_id).await?.unwrap();
        book.change_applied_at(None, Uuid::new_v4())?;
        assert!(books.update(book).await?);
        let updated = infra::book_history::Entity::find_by_id(2)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated.pub_id, original.pub_id);
        assert!(!updated.applied_without_date);
        let book = books.find_by_pub_id(original.pub_id).await?.unwrap();
        assert!(books.delete(book, Uuid::new_v4()).await?);
        let deleted = infra::book_history::Entity::find_by_id(3)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(deleted.operation_type, "DELETE");
        assert_eq!(deleted.pub_id, original.pub_id);
        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore = "requires DATABASE_URL pointing to PostgreSQL"]
async fn upgrades_a_database_already_using_revision_guards() -> anyhow::Result<()> {
    with_database(|db| async move {
        Migrator::up(&db, None).await?;
        let original = create_publisher(&db).await?;
        // Physical layout and migration ledger of the version before this fix.
        db.execute_unprepared(
            "ALTER TABLE book DROP COLUMN applied_without_date;
            ALTER TABLE book_history DROP COLUMN applied_without_date;
            DELETE FROM seaql_migrations WHERE version = 'm20261114_000004_upgrade_legacy_tables'",
        )
        .await?;
        Migrator::up(&db, None).await?;
        let repo = infra::publisher::SqlRepository::new(db.clone());
        let mut publisher = repo.find_by_pub_id(original.pub_id()).await?.unwrap();
        assert_eq!(publisher.revision(), original.revision());
        publisher.update(
            publisher::vo::PublisherName::new("Updated".into())?,
            Uuid::new_v4(),
        )?;
        assert!(repo.update(publisher).await?);
        let history = infra::publisher_history::Entity::find().all(&db).await?;
        assert_eq!(history.len(), 2);
        assert_eq!(history.iter().map(|row| row.revision).max(), Some(1));
        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore = "requires DATABASE_URL pointing to PostgreSQL"]
async fn annual_totals_use_utc_and_filter_before_aggregating() -> anyhow::Result<()> {
    with_database(|db| async move {
        Migrator::up(&db, None).await?;
        db.execute_unprepared("SET TIME ZONE 'Asia/Tokyo'").await?;
        let publisher = create_publisher(&db).await?;
        let owner = Uuid::new_v4();
        let other = Uuid::new_v4();
        let txn = db.begin_with_user(&owner).await?;
        for (id, date) in [
            (owner, Some("2025-01-01T00:15:00+09:00")),
            (owner, Some("2024-07-01T00:00:00Z")),
            (other, Some("2025-01-01T12:00:00Z")),
            (other, None),
        ] {
            txn.execute_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "INSERT INTO book (pub_id, title, author, publisher_id, format, price, applied_at,
                                   created_at, updated_at, created_by, updated_by, user_id)
                 VALUES ($1, 'Book', 'A', $2, 'Real', 2147483647, $3::text::timestamptz,
                         now(), now(), $4, $4, $4)",
                [
                    Uuid::new_v4().into(),
                    publisher.id().into(),
                    date.into(),
                    id.into(),
                ],
            ))
            .await?;
        }
        txn.commit().await?;
        let books = infra::book::SqlRepository::new(db);
        let totals = books
            .annual_totals(book::ListFilter::OwnedBy(owner))
            .await?;
        let first_year = book::AnnualTotals {
            year: 2024,
            count: 2,
            total_amount: 4_294_967_294,
        };
        let second_year = book::AnnualTotals {
            year: 2025,
            count: 1,
            total_amount: 2_147_483_647,
        };
        assert_eq!(totals.as_slice(), std::slice::from_ref(&first_year));
        let all = books.annual_totals(book::ListFilter::All).await?;
        assert_eq!(all, [first_year, second_year]);
        Ok(())
    })
    .await
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

        Migrator::down(&db, Some(3)).await?;
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
    with_connections(3, |db| async move {
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
        // Hold the row until both writers have reached UPDATE on separate DB
        // connections. The schema is unique to this test.
        let gate = db.begin().await?;
        gate.query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT id FROM book WHERE pub_id = $1 FOR UPDATE",
            [pub_id.into()],
        ))
        .await?;
        let release = async move {
            let ready = tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    let row = gate
                        .query_one_raw(Statement::from_string(
                            DbBackend::Postgres,
                            "SELECT count(*)::integer AS writers FROM pg_locks
                         WHERE relation = 'book'::regclass AND mode = 'RowExclusiveLock'
                           AND granted AND pid <> pg_backend_pid()",
                        ))
                        .await?
                        .unwrap();
                    if row.try_get::<i32>("", "writers")? == 2 {
                        return Ok::<_, sea_orm::DbErr>(());
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            })
            .await;
            gate.rollback().await?;
            ready??;
            Ok::<_, anyhow::Error>(())
        };
        let (left, right, released) = tokio::join!(repo.update(left), repo.update(right), release);
        released?;
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
