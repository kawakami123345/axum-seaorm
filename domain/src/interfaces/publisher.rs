use crate::models::{DomainError, Publisher};
use async_trait::async_trait;

#[async_trait]
pub trait PublisherRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Publisher>, DomainError>;
    async fn find_by_id(&self, id: i32) -> Result<Publisher, DomainError>;
    async fn create(&self, publisher: Publisher) -> Result<Publisher, DomainError>;
    async fn update(&self, publisher: Publisher) -> Result<Publisher, DomainError>;
    async fn delete(&self, id: i32) -> Result<(), DomainError>;
}
