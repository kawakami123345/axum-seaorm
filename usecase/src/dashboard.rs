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
        let books = self.find_authorized_books(ctx).await?;

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

    async fn find_authorized_books(
        &self,
        ctx: &UserContext,
    ) -> Result<Vec<book::Book>, UseCaseError> {
        match cedar::authorize_book_query(ctx, cedar::ACTION_GET_ANNUAL_SUMMARY)? {
            cedar::PolicyEvaluation::Allow => self
                .book_repo
                .find_all_by_filter(book::ListFilter::All)
                .await
                .map_err(|_| UseCaseError::DatabaseError),
            cedar::PolicyEvaluation::Deny => Ok(Vec::new()),
            cedar::PolicyEvaluation::Filter(filter) => self
                .book_repo
                .find_all_by_filter(filter)
                .await
                .map_err(|_| UseCaseError::DatabaseError),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::TimeZone;
    use std::{
        str::FromStr,
        sync::{Arc, Mutex},
    };
    use uuid::Uuid;

    struct FakeBookRepository {
        store: Arc<Mutex<Vec<book::Book>>>,
    }

    impl FakeBookRepository {
        fn new() -> Self {
            Self {
                store: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn add(&self, book: book::Book) {
            self.store.lock().unwrap().push(book);
        }
    }

    #[async_trait]
    impl book::Repository for FakeBookRepository {
        async fn find_all(&self) -> anyhow::Result<Vec<book::Book>> {
            Ok(self.store.lock().unwrap().clone())
        }

        async fn find_by_pub_id(&self, pub_id: Uuid) -> anyhow::Result<Option<book::Book>> {
            Ok(self
                .store
                .lock()
                .unwrap()
                .iter()
                .find(|book| book.pub_id() == pub_id)
                .cloned())
        }

        async fn create(&self, item: book::Book) -> anyhow::Result<()> {
            self.add(item);
            Ok(())
        }

        async fn update(&self, item: book::Book) -> anyhow::Result<()> {
            let mut store = self.store.lock().unwrap();
            if let Some(index) = store.iter().position(|book| book.pub_id() == item.pub_id()) {
                store[index] = item;
            }
            Ok(())
        }

        async fn delete(&self, item: book::Book, _deleted_by: Uuid) -> anyhow::Result<()> {
            self.store
                .lock()
                .unwrap()
                .retain(|book| book.pub_id() != item.pub_id());
            Ok(())
        }
    }

    fn applied_book(user_id: Uuid, price: i32, year: i32) -> book::Book {
        let publisher = publisher::Publisher::new(
            Uuid::new_v4(),
            publisher::vo::PublisherName::new("Test Publisher".to_string()).unwrap(),
            user_id,
        );
        let mut book = book::Book::new(
            Uuid::new_v4(),
            book::vo::BookTitle::new("Test Book".to_string()).unwrap(),
            book::vo::BookAuthor::new("Test Author".to_string()).unwrap(),
            publisher,
            None,
            book::vo::BookFormat::Real,
            book::vo::BookPrice::new(price).unwrap(),
            user_id,
            user_id,
        );
        book.change_applied_at(
            Some(chrono::Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).unwrap()),
            user_id,
        )
        .unwrap();
        book
    }

    #[tokio::test]
    async fn annual_summary_filters_books_with_cedar_rls() {
        let repo = Arc::new(FakeBookRepository::new());
        let user_id = Uuid::from_str("11111111-1234-5678-90ab-cdef12345678").unwrap();
        let other_user_id = Uuid::from_str("22222222-1234-5678-90ab-cdef12345678").unwrap();
        repo.add(applied_book(user_id, 1000, 2024));
        repo.add(applied_book(other_user_id, 2000, 2024));

        let service = Service::new(repo);
        let user_ctx = UserContext::new(user_id, vec![]);
        let summaries = service
            .get_annual_summary(&user_ctx)
            .await
            .expect("summary should be available");
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].count, 1);
        assert_eq!(summaries[0].total_amount, 1000);

        let admin_ctx = UserContext::new(user_id, vec!["admin".to_string()]);
        let summaries = service
            .get_annual_summary(&admin_ctx)
            .await
            .expect("admin summary should be available");
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].count, 2);
        assert_eq!(summaries[0].total_amount, 3000);
    }
}
