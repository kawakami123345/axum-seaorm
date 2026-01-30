use crate::{UserContext, cedar, error::UseCaseError};
use chrono::Datelike;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

pub struct Service {
    book_repo: Arc<dyn book::Repository>,
}

impl Service {
    pub fn new(book_repo: Arc<dyn book::Repository>) -> Self {
        Self { book_repo }
    }

    pub async fn get_annual_summary(
        &self,
        ctx: &UserContext,
    ) -> Result<Vec<DashboardDto>, UseCaseError> {
        cedar::authorize_dashboard_get_annual_summary(ctx)?;

        let books = self
            .book_repo
            .find_all()
            .await
            .map_err(|_| UseCaseError::DatabaseError)?;

        let mut applied_books: Vec<book::Book> = books
            .into_iter()
            .filter(|b| b.applied_at().is_some())
            .collect();

        // Sort by year for grouping
        applied_books.sort_by_key(|b| b.applied_at().unwrap().year());

        let summaries = applied_books
            .into_iter()
            .chunk_by(|b| b.applied_at().unwrap().year())
            .into_iter()
            .map(|(year, group)| {
                let books: Vec<book::Book> = group.collect();
                let count = books.len() as i32;
                let total_amount: i32 = books.iter().map(|b| b.price()).sum();
                let limit = 20000;
                let balance = limit - total_amount;
                let average = if count > 0 {
                    total_amount as f64 / count as f64
                } else {
                    0.0
                };

                DashboardDto {
                    year,
                    limit,
                    total_amount,
                    balance,
                    count,
                    average,
                }
            })
            .collect();

        Ok(summaries)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = DashboardDto)]
pub struct DashboardDto {
    pub year: i32,
    pub limit: i32,
    pub total_amount: i32,
    pub balance: i32,
    pub count: i32,
    pub average: f64,
}
