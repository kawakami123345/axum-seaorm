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
}

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Domain rule violation: {0}")]
    DomainRuleViolation(String),
}
