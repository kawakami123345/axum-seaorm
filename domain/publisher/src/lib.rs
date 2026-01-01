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
    pub id: i32,
    pub pub_id: uuid::Uuid,
    pub name: vo::PublisherName,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub updated_by: String,
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
