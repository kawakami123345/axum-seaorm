use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publisher {
    pub id: i32,
    pub name: String,
}
