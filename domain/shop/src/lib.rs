use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub mod vo;

#[async_trait]
pub trait Repository: Sync + Send {
    async fn find_all(&self) -> anyhow::Result<Vec<Shop>>;
    async fn find_by_pub_id(&self, pub_id: uuid::Uuid) -> anyhow::Result<Option<Shop>>;
    async fn create(&self, item: Shop) -> anyhow::Result<()>;
    async fn update(&self, item: Shop) -> anyhow::Result<()>;
    async fn delete(&self, item: Shop, deleted_by: Uuid) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shop {
    id: i32,
    pub_id: uuid::Uuid,
    name: vo::ShopName,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    created_by: Uuid,
    updated_by: Uuid,
}

impl Shop {
    pub fn new(pub_id: uuid::Uuid, name: vo::ShopName, created_by: Uuid) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: 0,
            pub_id,
            name,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }

    pub fn reconstruct(
        id: i32,
        pub_id: uuid::Uuid,
        name: vo::ShopName,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
        created_by: Uuid,
        updated_by: Uuid,
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
    pub fn name(&self) -> &str {
        self.name.value()
    }
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.updated_at
    }
    pub fn created_by(&self) -> &Uuid {
        &self.created_by
    }
    pub fn updated_by(&self) -> &Uuid {
        &self.updated_by
    }

    fn update_audit(&mut self, updated_by: Uuid) {
        self.updated_at = chrono::Utc::now();
        self.updated_by = updated_by;
    }

    pub fn update(&mut self, name: vo::ShopName, updated_by: Uuid) -> Result<(), DomainError> {
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
