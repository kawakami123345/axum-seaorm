use crate::models::{Book, DomainError};
use async_trait::async_trait;

#[async_trait]
pub trait BookRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Book>, DomainError>;
    async fn find_by_id(&self, id: i32) -> Result<Book, DomainError>;
    async fn create(&self, book: Book) -> Result<Book, DomainError>;
    async fn update(&self, book: Book) -> Result<Book, DomainError>;
    async fn delete(&self, id: i32) -> Result<(), DomainError>;
}
