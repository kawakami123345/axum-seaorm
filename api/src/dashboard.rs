use crate::AppState;
use crate::error::AppError;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/dashboard/annual-summary",
    tag = "Dashboard",
    operation_id = "get_annual_summary",
    responses(
        (status = 200, description = "Annual summary", body = [usecase::dashboard::DashboardDto])
    )
)]
pub async fn get_annual_summary(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.dashboard_usecase.get_annual_summary().await {
        Ok(summary) => (StatusCode::OK, Json(summary)).into_response(),
        Err(e) => AppError(e).into_response(),
    }
}
