use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Entity not found")]
    NotFound(String),
    #[error("Internal server error")]
    InternalServerError,
    #[error("Database execution failed")]
    DatabaseError,
    #[error("Domain error occurred: {0}")]
    BookDomainError(#[from] book::DomainError),
    #[error("Domain error occurred: {0}")]
    PublisherDomainError(#[from] publisher::DomainError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::InternalServerError | ApiError::DatabaseError => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            ApiError::BookDomainError(_) | ApiError::PublisherDomainError(_) => {
                StatusCode::BAD_REQUEST
            }
        }
        .into_response()
    }
}
