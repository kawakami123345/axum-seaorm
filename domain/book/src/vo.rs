use serde::{Deserialize, Serialize};
use std::fmt;

use crate::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct BookTitle(String);

impl BookTitle {
    pub fn new(title: String) -> Result<Self, DomainError> {
        if title.chars().count() > 32 {
            return Err(DomainError::InvalidFormat(
                "Title must be 32 chars or less".to_string(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BookFormat {
    #[default]
    Real,
    EBook,
}

impl BookFormat {
    pub fn as_str(&self) -> &str {
        match self {
            BookFormat::Real => "Real",
            BookFormat::EBook => "EBook",
        }
    }
}

impl fmt::Display for BookFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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
