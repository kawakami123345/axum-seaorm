use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use usecase::error::UseCaseError;

pub struct AppError(pub UseCaseError);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self.0 {
            UseCaseError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            UseCaseError::InternalServerError | UseCaseError::DatabaseError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ),
            UseCaseError::DomainRuleViolation(msg) => (StatusCode::BAD_REQUEST, msg),
            UseCaseError::BookDomainError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            UseCaseError::PublisherDomainError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            UseCaseError::ShopDomainError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

impl From<UseCaseError> for AppError {
    fn from(inner: UseCaseError) -> Self {
        AppError(inner)
    }
}
