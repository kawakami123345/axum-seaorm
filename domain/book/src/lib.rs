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
    id: i32,
    pub_id: uuid::Uuid,
    title: vo::BookTitle,
    author: vo::BookAuthor,
    publisher: publisher::Publisher,
    status: vo::BookStatus,
    price: vo::BookPrice,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    created_by: String,
    updated_by: String,
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

    #[allow(clippy::too_many_arguments)]
    pub fn reconstruct(
        id: i32,
        pub_id: uuid::Uuid,
        title: vo::BookTitle,
        author: vo::BookAuthor,
        publisher: publisher::Publisher,
        status: vo::BookStatus,
        price: vo::BookPrice,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
        created_by: String,
        updated_by: String,
    ) -> Self {
        Self {
            id,
            pub_id,
            title,
            author,
            publisher,
            status,
            price,
            created_at,
            updated_at,
            created_by,
            updated_by,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }
    pub fn pub_id(&self) -> &uuid::Uuid {
        &self.pub_id
    }
    pub fn title(&self) -> &vo::BookTitle {
        &self.title
    }
    pub fn author(&self) -> &vo::BookAuthor {
        &self.author
    }
    pub fn publisher(&self) -> &publisher::Publisher {
        &self.publisher
    }
    pub fn status(&self) -> &vo::BookStatus {
        &self.status
    }
    pub fn price(&self) -> &vo::BookPrice {
        &self.price
    }
    pub fn created_at(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.created_at
    }
    pub fn updated_at(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.updated_at
    }
    pub fn created_by(&self) -> &str {
        &self.created_by
    }
    pub fn updated_by(&self) -> &str {
        &self.updated_by
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
