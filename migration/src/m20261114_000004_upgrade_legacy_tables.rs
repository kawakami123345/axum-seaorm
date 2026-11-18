use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(include_str!("upgrade_legacy_tables.sql"))
            .await?;
        // The original audit functions copied columns by position. An upgraded
        // table has a different physical column order from a new installation.
        for table in ["book", "publisher"] {
            db.execute_unprepared(&format!(r#"
                CREATE OR REPLACE FUNCTION save_history_{table}() RETURNS trigger AS $$
                DECLARE
                    snapshot jsonb;
                BEGIN
                    snapshot := CASE WHEN TG_OP = 'DELETE' THEN to_jsonb(OLD) ELSE to_jsonb(NEW) END;
                    INSERT INTO {table}_history
                    SELECT * FROM jsonb_populate_record(NULL::{table}_history, snapshot || jsonb_build_object(
                        'history_id', nextval(pg_get_serial_sequence('{table}_history', 'history_id')),
                        'operation_type', TG_OP, 'operation_at', now(),
                        'operation_by', current_setting('app.current_user_id', true)::uuid
                    ));
                    RETURN NULL;
                END;
                $$ LANGUAGE plpgsql;
                ALTER TABLE {table} ENABLE TRIGGER trigger_{table}_history;
            "#)).await?;
        }
        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        // Preserve recovered records and audit columns across a rollback.
        // The original create-table migrations still own the tables.
        Ok(())
    }
}
