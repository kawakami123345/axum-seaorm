use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "publisher")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::book::Entity")]
    Book,
}

impl Related<super::book::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Book.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub struct PostgresRepository {
    pub(crate) db: DatabaseConnection,
}

impl PostgresRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<Model> for publisher::Publisher {
    fn from(model: Model) -> Self {
        publisher::Publisher {
            id: model.id,
            name: model.name,
        }
    }
}

#[async_trait]
impl publisher::Repository for PostgresRepository {
    async fn find_all(&self) -> anyhow::Result<Vec<publisher::Publisher>> {
        let publishers = Entity::find().all(&self.db).await?;
        Ok(publishers
            .into_iter()
            .map(publisher::Publisher::from)
            .collect())
    }

    async fn find_by_id(&self, id: i32) -> anyhow::Result<Option<publisher::Publisher>> {
        let publisher = Entity::find_by_id(id).one(&self.db).await?;
        Ok(publisher.map(publisher::Publisher::from))
    }

    async fn create(&self, item: publisher::Publisher) -> anyhow::Result<publisher::Publisher> {
        let active_model = ActiveModel {
            name: Set(item.name),
            ..Default::default() // id is ignored/auto-incremented
        };

        let result = active_model.insert(&self.db).await?;
        Ok(publisher::Publisher::from(result))
    }

    async fn update(&self, item: publisher::Publisher) -> anyhow::Result<publisher::Publisher> {
        let active_model = ActiveModel {
            id: Set(item.id),
            name: Set(item.name),
        };

        let result = active_model.update(&self.db).await?;
        Ok(publisher::Publisher::from(result))
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        let _ = Entity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }
}
