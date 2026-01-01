use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod vo;

#[async_trait]
pub trait Repository: Sync + Send {
    async fn find_all(&self) -> anyhow::Result<Vec<Publisher>>;
    async fn find_by_pub_id(&self, pub_id: uuid::Uuid) -> anyhow::Result<Option<Publisher>>;
    async fn create(&self, item: Publisher) -> anyhow::Result<Publisher>;
    async fn update(&self, item: Publisher) -> anyhow::Result<Publisher>;
    async fn delete(&self, item: Publisher) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publisher {
    id: i32,
    pub_id: uuid::Uuid,
    name: vo::PublisherName,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    created_by: String,
    updated_by: String,
}

impl Publisher {
    pub fn new(pub_id: uuid::Uuid, name: vo::PublisherName, created_by: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: 0,
            pub_id,
            name,
            created_at: now,
            updated_at: now,
            created_by: created_by.clone(),
            updated_by: created_by,
        }
    }

    pub fn reconstruct(
        id: i32,
        pub_id: uuid::Uuid,
        name: vo::PublisherName,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
        created_by: String,
        updated_by: String,
    ) -> Self {
        Self {
            id,
            pub_id,
            name,
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
    pub fn name(&self) -> String {
        self.name.value().to_string()
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

    pub fn update(
        &mut self,
        name: vo::PublisherName,
        updated_by: String,
    ) -> Result<(), DomainError> {
        self.name = name;
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
