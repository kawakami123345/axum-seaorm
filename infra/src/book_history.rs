use sea_orm::entity::prelude::*;
use sea_orm::sea_query::StringLen;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "book_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub history_id: i32,
    #[sea_orm(column_type = "String(StringLen::N(32))")]
    pub operation_type: String, // 'INSERT', 'UPDATE', 'DELETE'
    pub operation_at: chrono::DateTime<chrono::Utc>,

    // Copies from Book
    pub id: i32,
    pub pub_id: uuid::Uuid,
    #[sea_orm(column_type = "String(StringLen::N(32))")]
    pub title: String,
    #[sea_orm(column_type = "String(StringLen::N(32))")]
    pub author: String,
    pub publisher_id: i32,
    pub shop_id: Option<i32>,
    pub applied_at: Option<chrono::DateTime<chrono::Utc>>,
    #[sea_orm(column_type = "String(StringLen::N(32))")]
    pub format: String,
    pub price: i32,
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
    fn test_book_model_sync() {
        // この関数は Book と BookHistory のフィールドが一致していないとコンパイルエラーになります。
        // フィールドを追加した際は、両方の Model とこのテストを更新してください。
        fn _assert_sync(book: super::super::book::Model, history: Model) {
            let super::super::book::Model {
                id,
                pub_id,
                title,
                author,
                publisher_id,
                shop_id,
                applied_at,
                format,
                price,
                created_at,
                updated_at,
                created_by,
                updated_by,
            } = book;

            let Model {
                history_id: _,
                operation_type: _,
                operation_at: _,
                id: _,
                pub_id: _,
                title: _,
                author: _,
                publisher_id: _,
                shop_id: _,
                applied_at: _,
                format: _,
                price: _,
                created_at: _,
                updated_at: _,
                created_by: _,
                updated_by: _,
            } = history;

            // コンパイルエラーを防ぐために変数を使用
            let _ = (
                id,
                pub_id,
                title,
                author,
                publisher_id,
                shop_id,
                applied_at,
                format,
                price,
                created_at,
                updated_at,
                created_by,
                updated_by,
            );
        }
    }
}
