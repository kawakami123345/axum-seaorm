use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Keep the publisher/history columns in the same order for the existing
        // audit trigger. Fresh databases may already include the new columns.
        for table in ["publisher", "publisher_history", "shop"] {
            if !manager.has_column(table, "revision").await? {
                manager
                    .alter_table(
                        Table::alter()
                            .table(Alias::new(table))
                            .add_column(
                                ColumnDef::new(Alias::new("revision"))
                                    .big_integer()
                                    .not_null()
                                    .default(0),
                            )
                            .to_owned(),
                    )
                    .await?;
            }
        }

        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
            CREATE FUNCTION guard_record_update() RETURNS TRIGGER AS $$
            BEGIN
                IF NEW.id IS DISTINCT FROM OLD.id
                    OR NEW.pub_id IS DISTINCT FROM OLD.pub_id THEN
                    RAISE EXCEPTION 'id and pub_id are immutable on %', TG_TABLE_NAME
                        USING ERRCODE = '23514';
                END IF;
                -- Every write advances the version, including direct SQL writes.
                NEW.revision := OLD.revision + 1;
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;
            "#,
        )
        .await?;

        for table in ["book", "publisher", "shop"] {
            db.execute_unprepared(&format!(
                "CREATE TRIGGER guard_{table}_update BEFORE UPDATE ON {table} \
                 FOR EACH ROW EXECUTE FUNCTION guard_record_update()"
            ))
            .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        for table in ["book", "publisher", "shop"] {
            db.execute_unprepared(&format!("DROP TRIGGER guard_{table}_update ON {table}"))
                .await?;
        }
        db.execute_unprepared("DROP FUNCTION guard_record_update()")
            .await?;

        for table in ["publisher_history", "publisher", "shop"] {
            manager
                .alter_table(
                    Table::alter()
                        .table(Alias::new(table))
                        .drop_column(Alias::new("revision"))
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
}
