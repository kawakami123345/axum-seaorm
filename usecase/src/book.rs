use crate::{UserContext, error::UseCaseError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use time::UtcOffset;
use utoipa::ToSchema;
use uuid::Uuid;

pub struct Service {
    repo: Arc<dyn book::Repository>,
    publisher_repo: Arc<dyn publisher::Repository>,
    shop_repo: Arc<dyn shop::Repository>,
}

impl Service {
    pub fn new(
        repo: Arc<dyn book::Repository>,
        publisher_repo: Arc<dyn publisher::Repository>,
        shop_repo: Arc<dyn shop::Repository>,
    ) -> Self {
        Self {
            repo,
            publisher_repo,
            shop_repo,
        }
    }

    pub async fn get_all(&self, ctx: &UserContext) -> Result<Vec<ResponseDto>, UseCaseError> {
        let books = self.find_authorized_books(ctx).await?;

        let response_dtos = books.into_iter().map(ResponseDto::from).collect();
        Ok(response_dtos)
    }

    pub async fn get_year_applied_books(
        &self,
        ctx: &UserContext,
        year: i32,
    ) -> Result<Vec<ResponseDto>, UseCaseError> {
        let books = self.find_authorized_books(ctx).await?;

        Ok(books
            .into_iter()
            .filter(|b| {
                b.applied_at()
                    .is_some_and(|at| at.to_offset(UtcOffset::UTC).year() == year)
            })
            .map(ResponseDto::from)
            .collect())
    }

    async fn find_authorized_books(
        &self,
        ctx: &UserContext,
    ) -> Result<Vec<book::Book>, UseCaseError> {
        self.repo
            .find_all(ctx.book_filter())
            .await
            .map_err(|e| UseCaseError::database("list books", e))
    }

    pub async fn get(&self, ctx: &UserContext, pub_id: Uuid) -> Result<ResponseDto, UseCaseError> {
        Ok(self.find_book(ctx, pub_id).await?.into())
    }

    async fn find_book(&self, ctx: &UserContext, pub_id: Uuid) -> Result<book::Book, UseCaseError> {
        self.repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|e| UseCaseError::database("find book", e))?
            .filter(|book| ctx.book_filter().matches(book))
            .ok_or_else(|| UseCaseError::NotFound("Book not found".into()))
    }

    async fn find_publisher(&self, pub_id: Uuid) -> Result<publisher::Publisher, UseCaseError> {
        self.publisher_repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|e| UseCaseError::database("find book publisher", e))?
            .ok_or_else(|| UseCaseError::DomainRuleViolation("Publisher not found".into()))
    }

    async fn find_shop(&self, pub_id: Option<Uuid>) -> Result<Option<shop::Shop>, UseCaseError> {
        let Some(pub_id) = pub_id else {
            return Ok(None);
        };

        self.shop_repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|e| UseCaseError::database("find book shop", e))?
            .ok_or_else(|| UseCaseError::DomainRuleViolation("Shop not found".into()))
            .map(Some)
    }

    pub async fn create(&self, ctx: &UserContext, dto: CreateDto) -> Result<(), UseCaseError> {
        let title = book::vo::BookTitle::new(dto.title)?;
        let author = book::vo::BookAuthor::new(dto.author)?;
        let price = book::vo::BookPrice::new(dto.price)?;
        let format = match dto.format.as_deref() {
            Some("EBook") => book::vo::BookFormat::EBook,
            _ => book::vo::BookFormat::Real,
        };

        let publisher = self.find_publisher(dto.publisher_id).await?;
        let shop = self.find_shop(dto.shop_id).await?;

        let book = book::Book::new(
            Uuid::now_v7(),
            title,
            author,
            publisher,
            shop,
            format,
            price,
            *ctx.user_id(),
            *ctx.user_id(),
        );

        self.repo
            .create(book)
            .await
            .map_err(|e| UseCaseError::database("create book", e))
    }

    pub async fn update(
        &self,
        ctx: &UserContext,
        pub_id: Uuid,
        dto: UpdateDto,
    ) -> Result<(), UseCaseError> {
        let title = book::vo::BookTitle::new(dto.title)?;
        let author = book::vo::BookAuthor::new(dto.author)?;
        let price = book::vo::BookPrice::new(dto.price)?;
        let format = match dto.format.as_deref() {
            Some("EBook") => book::vo::BookFormat::EBook,
            _ => book::vo::BookFormat::Real,
        };

        let mut book = self.find_book(ctx, pub_id).await?;

        let publisher = if book.publisher().pub_id() != dto.publisher_id {
            self.find_publisher(dto.publisher_id).await?
        } else {
            book.publisher().clone()
        };

        let shop = self.find_shop(dto.shop_id).await?;

        book.update(
            title,
            author,
            publisher,
            shop,
            format,
            price,
            *ctx.user_id(),
        )
        .map_err(|e| UseCaseError::DomainRuleViolation(e.to_string()))?;

        self.save_book(book, "update book").await
    }

    pub async fn delete(&self, ctx: &UserContext, pub_id: Uuid) -> Result<(), UseCaseError> {
        let book = self.find_book(ctx, pub_id).await?;

        self.repo
            .delete(book, *ctx.user_id())
            .await
            .map_err(|e| UseCaseError::database("delete book", e))
    }

    pub async fn change_applied_at(
        &self,
        ctx: &UserContext,
        pub_id: Uuid,
        dto: ChangeAppliedAtDto,
    ) -> Result<ResponseDto, UseCaseError> {
        let mut book = self.find_book(ctx, pub_id).await?;

        book.change_applied_at(dto.applied_at, *ctx.user_id())
            .map_err(|e| UseCaseError::DomainRuleViolation(e.to_string()))?;

        let response = ResponseDto::from(&book);
        self.save_book(book, "change book applied date").await?;

        Ok(response)
    }

    async fn save_book(&self, book: book::Book, operation: &str) -> Result<(), UseCaseError> {
        let updated = self
            .repo
            .update(book)
            .await
            .map_err(|e| UseCaseError::database(operation, e))?;
        if !updated {
            return Err(UseCaseError::Conflict(
                "別の操作で書籍が更新されました。再読み込みしてからやり直してください。".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = BookCreateDto)]
pub struct CreateDto {
    pub title: String,
    pub author: String,
    pub publisher_id: uuid::Uuid,
    pub shop_id: Option<uuid::Uuid>,
    #[schema(value_type = Option<String>, example = "Real")]
    pub format: Option<String>,
    pub price: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = BookUpdateDto)]
pub struct UpdateDto {
    pub title: String,
    pub author: String,
    pub publisher_id: uuid::Uuid,
    pub shop_id: Option<uuid::Uuid>,
    #[schema(value_type = Option<String>, example = "Real")]
    pub format: Option<String>,
    pub price: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = BookChangeAppliedAtDto)]
pub struct ChangeAppliedAtDto {
    #[schema(value_type = Option<String>, example = "2024-01-01T00:00:00Z")]
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub applied_at: Option<time::OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = BookResponseDto)]
pub struct ResponseDto {
    pub pub_id: uuid::Uuid,
    pub title: String,
    pub author: String,
    pub publisher: BookPublisherDto,
    pub shop: Option<BookShopDto>,
    #[schema(value_type = Option<String>, example = "2024-01-01T00:00:00Z")]
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub applied_at: Option<time::OffsetDateTime>,
    #[schema(value_type = String, example = "Real")]
    pub format: String,
    pub price: i32,
}

impl From<book::Book> for ResponseDto {
    fn from(book: book::Book) -> Self {
        Self::from(&book)
    }
}

impl From<&book::Book> for ResponseDto {
    fn from(book: &book::Book) -> Self {
        Self {
            pub_id: book.pub_id(),
            title: book.title().to_string(),
            author: book.author().to_string(),
            publisher: BookPublisherDto {
                pub_id: book.publisher().pub_id(),
                name: book.publisher().name().to_string(),
            },
            shop: book.shop().as_ref().map(|s| BookShopDto {
                pub_id: s.pub_id(),
                name: s.name().to_string(),
            }),
            applied_at: book.applied_at(),
            format: book.format().to_string(),
            price: book.price(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = BookPublisherDto)]
pub struct BookPublisherDto {
    pub pub_id: uuid::Uuid,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = BookShopDto)]
pub struct BookShopDto {
    pub pub_id: uuid::Uuid,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use rstest::*;
    use std::{str::FromStr, sync::Mutex};
    use uuid::Uuid;

    struct FakeRepository {
        store: Arc<Mutex<Vec<book::Book>>>,
    }

    impl FakeRepository {
        fn new() -> Self {
            Self {
                store: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl book::Repository for FakeRepository {
        async fn find_all(&self, filter: book::ListFilter) -> anyhow::Result<Vec<book::Book>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter(|book| filter.matches(book))
                .cloned()
                .collect())
        }

        async fn find_by_pub_id(&self, pub_id: uuid::Uuid) -> anyhow::Result<Option<book::Book>> {
            let store = self.store.lock().unwrap();
            Ok(store.iter().find(|b| b.pub_id() == pub_id).cloned())
        }

        async fn create(&self, item: book::Book) -> anyhow::Result<()> {
            let mut store = self.store.lock().unwrap();
            let new_id = store.iter().map(|b| b.id()).max().unwrap_or(0) + 1;

            let new_book = book::Book::reconstruct(
                new_id,
                item.pub_id(),
                book::vo::BookTitle::new(item.title().to_string()).unwrap(),
                book::vo::BookAuthor::new(item.author().to_string()).unwrap(),
                item.publisher().clone(),
                item.shop().clone(),
                item.applied_at(),
                item.format(),
                book::vo::BookPrice::new(item.price()).unwrap(),
                item.created_at(),
                item.updated_at(),
                *item.created_by(),
                *item.updated_by(),
                *item.user_id(),
                item.revision(),
            );

            store.push(new_book);
            Ok(())
        }

        async fn update(&self, item: book::Book) -> anyhow::Result<bool> {
            let mut store = self.store.lock().unwrap();
            if let Some(index) = store.iter().position(|b| b.id() == item.id()) {
                if store[index].revision() != item.revision() {
                    return Ok(false);
                }
                let item = book::Book::reconstruct(
                    item.id(),
                    item.pub_id(),
                    book::vo::BookTitle::new(item.title().to_string()).unwrap(),
                    book::vo::BookAuthor::new(item.author().to_string()).unwrap(),
                    item.publisher().clone(),
                    item.shop().clone(),
                    item.applied_at(),
                    item.format(),
                    book::vo::BookPrice::new(item.price()).unwrap(),
                    item.created_at(),
                    item.updated_at(),
                    *item.created_by(),
                    *item.updated_by(),
                    *item.user_id(),
                    item.revision() + 1,
                );
                store[index] = item;
                Ok(true)
            } else {
                Ok(false)
            }
        }

        async fn delete(&self, item: book::Book, _deleted_by: uuid::Uuid) -> anyhow::Result<()> {
            let mut store = self.store.lock().unwrap();
            store.retain(|b| b.pub_id() != item.pub_id());
            Ok(())
        }
    }

    struct FakePublisherRepository {
        store: Arc<Mutex<Vec<publisher::Publisher>>>,
    }

    impl FakePublisherRepository {
        fn new() -> Self {
            Self {
                store: Arc::new(Mutex::new(Vec::new())),
            }
        }
        fn add(&self, item: publisher::Publisher) {
            self.store.lock().unwrap().push(item);
        }
    }

    #[async_trait]
    impl publisher::Repository for FakePublisherRepository {
        async fn find_all(&self) -> anyhow::Result<Vec<publisher::Publisher>> {
            Ok(self.store.lock().unwrap().clone())
        }
        async fn find_by_pub_id(
            &self,
            pub_id: uuid::Uuid,
        ) -> anyhow::Result<Option<publisher::Publisher>> {
            Ok(self
                .store
                .lock()
                .unwrap()
                .iter()
                .find(|p| p.pub_id() == pub_id)
                .cloned())
        }
        async fn create(&self, item: publisher::Publisher) -> anyhow::Result<()> {
            self.store.lock().unwrap().push(item.clone());
            Ok(())
        }
        async fn update(&self, _item: publisher::Publisher) -> anyhow::Result<()> {
            panic!("Not implemented")
        }
        async fn delete(
            &self,
            _item: publisher::Publisher,
            _delete_by: Uuid,
        ) -> anyhow::Result<()> {
            panic!("Not implemented")
        }
    }

    struct FakeShopRepository {
        store: Arc<Mutex<Vec<shop::Shop>>>,
    }

    impl FakeShopRepository {
        fn new() -> Self {
            Self {
                store: Arc::new(Mutex::new(Vec::new())),
            }
        }
        fn add(&self, item: shop::Shop) {
            self.store.lock().unwrap().push(item);
        }
    }

    #[async_trait]
    impl shop::Repository for FakeShopRepository {
        async fn find_all(&self) -> anyhow::Result<Vec<shop::Shop>> {
            Ok(self.store.lock().unwrap().clone())
        }
        async fn find_by_pub_id(&self, pub_id: uuid::Uuid) -> anyhow::Result<Option<shop::Shop>> {
            Ok(self
                .store
                .lock()
                .unwrap()
                .iter()
                .find(|s| s.pub_id() == pub_id)
                .cloned())
        }
        async fn create(&self, item: shop::Shop) -> anyhow::Result<()> {
            self.store.lock().unwrap().push(item.clone());
            Ok(())
        }
        async fn update(&self, _item: shop::Shop) -> anyhow::Result<()> {
            panic!("Not implemented")
        }
        async fn delete(&self, _item: shop::Shop, _deleted_by: Uuid) -> anyhow::Result<()> {
            panic!("Not implemented")
        }
    }

    #[fixture]
    async fn service() -> (
        Service,
        Arc<FakePublisherRepository>,
        Arc<FakeShopRepository>,
    ) {
        let repo = FakeRepository::new();
        let pub_repo = Arc::new(FakePublisherRepository::new());
        let shop_repo = Arc::new(FakeShopRepository::new());
        (
            Service::new(Arc::new(repo), pub_repo.clone(), shop_repo.clone()),
            pub_repo,
            shop_repo,
        )
    }

    fn create_dummy_publisher(pub_id: Uuid) -> publisher::Publisher {
        publisher::Publisher::new(
            pub_id,
            publisher::vo::PublisherName::new("Test Publisher".to_string()).unwrap(),
            Uuid::from_str("11111111-1234-5678-90ab-cdef12345678").unwrap(),
        )
    }

    fn create_dummy_shop(pub_id: Uuid) -> shop::Shop {
        shop::Shop::new(
            pub_id,
            shop::vo::ShopName::new("Test Shop".to_string()).unwrap(),
            Uuid::from_str("11111111-1234-5678-90ab-cdef12345678").unwrap(),
        )
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_and_get(
        #[future] service: (
            Service,
            Arc<FakePublisherRepository>,
            Arc<FakeShopRepository>,
        ),
    ) {
        let (service, pub_repo, shop_repo) = service.await;
        let pub_id = uuid::Uuid::new_v4();
        let shop_id = uuid::Uuid::new_v4();

        // Setup
        pub_repo.add(create_dummy_publisher(pub_id));
        shop_repo.add(create_dummy_shop(shop_id));

        let dto = CreateDto {
            title: "Test Book".to_string(),
            author: "Author 1".to_string(),
            publisher_id: pub_id,
            shop_id: Some(shop_id),
            format: Some("Real".to_string()),
            price: 1000,
        };

        let ctx = UserContext::new(
            Uuid::from_str("11111111-1234-5678-90ab-cdef12345678").unwrap(),
            vec![],
        );
        // Create
        service
            .create(&ctx, dto)
            .await
            .expect("Failed to create book");

        let mut all = service.get_all(&ctx).await.expect("Failed to get all");
        assert_eq!(all.len(), 1);
        let created = all.remove(0);
        assert_eq!(created.title, "Test Book");
        assert_eq!(created.shop.unwrap().pub_id, shop_id);
        assert_eq!(created.format, "Real");

        // Get
        let fetched = service
            .get(&ctx, created.pub_id)
            .await
            .expect("Failed to get book");
        assert_eq!(fetched.pub_id, created.pub_id);
        assert_eq!(fetched.shop.unwrap().name, "Test Shop");
    }

    #[rstest]
    #[tokio::test]
    async fn a_stale_save_returns_a_conflict_without_undoing_the_application(
        #[future] service: (
            Service,
            Arc<FakePublisherRepository>,
            Arc<FakeShopRepository>,
        ),
    ) {
        let (service, publishers, _) = service.await;
        let publisher_id = Uuid::new_v4();
        publishers.add(create_dummy_publisher(publisher_id));
        let ctx = UserContext::new(Uuid::new_v4(), vec![]);
        service
            .create(
                &ctx,
                CreateDto {
                    title: "Book".into(),
                    author: "Author".into(),
                    publisher_id,
                    shop_id: None,
                    format: None,
                    price: 1000,
                },
            )
            .await
            .unwrap();
        let pub_id = service.get_all(&ctx).await.unwrap()[0].pub_id;
        let stale = service.find_book(&ctx, pub_id).await.unwrap();
        let applied_at = time::OffsetDateTime::now_utc();
        service
            .change_applied_at(
                &ctx,
                pub_id,
                ChangeAppliedAtDto {
                    applied_at: Some(applied_at),
                },
            )
            .await
            .unwrap();

        assert!(matches!(
            service.save_book(stale, "save stale book").await,
            Err(UseCaseError::Conflict(_))
        ));
        assert_eq!(
            service.get(&ctx, pub_id).await.unwrap().applied_at,
            Some(applied_at)
        );
    }

    #[rstest]
    #[tokio::test]
    async fn applied_at_preserves_rfc3339_and_uses_the_utc_year(
        #[future] service: (
            Service,
            Arc<FakePublisherRepository>,
            Arc<FakeShopRepository>,
        ),
    ) {
        let (service, publishers, _) = service.await;
        let publisher_id = Uuid::new_v4();
        publishers.add(create_dummy_publisher(publisher_id));
        let ctx = UserContext::new(Uuid::new_v4(), vec![]);
        service
            .create(
                &ctx,
                CreateDto {
                    title: "Year boundary".into(),
                    author: "Author".into(),
                    publisher_id,
                    shop_id: None,
                    format: None,
                    price: 100,
                },
            )
            .await
            .unwrap();
        let pub_id = service.get_all(&ctx).await.unwrap()[0].pub_id;
        let dto = serde_json::from_str(r#"{"applied_at":"2025-01-01T00:30:00.123456789+09:00"}"#)
            .unwrap();
        let response = service.change_applied_at(&ctx, pub_id, dto).await.unwrap();

        assert_eq!(
            serde_json::to_value(response).unwrap()["applied_at"],
            "2024-12-31T15:30:00.123456789Z"
        );
        assert_eq!(
            service
                .get_year_applied_books(&ctx, 2024)
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(
            service
                .get_year_applied_books(&ctx, 2025)
                .await
                .unwrap()
                .is_empty()
        );
        let dashboard = crate::dashboard::Service::new(service.repo.clone());
        let summaries = dashboard.get_annual_summary(&ctx).await.unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].year, 2024);

        for json in [r#"{"applied_at":null}"#, "{}"] {
            let dto = serde_json::from_str(json).unwrap();
            let response = service.change_applied_at(&ctx, pub_id, dto).await.unwrap();
            assert!(serde_json::to_value(response).unwrap()["applied_at"].is_null());
            assert!(dashboard.get_annual_summary(&ctx).await.unwrap().is_empty());
        }

        let outside_utc_range = time::Date::MAX
            .with_hms(23, 59, 59)
            .unwrap()
            .assume_offset(UtcOffset::from_hms(-23, -59, 0).unwrap());
        let result = service
            .change_applied_at(
                &ctx,
                pub_id,
                ChangeAppliedAtDto {
                    applied_at: Some(outside_utc_range),
                },
            )
            .await;
        assert!(matches!(result, Err(UseCaseError::DomainRuleViolation(_))));
        assert!(
            service
                .get(&ctx, pub_id)
                .await
                .unwrap()
                .applied_at
                .is_none()
        );
    }

    #[rstest]
    #[tokio::test]
    async fn update_changes_publisher_and_can_set_and_clear_shop(
        #[future] service: (
            Service,
            Arc<FakePublisherRepository>,
            Arc<FakeShopRepository>,
        ),
    ) {
        let (service, publishers, shops) = service.await;
        let publisher_id = Uuid::new_v4();
        let next_publisher_id = Uuid::new_v4();
        let shop_id = Uuid::new_v4();
        publishers.add(create_dummy_publisher(publisher_id));
        publishers.add(create_dummy_publisher(next_publisher_id));
        shops.add(create_dummy_shop(shop_id));
        let ctx = UserContext::new(Uuid::new_v4(), vec![]);
        service
            .create(
                &ctx,
                CreateDto {
                    title: "Original".into(),
                    author: "Author".into(),
                    publisher_id,
                    shop_id: None,
                    format: None,
                    price: 100,
                },
            )
            .await
            .unwrap();
        let pub_id = service.get_all(&ctx).await.unwrap()[0].pub_id;

        for shop_id in [Some(shop_id), None] {
            service
                .update(
                    &ctx,
                    pub_id,
                    UpdateDto {
                        title: "Updated".into(),
                        author: "New Author".into(),
                        publisher_id: next_publisher_id,
                        shop_id,
                        format: Some("EBook".into()),
                        price: 200,
                    },
                )
                .await
                .unwrap();

            let updated = service.get(&ctx, pub_id).await.unwrap();
            assert_eq!(updated.title, "Updated");
            assert_eq!(updated.author, "New Author");
            assert_eq!(updated.publisher.pub_id, next_publisher_id);
            assert_eq!(updated.shop.map(|shop| shop.pub_id), shop_id);
            assert_eq!(updated.format, "EBook");
            assert_eq!(updated.price, 200);
        }
    }

    #[rstest]
    #[case::create_without_publisher(false, true)]
    #[case::create_without_shop(false, false)]
    #[case::update_without_publisher(true, true)]
    #[case::update_without_shop(true, false)]
    #[tokio::test]
    async fn missing_related_entity_does_not_persist_book_changes(
        #[future] service: (
            Service,
            Arc<FakePublisherRepository>,
            Arc<FakeShopRepository>,
        ),
        #[case] updating: bool,
        #[case] missing_publisher: bool,
    ) {
        let (service, publishers, _) = service.await;
        let publisher_id = Uuid::new_v4();
        publishers.add(create_dummy_publisher(publisher_id));
        let ctx = UserContext::new(Uuid::new_v4(), vec![]);
        let create_dto = || CreateDto {
            title: "Original".into(),
            author: "Author".into(),
            publisher_id,
            shop_id: None,
            format: None,
            price: 100,
        };
        if updating {
            service.create(&ctx, create_dto()).await.unwrap();
        }

        let mut dto = create_dto();
        dto.title = "Changed".into();
        dto.price = 200;
        if missing_publisher {
            dto.publisher_id = Uuid::new_v4();
        } else {
            dto.shop_id = Some(Uuid::new_v4());
        }
        let result = if updating {
            let pub_id = service.get_all(&ctx).await.unwrap()[0].pub_id;
            service
                .update(
                    &ctx,
                    pub_id,
                    UpdateDto {
                        title: dto.title,
                        author: dto.author,
                        publisher_id: dto.publisher_id,
                        shop_id: dto.shop_id,
                        format: dto.format,
                        price: dto.price,
                    },
                )
                .await
        } else {
            service.create(&ctx, dto).await
        };

        let expected = if missing_publisher {
            "Publisher not found"
        } else {
            "Shop not found"
        };
        assert!(matches!(
            result,
            Err(UseCaseError::DomainRuleViolation(message)) if message == expected
        ));
        let books = service.get_all(&ctx).await.unwrap();
        if updating {
            assert_eq!(books.len(), 1);
            assert_eq!(books[0].title, "Original");
            assert_eq!(books[0].price, 100);
            assert_eq!(books[0].publisher.pub_id, publisher_id);
            assert!(books[0].shop.is_none());
        } else {
            assert!(books.is_empty());
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_all(
        #[future] service: (
            Service,
            Arc<FakePublisherRepository>,
            Arc<FakeShopRepository>,
        ),
    ) {
        let (service, pub_repo, _) = service.await;
        let pub_id_1 = uuid::Uuid::new_v4();
        pub_repo.add(create_dummy_publisher(pub_id_1));

        let dto = CreateDto {
            title: "Book 1".to_string(),
            author: "Author 1".to_string(),
            publisher_id: pub_id_1,
            shop_id: None,
            format: None,
            price: 100,
        };
        let ctx = UserContext::new(
            Uuid::from_str("11111111-1234-5678-90ab-cdef12345678").unwrap(),
            vec![],
        );
        service.create(&ctx, dto).await.expect("Failed to create");

        let all = service.get_all(&ctx).await.expect("Failed to get all");
        assert_eq!(all.len(), 1);
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_all_filters_by_owner_unless_admin(
        #[future] service: (
            Service,
            Arc<FakePublisherRepository>,
            Arc<FakeShopRepository>,
        ),
    ) {
        let (service, pub_repo, _) = service.await;
        let publisher_id = uuid::Uuid::new_v4();
        pub_repo.add(create_dummy_publisher(publisher_id));

        let user_id = Uuid::from_str("11111111-1234-5678-90ab-cdef12345678").unwrap();
        let other_user_id = Uuid::from_str("22222222-1234-5678-90ab-cdef12345678").unwrap();
        let user_ctx = UserContext::new(user_id, vec![]);
        let other_user_ctx = UserContext::new(other_user_id, vec![]);

        service
            .create(
                &user_ctx,
                CreateDto {
                    title: "User Book".to_string(),
                    author: "Author 1".to_string(),
                    publisher_id,
                    shop_id: None,
                    format: None,
                    price: 100,
                },
            )
            .await
            .expect("Failed to create user book");
        service
            .create(
                &other_user_ctx,
                CreateDto {
                    title: "Other User Book".to_string(),
                    author: "Author 2".to_string(),
                    publisher_id,
                    shop_id: None,
                    format: None,
                    price: 200,
                },
            )
            .await
            .expect("Failed to create other user book");

        let user_books = service.get_all(&user_ctx).await.expect("Failed to get all");
        assert_eq!(user_books.len(), 1);
        assert_eq!(user_books[0].title, "User Book");

        let admin_ctx = UserContext::new(user_id, vec!["admin".to_string()]);
        let admin_books = service
            .get_all(&admin_ctx)
            .await
            .expect("Failed to get all as admin");
        assert_eq!(admin_books.len(), 2);

        let other_books = service.get_all(&other_user_ctx).await.unwrap();
        assert_eq!(other_books.len(), 1);
        assert_eq!(other_books[0].title, "Other User Book");

        let stranger = UserContext::new(Uuid::new_v4(), vec!["editor".into()]);
        assert!(service.get_all(&stranger).await.unwrap().is_empty());
    }

    #[rstest]
    #[case::owner(true, vec![], true)]
    #[case::other_user(false, vec![], false)]
    #[case::other_role(false, vec!["editor".into()], false)]
    #[case::admin(false, vec!["admin".into()], true)]
    #[tokio::test]
    async fn book_operations_require_owner_or_admin(
        #[future] service: (
            Service,
            Arc<FakePublisherRepository>,
            Arc<FakeShopRepository>,
        ),
        #[case] owns_book: bool,
        #[case] roles: Vec<String>,
        #[case] allowed: bool,
    ) {
        let (service, pub_repo, _) = service.await;
        let publisher_id = Uuid::new_v4();
        pub_repo.add(create_dummy_publisher(publisher_id));
        let owner = UserContext::new(Uuid::new_v4(), vec![]);
        let actor = UserContext::new(
            if owns_book {
                *owner.user_id()
            } else {
                Uuid::new_v4()
            },
            roles,
        );
        service
            .create(
                &owner,
                CreateDto {
                    title: "Original".into(),
                    author: "Author".into(),
                    publisher_id,
                    shop_id: None,
                    format: None,
                    price: 100,
                },
            )
            .await
            .unwrap();
        let pub_id = service.get_all(&owner).await.unwrap()[0].pub_id;

        let fetched = service.get(&actor, pub_id).await;
        let updated = service
            .update(
                &actor,
                pub_id,
                UpdateDto {
                    title: "Updated".into(),
                    author: "Author".into(),
                    publisher_id,
                    shop_id: None,
                    format: None,
                    price: 200,
                },
            )
            .await;
        let applied_at = time::Date::from_calendar_date(2024, time::Month::January, 1)
            .unwrap()
            .midnight()
            .assume_utc();
        let applied = service
            .change_applied_at(
                &actor,
                pub_id,
                ChangeAppliedAtDto {
                    applied_at: Some(applied_at),
                },
            )
            .await;

        let stored = service.repo.find_by_pub_id(pub_id).await.unwrap().unwrap();
        assert_eq!(stored.user_id(), owner.user_id());
        let deleted = service.delete(&actor, pub_id).await;

        if allowed {
            assert_eq!(fetched.unwrap().title, "Original");
            updated.unwrap();
            let applied = applied.unwrap();
            assert_eq!(applied.title, "Updated");
            assert_eq!(applied.applied_at, Some(applied_at));
            assert_eq!(stored.updated_by(), actor.user_id());
            deleted.unwrap();
            assert!(service.repo.find_by_pub_id(pub_id).await.unwrap().is_none());
        } else {
            assert!(matches!(fetched, Err(UseCaseError::NotFound(_))));
            assert!(matches!(updated, Err(UseCaseError::NotFound(_))));
            assert!(matches!(applied, Err(UseCaseError::NotFound(_))));
            assert!(matches!(deleted, Err(UseCaseError::NotFound(_))));
            let unchanged = service.get(&owner, pub_id).await.unwrap();
            assert_eq!(unchanged.title, "Original");
            assert_eq!(unchanged.price, 100);
            assert!(unchanged.applied_at.is_none());
            assert_eq!(stored.updated_by(), owner.user_id());
        }
    }

    #[rstest]
    #[tokio::test]
    async fn year_applied_books_filter_by_owner_and_year(
        #[future] service: (
            Service,
            Arc<FakePublisherRepository>,
            Arc<FakeShopRepository>,
        ),
    ) {
        let (service, _, _) = service.await;
        let user_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();
        for (owner_id, year) in [
            (user_id, Some(2024)),
            (other_user_id, Some(2024)),
            (user_id, Some(2023)),
            (user_id, None),
        ] {
            let mut book = book::Book::new(
                Uuid::new_v4(),
                book::vo::BookTitle::new("Book".into()).unwrap(),
                book::vo::BookAuthor::new("Author".into()).unwrap(),
                create_dummy_publisher(Uuid::new_v4()),
                None,
                book::vo::BookFormat::Real,
                book::vo::BookPrice::new(100).unwrap(),
                owner_id,
                owner_id,
            );
            if let Some(year) = year {
                book.change_applied_at(
                    Some(
                        time::Date::from_calendar_date(year, time::Month::January, 1)
                            .unwrap()
                            .midnight()
                            .assume_utc(),
                    ),
                    owner_id,
                )
                .unwrap();
            }
            service.repo.create(book).await.unwrap();
        }

        let user = UserContext::new(user_id, vec![]);
        assert_eq!(
            service
                .get_year_applied_books(&user, 2024)
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(
            service
                .get_year_applied_books(&user, 2022)
                .await
                .unwrap()
                .is_empty()
        );

        let admin = UserContext::new(user_id, vec!["admin".into()]);
        assert_eq!(
            service
                .get_year_applied_books(&admin, 2024)
                .await
                .unwrap()
                .len(),
            2
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_delete(
        #[future] service: (
            Service,
            Arc<FakePublisherRepository>,
            Arc<FakeShopRepository>,
        ),
    ) {
        let (service, pub_repo, _) = service.await;
        let pub_id = uuid::Uuid::new_v4();
        pub_repo.add(create_dummy_publisher(pub_id));

        let dto = CreateDto {
            title: "Book To Delete".to_string(),
            author: "Author".to_string(),
            publisher_id: pub_id,
            shop_id: None,
            format: None,
            price: 100,
        };
        let ctx = UserContext::new(
            Uuid::from_str("11111111-1234-5678-90ab-cdef12345678").unwrap(),
            vec![],
        );
        service
            .create(&ctx, dto)
            .await
            .expect("Failed to create book");

        let mut all = service.get_all(&ctx).await.expect("Failed to get all");
        assert_eq!(all.len(), 1);
        let created = all.remove(0);

        service
            .delete(&ctx, created.pub_id)
            .await
            .expect("Failed to delete");
        assert!(service.get(&ctx, created.pub_id).await.is_err());
    }
}
