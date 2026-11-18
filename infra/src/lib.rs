use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, DatabaseTransaction, DbErr, Statement,
    TransactionTrait,
};
use uuid::Uuid;

pub mod book;
pub mod book_history;
pub mod publisher;
pub mod publisher_history;
pub mod session;
pub mod shop;

fn classify_name_error(error: DbErr, constraint: &str) -> anyhow::Error {
    use sea_orm::RuntimeErr;
    if let DbErr::Query(RuntimeErr::SqlxError(e)) | DbErr::Exec(RuntimeErr::SqlxError(e)) = &error
        && let Some(db) = e.as_database_error()
        && db.is_unique_violation()
        && db.constraint() == Some(constraint)
    {
        return usecase::error::DuplicateName.into();
    }
    error.into()
}

#[cfg(test)]
mod tests;

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
