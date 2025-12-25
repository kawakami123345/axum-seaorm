use crate::entities::publisher;
use async_trait::async_trait;
use domain::{Publisher, interfaces::PublisherRepository, models::DomainError};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

pub struct PublisherRepositoryImpl {
    pub(crate) db: DatabaseConnection,
}

impl PublisherRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<publisher::Model> for Publisher {
    fn from(model: publisher::Model) -> Self {
        Publisher {
            id: model.id,
            name: model.name,
        }
    }
}

use domain::interfaces::RepositoryBase;

#[async_trait]
impl RepositoryBase<Publisher> for PublisherRepositoryImpl {
    async fn find_all(&self) -> Result<Vec<Publisher>, DomainError> {
        let publishers = publisher::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(publishers.into_iter().map(Publisher::from).collect())
    }

    async fn find_by_id(&self, id: i32) -> Result<Publisher, DomainError> {
        let publisher = publisher::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?
            .ok_or(DomainError::NotFound)?;

        Ok(Publisher::from(publisher))
    }

    async fn create(&self, item: Publisher) -> Result<Publisher, DomainError> {
        let active_model = publisher::ActiveModel {
            name: Set(item.name),
            ..Default::default() // id is ignored/auto-incremented
        };

        let result = active_model
            .insert(&self.db)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(Publisher::from(result))
    }

    async fn update(&self, item: Publisher) -> Result<Publisher, DomainError> {
        let active_model = publisher::ActiveModel {
            id: Set(item.id),
            name: Set(item.name),
        };

        let result = active_model
            .update(&self.db)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(Publisher::from(result))
    }

    async fn delete(&self, id: i32) -> Result<(), DomainError> {
        let result = publisher::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        if result.rows_affected == 0 {
            return Err(DomainError::NotFound);
        }

        Ok(())
    }
}

#[async_trait]
impl PublisherRepository for PublisherRepositoryImpl {}
