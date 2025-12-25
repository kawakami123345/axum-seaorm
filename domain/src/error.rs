use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Entity not found")]
    NotFound,
    #[error("Database error: {0}")]
    InfraError(String),
}
