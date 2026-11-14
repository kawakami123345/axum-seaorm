use sea_orm::{DbBackend, Schema, sea_query::PostgresQueryBuilder};

#[test]
fn database_columns_keep_their_timestamp_with_time_zone_type() {
    let schema = Schema::new(DbBackend::Postgres);
    let tables = [
        schema.create_table_from_entity(crate::book::Entity),
        schema.create_table_from_entity(crate::publisher::Entity),
        schema.create_table_from_entity(crate::shop::Entity),
        schema.create_table_from_entity(crate::book_history::Entity),
        schema.create_table_from_entity(crate::publisher_history::Entity),
    ];
    for table in tables {
        let sql = table.to_string(PostgresQueryBuilder);
        for field in ["created_at", "updated_at"] {
            assert!(
                sql.contains(&format!("\"{field}\" timestamp with time zone NOT NULL")),
                "{sql}"
            );
        }
    }
}

#[cfg(feature = "test")]
#[tokio::test]
#[ignore = "requires DATABASE_URL pointing to a running PostgreSQL instance"]
async fn postgres_timestamp_round_trip_preserves_the_instant() -> anyhow::Result<()> {
    use anyhow::Context;
    use sea_orm::{ConnectionTrait, Database, Statement, TransactionTrait};
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    let db = Database::connect(std::env::var("DATABASE_URL")?).await?;
    let txn = db.begin().await?;
    txn.execute_unprepared("SET LOCAL TIME ZONE 'Asia/Tokyo'")
        .await?;
    for value in [
        "1969-12-31T23:59:59.123456Z",
        "2025-01-01T00:30:00.987654+09:00",
    ] {
        let timestamp = OffsetDateTime::parse(value, &Rfc3339)?;
        let row = txn
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT $1::timestamptz AS timestamp",
                [timestamp.into()],
            ))
            .await?
            .context("Expected a PostgreSQL timestamp row")?;
        let stored: OffsetDateTime = row.try_get("", "timestamp")?;
        assert_eq!(stored, timestamp);
    }
    txn.rollback().await?;
    db.close().await?;
    Ok(())
}
