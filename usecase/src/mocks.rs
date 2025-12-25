use async_trait::async_trait;
use domain::{
    Book, Publisher, interfaces::BookRepository, interfaces::PublisherRepository,
    models::DomainError,
};

use domain::interfaces::RepositoryBase;

pub struct MockBookRepository;
#[async_trait]
impl RepositoryBase<Book> for MockBookRepository {
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

#[async_trait]
impl BookRepository for MockBookRepository {}

pub struct MockPublisherRepository;
#[async_trait]
impl RepositoryBase<Publisher> for MockPublisherRepository {
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

#[async_trait]
impl PublisherRepository for MockPublisherRepository {}
