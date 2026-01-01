use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "publisher_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub history_id: i32,
    pub operation_type: String, // 'INSERT', 'UPDATE', 'DELETE'
    pub operation_at: chrono::DateTime<chrono::Utc>,

    // Copies from Publisher
    pub id: i32,
    pub pub_id: uuid::Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub updated_by: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
