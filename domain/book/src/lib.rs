use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod vo;

#[async_trait]
pub trait Repository: Sync + Send {
    async fn find_all(&self) -> anyhow::Result<Vec<Book>>;
    async fn find_by_pub_id(&self, pub_id: uuid::Uuid) -> anyhow::Result<Option<Book>>;
    async fn create(&self, item: Book) -> anyhow::Result<Book>;
    async fn update(&self, item: Book) -> anyhow::Result<Book>;
    async fn delete(&self, item: Book) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: i32,
    pub pub_id: uuid::Uuid,
    pub title: vo::BookTitle,
    pub author: vo::BookAuthor,
    pub publisher: publisher::Publisher,
    pub status: vo::BookStatus,
    pub price: vo::BookPrice,
}

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Domain rule violation: {0}")]
    DomainRuleViolation(String),
}
