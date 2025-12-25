use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::RepositoryBase;

#[async_trait]
pub trait Repository: RepositoryBase<Publisher> {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publisher {
    pub id: i32,
    pub name: String,
}
