use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::schema::Schema;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let schema = Schema::new(manager.get_database_backend());

        // 1. Create history tables from Entity definitions
        manager
            .create_table(schema.create_table_from_entity(infra::book_history::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(infra::publisher_history::Entity))
            .await?;

        // 2. Create Trigger Functions and Triggers
        // Book - Trigger Function
        db.execute_unprepared(
            r#"
            CREATE OR REPLACE FUNCTION save_history_book() RETURNS TRIGGER AS $$
            BEGIN
                IF (TG_OP = 'DELETE') THEN
                    INSERT INTO book_history
                    SELECT
                        nextval(pg_get_serial_sequence('book_history', 'history_id')),
                        'DELETE',
                        NOW(),
                        (OLD).*;
                    RETURN OLD;
                ELSE
                    INSERT INTO book_history
                    SELECT
                        nextval(pg_get_serial_sequence('book_history', 'history_id')),
                        TG_OP,
                        NOW(),
                        (NEW).*;
                    RETURN NEW;
                END IF;
            END;
            $$ LANGUAGE plpgsql;
            "#,
        )
        .await?;

        // Book - Trigger
        db.execute_unprepared(
            r#"
            CREATE TRIGGER trigger_book_history
            AFTER INSERT OR UPDATE OR DELETE ON book
            FOR EACH ROW EXECUTE FUNCTION save_history_book();
            "#,
        )
        .await?;

        // Publisher - Trigger Function
        db.execute_unprepared(
            r#"
            CREATE OR REPLACE FUNCTION save_history_publisher() RETURNS TRIGGER AS $$
            BEGIN
                IF (TG_OP = 'DELETE') THEN
                    INSERT INTO publisher_history
                    SELECT
                        nextval(pg_get_serial_sequence('publisher_history', 'history_id')),
                        'DELETE',
                        NOW(),
                        (OLD).*;
                    RETURN OLD;
                ELSE
                    INSERT INTO publisher_history
                    SELECT
                        nextval(pg_get_serial_sequence('publisher_history', 'history_id')),
                        TG_OP,
                        NOW(),
                        (NEW).*;
                    RETURN NEW;
                END IF;
            END;
            $$ LANGUAGE plpgsql;
            "#,
        )
        .await?;

        // Publisher - Trigger
        db.execute_unprepared(
            r#"
            CREATE TRIGGER trigger_publisher_history
            AFTER INSERT OR UPDATE OR DELETE ON publisher
            FOR EACH ROW EXECUTE FUNCTION save_history_publisher();
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Drop Triggers and Functions
        db.execute_unprepared("DROP TRIGGER IF EXISTS trigger_publisher_history ON publisher")
            .await?;
        db.execute_unprepared("DROP FUNCTION IF EXISTS save_history_publisher")
            .await?;
        db.execute_unprepared("DROP TRIGGER IF EXISTS trigger_book_history ON book")
            .await?;
        db.execute_unprepared("DROP FUNCTION IF EXISTS save_history_book")
            .await?;

        // Drop History Tables
        manager
            .drop_table(
                Table::drop()
                    .table(infra::publisher_history::Entity)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(infra::book_history::Entity).to_owned())
            .await?;

        Ok(())
    }
}
