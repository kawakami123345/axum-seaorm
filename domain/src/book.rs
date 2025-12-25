use crate::RepositoryBase;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait Repository: RepositoryBase<Book> {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub publisher_id: i32,
}
