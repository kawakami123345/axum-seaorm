use async_trait::async_trait;
use domain::{Book, BookRepository, Publisher, PublisherRepository, models::DomainError};

pub struct MockBookRepository;
#[async_trait]
impl BookRepository for MockBookRepository {
    async fn find_all(&self) -> Result<Vec<Book>, DomainError> {
        Ok(vec![])
    }
    async fn find_by_id(&self, _id: i32) -> Result<Book, DomainError> {
        Err(DomainError::NotFound)
    }
    async fn create(&self, _book: Book) -> Result<Book, DomainError> {
        Err(DomainError::NotFound)
    }
    async fn update(&self, _book: Book) -> Result<Book, DomainError> {
        Err(DomainError::NotFound)
    }
    async fn delete(&self, _id: i32) -> Result<(), DomainError> {
        Ok(())
    }
}

pub struct MockPublisherRepository;
#[async_trait]
impl PublisherRepository for MockPublisherRepository {
    async fn find_all(&self) -> Result<Vec<Publisher>, DomainError> {
        Ok(vec![])
    }
    async fn find_by_id(&self, _id: i32) -> Result<Publisher, DomainError> {
        Err(DomainError::NotFound)
    }
    async fn create(&self, _publisher: Publisher) -> Result<Publisher, DomainError> {
        Err(DomainError::NotFound)
    }
    async fn update(&self, _publisher: Publisher) -> Result<Publisher, DomainError> {
        Err(DomainError::NotFound)
    }
    async fn delete(&self, _id: i32) -> Result<(), DomainError> {
        Ok(())
    }
}
