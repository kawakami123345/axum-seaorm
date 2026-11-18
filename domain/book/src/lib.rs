use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub mod vo;

#[async_trait]
pub trait Repository: Sync + Send {
    async fn find_all(&self, filter: ListFilter) -> anyhow::Result<Vec<Book>>;
    async fn annual_totals(&self, filter: ListFilter) -> anyhow::Result<Vec<AnnualTotals>>;
    async fn find_by_pub_id(&self, pub_id: Uuid) -> anyhow::Result<Option<Book>>;
    async fn create(&self, item: Book) -> anyhow::Result<()>;
    /// Returns false when the stored revision no longer matches the loaded book.
    async fn update(&self, item: Book) -> anyhow::Result<bool>;
    /// Returns false when the record was updated or deleted after it was loaded.
    async fn delete(&self, item: Book, deleted_by: Uuid) -> anyhow::Result<bool>;
}

#[derive(Debug, PartialEq)]
pub struct AnnualTotals {
    pub year: i32,
    pub count: i64,
    pub total_amount: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListFilter {
    All,
    OwnedBy(Uuid),
}

impl ListFilter {
    pub fn matches(&self, book: &Book) -> bool {
        match self {
            Self::All => true,
            Self::OwnedBy(user_id) => !user_id.is_nil() && book.user_id() == user_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    id: i32,
    pub_id: Uuid,
    title: vo::BookTitle,
    author: vo::BookAuthor,
    publisher: publisher::Publisher,
    shop: Option<shop::Shop>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    applied_at: Option<time::OffsetDateTime>,
    #[serde(default)]
    applied_without_date: bool,
    format: vo::BookFormat,
    price: vo::BookPrice,
    #[serde(with = "time::serde::rfc3339")]
    created_at: time::OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    updated_at: time::OffsetDateTime,
    created_by: Uuid,
    updated_by: Uuid,
    user_id: Uuid,
    revision: i64,
}

impl Book {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pub_id: Uuid,
        title: vo::BookTitle,
        author: vo::BookAuthor,
        publisher: publisher::Publisher,
        shop: Option<shop::Shop>,
        format: vo::BookFormat,
        price: vo::BookPrice,
        created_by: Uuid,
        user_id: Uuid,
    ) -> Self {
        let now = time::OffsetDateTime::now_utc();
        Self {
            id: 0,
            pub_id,
            title,
            author,
            publisher,
            shop,
            applied_at: None,
            applied_without_date: false,
            format,
            price,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
            user_id,
            revision: 0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstruct(
        id: i32,
        pub_id: Uuid,
        title: vo::BookTitle,
        author: vo::BookAuthor,
        publisher: publisher::Publisher,
        shop: Option<shop::Shop>,
        applied_at: Option<time::OffsetDateTime>,
        applied_without_date: bool,
        format: vo::BookFormat,
        price: vo::BookPrice,
        created_at: time::OffsetDateTime,
        updated_at: time::OffsetDateTime,
        created_by: Uuid,
        updated_by: Uuid,
        user_id: Uuid,
        revision: i64,
    ) -> Self {
        Self {
            id,
            pub_id,
            title,
            author,
            publisher,
            shop,
            applied_at,
            applied_without_date,
            format,
            price,
            created_at,
            updated_at,
            created_by,
            updated_by,
            user_id,
            revision,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }
    pub fn pub_id(&self) -> Uuid {
        self.pub_id
    }
    pub fn title(&self) -> &str {
        self.title.value()
    }
    pub fn author(&self) -> &str {
        self.author.value()
    }
    pub fn publisher(&self) -> &publisher::Publisher {
        &self.publisher
    }
    pub fn shop(&self) -> &Option<shop::Shop> {
        &self.shop
    }
    pub fn applied_at(&self) -> Option<time::OffsetDateTime> {
        self.applied_at
    }
    pub fn applied_without_date(&self) -> bool {
        self.applied_without_date
    }
    pub fn format(&self) -> vo::BookFormat {
        self.format
    }
    pub fn price(&self) -> i32 {
        self.price.value()
    }
    pub fn created_at(&self) -> time::OffsetDateTime {
        self.created_at
    }
    pub fn updated_at(&self) -> time::OffsetDateTime {
        self.updated_at
    }
    pub fn created_by(&self) -> &Uuid {
        &self.created_by
    }
    pub fn updated_by(&self) -> &Uuid {
        &self.updated_by
    }
    pub fn user_id(&self) -> &Uuid {
        &self.user_id
    }
    pub fn revision(&self) -> i64 {
        self.revision
    }

    fn update_audit(&mut self, updated_by: Uuid) {
        self.updated_at = time::OffsetDateTime::now_utc();
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
        updated_by: Uuid,
    ) -> Result<(), DomainError> {
        if self.applied_at.is_some() || self.applied_without_date {
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
        applied_at: Option<time::OffsetDateTime>,
        updated_by: Uuid,
    ) -> Result<(), DomainError> {
        let applied_at = applied_at
            .map(|at| {
                at.checked_to_offset(time::UtcOffset::UTC).ok_or_else(|| {
                    DomainError::InvalidFormat(
                        "Applied date is outside the supported UTC range".into(),
                    )
                })
            })
            .transpose()?;
        self.applied_at = applied_at;
        self.applied_without_date = false;
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
