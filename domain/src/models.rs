pub mod book;
pub mod publisher;

pub use book::Book;
pub use publisher::Publisher;
pub use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Entity not found")]
    NotFound,
    #[error("Database error: {0}")]
    InfrastructureError(String),
}
