use serde::{Deserialize, Serialize};

use crate::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct BookTitle(String);

impl BookTitle {
    pub fn new(title: String) -> Result<Self, DomainError> {
        if title.chars().count() > 128 {
            return Err(DomainError::InvalidFormat(
                "Title must be 128 chars or less".to_string(),
            ));
        }
        Ok(Self(title))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct BookAuthor(String);

impl BookAuthor {
    pub fn new(author: String) -> Result<Self, DomainError> {
        if author.chars().count() > 32 {
            return Err(DomainError::InvalidFormat(
                "Author must be 32 chars or less".to_string(),
            ));
        }
        Ok(Self(author))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BookStatus {
    Unapplied,
    Applied,
}

impl BookStatus {
    pub fn value(&self) -> &str {
        match self {
            BookStatus::Unapplied => "Unapplied",
            BookStatus::Applied => "Applied",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct BookPrice(i32);

impl BookPrice {
    pub fn new(price: i32) -> Result<Self, DomainError> {
        if price < 0 {
            return Err(DomainError::InvalidFormat(
                "Price must be 0 or more".to_string(),
            ));
        }
        Ok(Self(price))
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}
