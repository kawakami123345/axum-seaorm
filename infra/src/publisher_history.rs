use sea_orm::entity::prelude::*;
use sea_orm::sea_query::StringLen;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "publisher_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub history_id: i32,
    #[sea_orm(column_type = "String(StringLen::N(32))")]
    pub operation_type: String, // 'INSERT', 'UPDATE', 'DELETE'
    pub operation_at: chrono::DateTime<chrono::Utc>,

    // Copies from Publisher
    pub id: i32,
    pub pub_id: uuid::Uuid,
    #[sea_orm(column_type = "String(StringLen::N(32))")]
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[sea_orm(column_type = "String(StringLen::N(32))")]
    pub created_by: String,
    #[sea_orm(column_type = "String(StringLen::N(32))")]
    pub updated_by: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publisher_model_sync() {
        fn _assert_sync(pub_orig: super::super::publisher::Model, history: Model) {
            let super::super::publisher::Model {
                id,
                pub_id,
                name,
                created_at,
                updated_at,
                created_by,
                updated_by,
            } = pub_orig;

            let Model {
                history_id: _,
                operation_type: _,
                operation_at: _,
                id: _,
                pub_id: _,
                name: _,
                created_at: _,
                updated_at: _,
                created_by: _,
                updated_by: _,
            } = history;

            let _ = (
                id, pub_id, name, created_at, updated_at, created_by, updated_by,
            );
        }
    }
}
