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
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub updated_by: String,
}

impl Book {
    pub fn new(
        pub_id: uuid::Uuid,
        title: vo::BookTitle,
        author: vo::BookAuthor,
        publisher: publisher::Publisher,
        price: vo::BookPrice,
        created_by: String,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: 0,
            pub_id,
            title,
            author,
            publisher,
            status: vo::BookStatus::Unapplied,
            price,
            created_at: now,
            updated_at: now,
            created_by: created_by.clone(),
            updated_by: created_by,
        }
    }

    fn update_audit(&mut self, updated_by: String) {
        self.updated_at = chrono::Utc::now();
        self.updated_by = updated_by;
    }

    pub fn update(
        &mut self,
        title: vo::BookTitle,
        author: vo::BookAuthor,
        publisher: publisher::Publisher,
        price: vo::BookPrice,
        updated_by: String,
    ) -> Result<(), DomainError> {
        if self.status == vo::BookStatus::Applied {
            return Err(DomainError::DomainRuleViolation(
                "Cannot update a book that is already applied.".to_string(),
            ));
        }

        self.title = title;
        self.author = author;
        self.publisher = publisher;
        self.price = price;
        self.update_audit(updated_by);
        Ok(())
    }

    pub fn switch_status(&mut self, updated_by: String) -> Result<(), DomainError> {
        match self.status {
            vo::BookStatus::Unapplied => self.status = vo::BookStatus::Applied,
            vo::BookStatus::Applied => self.status = vo::BookStatus::Unapplied,
        }
        self.update_audit(updated_by);
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Domain rule violation: {0}")]
    DomainRuleViolation(String),
}
