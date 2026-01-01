use serde::{Deserialize, Serialize};

use crate::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct PublisherName(String);

impl PublisherName {
    pub fn new(name: String) -> Result<Self, DomainError> {
        if name.chars().count() > 32 {
            return Err(DomainError::InvalidFormat(
                "Name must be 32 chars or less".to_string(),
            ));
        }
        Ok(Self(name))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
