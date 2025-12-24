use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub publisher_id: i32,
}
