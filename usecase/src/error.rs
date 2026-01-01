use thiserror::Error;

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("Entity not found")]
    NotFound(String),
    #[error("Internal server error")]
    InternalServerError,
    #[error("Database execution failed")]
    DatabaseError,
    #[error("Domain rule violation: {0}")]
    DomainRuleViolation(String),
    #[error("Domain error occurred: {0}")]
    BookDomainError(#[from] book::DomainError),
    #[error("Domain error occurred: {0}")]
    PublisherDomainError(#[from] publisher::DomainError),
}
