use crate::{UserContext, error::UseCaseError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

const ANNUAL_BUDGET: i64 = 20_000;

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
        let annual_totals = self
            .book_repo
            .annual_totals(ctx.book_filter())
            .await
            .map_err(|e| UseCaseError::database("aggregate annual book totals", e))?;

        Ok(annual_totals
            .into_iter()
            .map(|totals| DashboardDto {
                year: totals.year,
                limit: ANNUAL_BUDGET,
                total_amount: totals.total_amount,
                balance: ANNUAL_BUDGET - totals.total_amount,
                count: totals.count,
                average: totals.total_amount as f64 / totals.count as f64,
            })
            .collect())
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = DashboardDto)]
pub struct DashboardDto {
    pub year: i32,
    pub limit: i64,
    pub total_amount: i64,
    pub balance: i64,
    pub count: i64,
    pub average: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
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
        async fn annual_totals(
            &self,
            filter: book::ListFilter,
        ) -> anyhow::Result<Vec<book::AnnualTotals>> {
            Ok(crate::tests::annual_totals(self.find_all(filter).await?))
        }

        async fn find_all(&self, filter: book::ListFilter) -> anyhow::Result<Vec<book::Book>> {
            Ok(self
                .store
                .lock()
                .unwrap()
                .iter()
                .filter(|book| filter.matches(book))
                .cloned()
                .collect())
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

        async fn update(&self, item: book::Book) -> anyhow::Result<bool> {
            let mut store = self.store.lock().unwrap();
            if let Some(index) = store.iter().position(|book| book.pub_id() == item.pub_id()) {
                store[index] = item;
            }
            Ok(true)
        }

        async fn delete(&self, item: book::Book, _deleted_by: Uuid) -> anyhow::Result<bool> {
            let mut store = self.store.lock().unwrap();
            let len = store.len();
            store.retain(|book| {
                book.pub_id() != item.pub_id() || book.revision() != item.revision()
            });
            Ok(store.len() != len)
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
            Some(
                time::Date::from_calendar_date(year, time::Month::January, 1)
                    .unwrap()
                    .midnight()
                    .assume_utc(),
            ),
            user_id,
        )
        .unwrap();
        book
    }

    #[tokio::test]
    async fn annual_summary_calculates_totals_in_ascending_year_order() {
        let repo = Arc::new(FakeBookRepository::new());
        let user_id = Uuid::new_v4();
        repo.add(applied_book(user_id, 15_000, 2025));
        repo.add(applied_book(user_id, 0, 2023));
        repo.add(applied_book(user_id, 10_001, 2025));
        let service = Service::new(repo);
        let ctx = UserContext::new(user_id, vec![]);

        let summaries = service.get_annual_summary(&ctx).await.unwrap();
        assert_eq!(summaries.len(), 2);
        let zero_price = &summaries[0];
        assert_eq!(zero_price.year, 2023);
        assert_eq!(zero_price.count, 1);
        assert_eq!(zero_price.limit, 20_000);
        assert_eq!(zero_price.total_amount, 0);
        assert_eq!(zero_price.balance, 20_000);
        assert_eq!(zero_price.average, 0.0);

        let over_budget = &summaries[1];
        assert_eq!(over_budget.year, 2025);
        assert_eq!(over_budget.count, 2);
        assert_eq!(over_budget.limit, 20_000);
        assert_eq!(over_budget.total_amount, 25_001);
        assert_eq!(over_budget.balance, -5_001);
        assert_eq!(over_budget.average, 12_500.5);
    }

    #[tokio::test]
    async fn annual_summary_supports_totals_above_the_maximum_book_price() {
        let repo = Arc::new(FakeBookRepository::new());
        let user_id = Uuid::new_v4();
        repo.add(applied_book(user_id, i32::MAX, 2026));
        repo.add(applied_book(user_id, i32::MAX, 2026));

        let summaries = Service::new(repo)
            .get_annual_summary(&UserContext::new(user_id, vec![]))
            .await
            .unwrap();

        assert_eq!(summaries[0].total_amount, 4_294_967_294);
        assert_eq!(summaries[0].balance, -4_294_947_294);
        assert_eq!(summaries[0].average, f64::from(i32::MAX));
    }

    #[tokio::test]
    async fn annual_summary_is_empty_without_applied_books() {
        let repo = Arc::new(FakeBookRepository::new());
        let service = Service::new(repo.clone());
        let user_id = Uuid::new_v4();
        let ctx = UserContext::new(user_id, vec![]);
        assert!(service.get_annual_summary(&ctx).await.unwrap().is_empty());

        let mut unapplied = applied_book(user_id, 1_000, 2024);
        unapplied.change_applied_at(None, user_id).unwrap();
        repo.add(unapplied);
        assert!(service.get_annual_summary(&ctx).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn annual_summary_filters_books_by_owner_unless_admin() {
        let repo = Arc::new(FakeBookRepository::new());
        let user_id = Uuid::from_str("11111111-1234-5678-90ab-cdef12345678").unwrap();
        let other_user_id = Uuid::from_str("22222222-1234-5678-90ab-cdef12345678").unwrap();
        repo.add(applied_book(user_id, 1000, 2024));
        repo.add(applied_book(other_user_id, 2000, 2024));
        repo.add(applied_book(other_user_id, 500, 2023));
        let mut unapplied = applied_book(user_id, 900, 2025);
        unapplied.change_applied_at(None, user_id).unwrap();
        repo.add(unapplied);

        let service = Service::new(repo);
        let user_ctx = UserContext::new(user_id, vec![]);
        let summaries = service
            .get_annual_summary(&user_ctx)
            .await
            .expect("summary should be available");
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].year, 2024);
        assert_eq!(summaries[0].count, 1);
        assert_eq!(summaries[0].total_amount, 1000);

        let admin_ctx = UserContext::new(user_id, vec!["admin".to_string()]);
        let summaries = service
            .get_annual_summary(&admin_ctx)
            .await
            .expect("admin summary should be available");
        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[0].year, 2023);
        assert_eq!(summaries[0].count, 1);
        assert_eq!(summaries[0].total_amount, 500);
        assert_eq!(summaries[1].year, 2024);
        assert_eq!(summaries[1].count, 2);
        assert_eq!(summaries[1].total_amount, 3000);

        let stranger = UserContext::new(Uuid::new_v4(), vec![]);
        assert!(
            service
                .get_annual_summary(&stranger)
                .await
                .unwrap()
                .is_empty()
        );
    }
}
