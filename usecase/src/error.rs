use axum::http::StatusCode;
use domain::DomainError;

#[derive(Debug)]
pub enum UseCaseError {
    NotFound,
    Internal(String),
}

impl From<DomainError> for UseCaseError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::NotFound => Self::NotFound,
            DomainError::InfraError(e) => Self::Internal(e),
        }
    }
}

pub fn map_error(err: UseCaseError) -> (StatusCode, String) {
    match err {
        UseCaseError::NotFound => (StatusCode::NOT_FOUND, "Not Found".to_string()),
        UseCaseError::Internal(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}
