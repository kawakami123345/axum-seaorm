use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[async_trait]
pub trait Repository: Sync + Send {
    async fn find_all(&self) -> anyhow::Result<Vec<Publisher>>;
    async fn find_by_id(&self, id: i32) -> anyhow::Result<Option<Publisher>>;
    async fn create(&self, item: Publisher) -> anyhow::Result<Publisher>;
    async fn update(&self, item: Publisher) -> anyhow::Result<Publisher>;
    async fn delete(&self, id: i32) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publisher {
    pub id: i32,
    pub name: String,
}

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Invalid format")]
    InvalidFormat,
    #[error("Domain rule violation")]
    DomainRuleViolation,
}
