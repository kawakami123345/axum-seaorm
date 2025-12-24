pub mod entities;
pub mod repositories;

// Re-export implementations if needed, or allow access via repositories::

use domain::models::DomainError;
use entities::{book, publisher};
use sea_orm::{ConnectionTrait, DatabaseConnection, Schema};

pub async fn init_db(db: &DatabaseConnection) -> Result<(), DomainError> {
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);

    let statements = vec![
        builder.build(
            schema
                .create_table_from_entity(publisher::Entity)
                .if_not_exists(),
        ),
        builder.build(
            schema
                .create_table_from_entity(book::Entity)
                .if_not_exists(),
        ),
    ];

    for stmt in statements {
        db.execute(stmt)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
    }

    Ok(())
}
