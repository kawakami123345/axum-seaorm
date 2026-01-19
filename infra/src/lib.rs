use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, DatabaseTransaction, DbErr, Statement,
    TransactionTrait,
};
use uuid::Uuid;

pub mod book;
pub mod book_history;
pub mod publisher;
pub mod publisher_history;
pub mod shop;

#[async_trait::async_trait]
pub trait BeginWithUser {
    async fn begin_with_user(&self, user_id: &Uuid) -> Result<DatabaseTransaction, DbErr>;
}

#[async_trait::async_trait]
impl BeginWithUser for DatabaseConnection {
    async fn begin_with_user(&self, user_id: &Uuid) -> Result<DatabaseTransaction, DbErr> {
        let txn = self.begin().await?;

        txn.query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT set_config('app.current_user_id', $1, true)",
            vec![user_id.to_string().into()],
        ))
        .await?;

        Ok(txn)
    }
}
