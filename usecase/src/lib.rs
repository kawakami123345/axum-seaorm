pub mod book;
pub mod dashboard;
pub mod error;
pub mod publisher;
pub mod shop;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: String,
    pub roles: Vec<String>,
}

impl UserContext {
    pub fn new(user_id: String, roles: Vec<String>) -> Self {
        Self { user_id, roles }
    }

    pub fn is_admin(&self) -> bool {
        self.roles.iter().any(|r| r == "admin")
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn roles(&self) -> &Vec<String> {
        &self.roles
    }
}
