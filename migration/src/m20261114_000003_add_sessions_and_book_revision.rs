use sea_orm_migration::{prelude::*, sea_orm::Schema};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Earlier migrations derive tables from entities, so fresh databases may
        // already have these columns. Existing databases need both appended in
        // the same order for the history trigger's (NEW).* / (OLD).* expansion.
        for table in ["book", "book_history"] {
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

        let schema = Schema::new(manager.get_database_backend());
        manager
            .create_table(schema.create_table_from_entity(infra::session::Entity))
            .await?;
        for index in schema.create_index_from_entity(infra::session::Entity) {
            manager.create_index(index).await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(infra::session::Entity).to_owned())
            .await?;
        for table in ["book_history", "book"] {
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
