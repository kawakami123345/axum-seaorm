pub mod book;
pub mod error;
pub mod publisher;

use async_trait::async_trait;
pub use error::DomainError;

#[async_trait]
pub trait RepositoryBase<T>: Send + Sync {
    async fn find_all(&self) -> Result<Vec<T>, DomainError>;
    async fn find_by_id(&self, id: i32) -> Result<T, DomainError>;
    async fn create(&self, item: T) -> Result<T, DomainError>;
    async fn update(&self, item: T) -> Result<T, DomainError>;
    async fn delete(&self, id: i32) -> Result<(), DomainError>;
}
