use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::schema::Schema;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(manager.get_database_backend());

        manager
            .create_table(schema.create_table_from_entity(infra::publisher::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(infra::book::Entity))
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(infra::publisher::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(infra::book::Entity).to_owned())
            .await?;
        Ok(())
    }
}
