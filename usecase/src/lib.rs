pub mod book;
pub mod dashboard;
pub mod error;
pub mod publisher;
pub mod session;
pub mod shop;

#[cfg(test)]
mod tests;

use ::book::ListFilter;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    user_id: Uuid,
    roles: Vec<String>,
}

impl UserContext {
    pub fn new(user_id: Uuid, roles: Vec<String>) -> Self {
        Self { user_id, roles }
    }

    pub fn is_admin(&self) -> bool {
        self.roles.iter().any(|r| r == "admin")
    }

    pub(crate) fn book_filter(&self) -> ListFilter {
        if self.is_admin() {
            ListFilter::All
        } else {
            ListFilter::OwnedBy(self.user_id)
        }
    }

    pub fn user_id(&self) -> &Uuid {
        &self.user_id
    }

    pub fn roles(&self) -> &Vec<String> {
        &self.roles
    }
}
