use async_trait::async_trait;
use domain::{DomainError, RepositoryBase};
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

impl From<Model> for domain::publisher::Publisher {
    fn from(model: Model) -> Self {
        domain::publisher::Publisher {
            id: model.id,
            name: model.name,
        }
    }
}

#[async_trait]
impl RepositoryBase<domain::publisher::Publisher> for PostgresRepository {
    async fn find_all(&self) -> Result<Vec<domain::publisher::Publisher>, DomainError> {
        let publishers = Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| DomainError::InfraError(e.to_string()))?;

        Ok(publishers
            .into_iter()
            .map(domain::publisher::Publisher::from)
            .collect())
    }

    async fn find_by_id(&self, id: i32) -> Result<domain::publisher::Publisher, DomainError> {
        let publisher = Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| DomainError::InfraError(e.to_string()))?
            .ok_or(DomainError::NotFound)?;

        Ok(domain::publisher::Publisher::from(publisher))
    }

    async fn create(
        &self,
        item: domain::publisher::Publisher,
    ) -> Result<domain::publisher::Publisher, DomainError> {
        let active_model = ActiveModel {
            name: Set(item.name),
            ..Default::default() // id is ignored/auto-incremented
        };

        let result = active_model
            .insert(&self.db)
            .await
            .map_err(|e| DomainError::InfraError(e.to_string()))?;

        Ok(domain::publisher::Publisher::from(result))
    }

    async fn update(
        &self,
        item: domain::publisher::Publisher,
    ) -> Result<domain::publisher::Publisher, DomainError> {
        let active_model = ActiveModel {
            id: Set(item.id),
            name: Set(item.name),
        };

        let result = active_model
            .update(&self.db)
            .await
            .map_err(|e| DomainError::InfraError(e.to_string()))?;

        Ok(domain::publisher::Publisher::from(result))
    }

    async fn delete(&self, id: i32) -> Result<(), DomainError> {
        let result = Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| DomainError::InfraError(e.to_string()))?;

        if result.rows_affected == 0 {
            return Err(DomainError::NotFound);
        }

        Ok(())
    }
}

#[async_trait]
impl domain::publisher::Repository for PostgresRepository {}
