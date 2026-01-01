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
    shop: Option<shop::Shop>,
    applied_at: Option<chrono::DateTime<chrono::Utc>>,
    format: vo::BookFormat,
    price: vo::BookPrice,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    created_by: String,
    updated_by: String,
}

impl Book {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pub_id: uuid::Uuid,
        title: vo::BookTitle,
        author: vo::BookAuthor,
        publisher: publisher::Publisher,
        shop: Option<shop::Shop>,
        format: vo::BookFormat,
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
            shop,
            applied_at: None,
            format,
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
        shop: Option<shop::Shop>,
        applied_at: Option<chrono::DateTime<chrono::Utc>>,
        format: vo::BookFormat,
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
            shop,
            applied_at,
            format,
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
    pub fn pub_id(&self) -> uuid::Uuid {
        self.pub_id
    }
    pub fn title(&self) -> String {
        self.title.value().to_string()
    }
    pub fn author(&self) -> String {
        self.author.value().to_string()
    }
    pub fn publisher(&self) -> publisher::Publisher {
        self.publisher.clone()
    }
    pub fn shop(&self) -> Option<shop::Shop> {
        self.shop.clone()
    }
    pub fn applied_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.applied_at
    }
    pub fn format(&self) -> vo::BookFormat {
        self.format
    }
    pub fn price(&self) -> i32 {
        self.price.value()
    }
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.updated_at
    }
    pub fn created_by(&self) -> String {
        self.created_by.clone()
    }
    pub fn updated_by(&self) -> String {
        self.updated_by.clone()
    }

    fn update_audit(&mut self, updated_by: String) {
        self.updated_at = chrono::Utc::now();
        self.updated_by = updated_by;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        title: vo::BookTitle,
        author: vo::BookAuthor,
        publisher: publisher::Publisher,
        shop: Option<shop::Shop>,
        format: vo::BookFormat,
        price: vo::BookPrice,
        updated_by: String,
    ) -> Result<(), DomainError> {
        if self.applied_at.is_some() {
            return Err(DomainError::DomainRuleViolation(
                "Cannot update a book that is already applied.".to_string(),
            ));
        }

        self.title = title;
        self.author = author;
        self.publisher = publisher;
        self.shop = shop;
        self.format = format;
        self.price = price;
        self.update_audit(updated_by);
        Ok(())
    }

    pub fn change_applied_at(
        &mut self,
        applied_at: Option<chrono::DateTime<chrono::Utc>>,
        updated_by: String,
    ) -> Result<(), DomainError> {
        self.applied_at = applied_at;
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
