pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20260101_000002_add_audit_and_history;
mod m20261114_000003_add_sessions_and_book_revision;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20260101_000002_add_audit_and_history::Migration),
            Box::new(m20261114_000003_add_sessions_and_book_revision::Migration),
        ]
    }
}
