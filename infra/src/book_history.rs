use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "book_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub history_id: i32,
    pub operation_type: String, // 'INSERT', 'UPDATE', 'DELETE'
    pub operation_at: chrono::DateTime<chrono::Utc>,

    // Copies from Book
    pub id: i32,
    pub pub_id: uuid::Uuid,
    pub title: String,
    pub author: String,
    pub publisher_id: i32,
    pub status: String,
    pub price: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub updated_by: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
