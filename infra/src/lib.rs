pub mod book;
pub mod publisher;
// Re-export implementations if needed, or allow access via repositories::

#[cfg(debug_assertions)]
use sea_orm::{DatabaseConnection, DbErr};

#[cfg(debug_assertions)]
pub async fn init_db(db: &DatabaseConnection) -> Result<(), DbErr> {
    use sea_orm::{ConnectionTrait, Schema};

    let builder = db.get_database_backend();
    let schema = Schema::new(builder);

    let mut stmt1 = schema.create_table_from_entity(publisher::Entity);
    stmt1.if_not_exists();

    let mut stmt2 = schema.create_table_from_entity(book::Entity);
    stmt2.if_not_exists();

    let statements = vec![stmt1, stmt2];

    for stmt in statements {
        db.execute(&stmt).await?;
    }
    println!("Database schema initialized.");
    Ok(())
}
