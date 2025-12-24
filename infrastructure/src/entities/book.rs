use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "book")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub author: String,
    pub publisher_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::publisher::Entity",
        from = "Column::PublisherId",
        to = "super::publisher::Column::Id"
    )]
    Publisher,
}

impl Related<super::publisher::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Publisher.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
