use crate::{UserContext, error::UseCaseError};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

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
        let books = self.repo.find_all().await.map_err(|e| {
            eprintln!("Database error in create book (find publisher): {:?}", e);
            UseCaseError::DatabaseError
        })?;

        let response_dtos = books
            .into_iter()
            .filter(|b| ctx.is_admin() || b.user_id() == ctx.user_id())
            .map(ResponseDto::from)
            .collect();
        Ok(response_dtos)
    }

    pub async fn get_year_applied_books(
        &self,
        ctx: &UserContext,
        year: i32,
    ) -> Result<Vec<ResponseDto>, UseCaseError> {
        let books = self.repo.find_all().await.map_err(|e| {
            eprintln!("Database error in create book (find publisher): {:?}", e);
            UseCaseError::DatabaseError
        })?;

        let response_dtos = books
            .into_iter()
            .filter(|b| {
                (ctx.is_admin() || b.user_id() == ctx.user_id())
                    && b.applied_at().map_or(false, |at| at.year() == year)
            })
            .map(ResponseDto::from)
            .collect();

        Ok(response_dtos)
    }

    pub async fn get(
        &self,
        ctx: &UserContext,
        pub_id: uuid::Uuid,
    ) -> Result<ResponseDto, UseCaseError> {
        let book = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|e| {
                eprintln!("Database error in create book (find publisher): {:?}", e);
                UseCaseError::DatabaseError
            })?
            .filter(|b| ctx.is_admin() || b.user_id() == ctx.user_id())
            .ok_or(UseCaseError::NotFound("Book not found".into()))?;

        Ok(book.into())
    }

    pub async fn create(
        &self,
        ctx: &UserContext,
        dto: CreateDto,
    ) -> Result<ResponseDto, UseCaseError> {
        let title = book::vo::BookTitle::new(dto.title)?;
        let author = book::vo::BookAuthor::new(dto.author)?;
        let price = book::vo::BookPrice::new(dto.price)?;
        let format = match dto.format.as_deref() {
            Some("EBook") => book::vo::BookFormat::EBook,
            _ => book::vo::BookFormat::Real,
        };

        let publisher = self
            .publisher_repo
            .find_by_pub_id(dto.publisher_id)
            .await
            .map_err(|e| {
                eprintln!("Database error in create book (find publisher): {:?}", e);
                UseCaseError::DatabaseError
            })?
            .ok_or(UseCaseError::DomainRuleViolation(
                "Publisher not found".into(),
            ))?;

        let shop = if let Some(shop_id) = dto.shop_id {
            self.shop_repo
                .find_by_pub_id(shop_id)
                .await
                .map_err(|e| {
                    eprintln!("Database error in create book (find shop): {:?}", e);
                    UseCaseError::DatabaseError
                })?
                .ok_or(UseCaseError::DomainRuleViolation("Shop not found".into()))
                .map(Some)?
        } else {
            None
        };

        let book = book::Book::new(
            uuid::Uuid::now_v7(),
            title,
            author,
            publisher,
            shop,
            format,
            price,
            ctx.user_id.clone(),
            ctx.user_id.clone(),
        );
        self.repo.create(book.clone()).await.map_err(|e| {
            eprintln!("Database error in create book: {:?}", e);
            UseCaseError::DatabaseError
        })?;

        Ok(book.into())
    }

    pub async fn update(
        &self,
        ctx: &UserContext,
        pub_id: uuid::Uuid,
        dto: UpdateDto,
    ) -> Result<ResponseDto, UseCaseError> {
        let title = book::vo::BookTitle::new(dto.title)?;
        let author = book::vo::BookAuthor::new(dto.author)?;
        let price = book::vo::BookPrice::new(dto.price)?;
        let format = match dto.format.as_deref() {
            Some("EBook") => book::vo::BookFormat::EBook,
            _ => book::vo::BookFormat::Real,
        };

        let mut book = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|e| {
                eprintln!("Database error in create book (find publisher): {:?}", e);
                UseCaseError::DatabaseError
            })?
            .filter(|b| ctx.is_admin() || b.user_id() == ctx.user_id())
            .ok_or(UseCaseError::NotFound("Book not found".to_string()))?;

        // Resolve Publisher
        let publisher = if book.publisher().pub_id() != dto.publisher_id {
            self.publisher_repo
                .find_by_pub_id(dto.publisher_id)
                .await
                .map_err(|e| {
                    eprintln!("Database error in create book (find publisher): {:?}", e);
                    UseCaseError::DatabaseError
                })?
                .ok_or(UseCaseError::DomainRuleViolation(
                    "Publisher not found".to_string(),
                ))?
        } else {
            book.publisher().clone()
        };

        // Resolve Shop
        let shop = if let Some(shop_id) = dto.shop_id {
            Some(
                self.shop_repo
                    .find_by_pub_id(shop_id)
                    .await
                    .map_err(|e| {
                        eprintln!("Database error in create book (find publisher): {:?}", e);
                        UseCaseError::DatabaseError
                    })?
                    .ok_or(UseCaseError::DomainRuleViolation(
                        "Shop not found".to_string(),
                    ))?,
            )
        } else {
            None
        };

        book.update(
            title,
            author,
            publisher,
            shop,
            format,
            price,
            ctx.user_id.clone(),
        )
        .map_err(|e| UseCaseError::DomainRuleViolation(e.to_string()))?;

        self.repo.update(book.clone()).await.map_err(|e| {
            eprintln!("Database error in create book (find publisher): {:?}", e);
            UseCaseError::DatabaseError
        })?;

        Ok(book.into())
    }

    pub async fn delete(&self, ctx: &UserContext, pub_id: uuid::Uuid) -> Result<(), UseCaseError> {
        let book = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|e| {
                eprintln!("Database error in create book (find publisher): {:?}", e);
                UseCaseError::DatabaseError
            })?
            .filter(|b| ctx.is_admin() || b.user_id() == ctx.user_id())
            .ok_or(UseCaseError::NotFound("Book not found".to_string()))?;

        self.repo.delete(book).await.map_err(|e| {
            eprintln!("Database error in create book (find publisher): {:?}", e);
            UseCaseError::DatabaseError
        })?;
        Ok(())
    }
    pub async fn change_applied_at(
        &self,
        ctx: &UserContext,
        pub_id: uuid::Uuid,
        dto: ChangeAppliedAtDto,
    ) -> Result<ResponseDto, UseCaseError> {
        let mut book = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|e| {
                eprintln!("Database error in create book (find publisher): {:?}", e);
                UseCaseError::DatabaseError
            })?
            .filter(|b| ctx.is_admin() || b.user_id() == ctx.user_id())
            .ok_or(UseCaseError::NotFound("Book not found".to_string()))?;

        book.change_applied_at(dto.applied_at, ctx.user_id.clone())
            .map_err(|e| UseCaseError::DomainRuleViolation(e.to_string()))?;

        self.repo.update(book.clone()).await.map_err(|e| {
            eprintln!("Database error in create book (find publisher): {:?}", e);
            UseCaseError::DatabaseError
        })?;

        Ok(book.into())
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
    pub applied_at: Option<chrono::DateTime<chrono::Utc>>,
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
    pub applied_at: Option<chrono::DateTime<chrono::Utc>>,
    #[schema(value_type = String, example = "Real")]
    pub format: String,
    pub price: i32,
}

impl From<book::Book> for ResponseDto {
    fn from(book: book::Book) -> Self {
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
        async fn find_all(&self) -> anyhow::Result<Vec<book::Book>> {
            let store = self.store.lock().unwrap();
            Ok(store.clone())
        }

        async fn find_by_pub_id(&self, pub_id: uuid::Uuid) -> anyhow::Result<Option<book::Book>> {
            let store = self.store.lock().unwrap();
            Ok(store.iter().find(|b| b.pub_id() == pub_id).cloned())
        }

        async fn create(&self, item: book::Book) -> anyhow::Result<book::Book> {
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
                item.created_by().clone(),
                item.updated_by().clone(),
                item.user_id().clone(),
            );

            store.push(new_book.clone());
            Ok(new_book)
        }

        async fn update(&self, item: book::Book) -> anyhow::Result<book::Book> {
            let mut store = self.store.lock().unwrap();
            if let Some(index) = store.iter().position(|b| b.id() == item.id()) {
                store[index] = item.clone();
                Ok(item)
            } else {
                Err(anyhow::anyhow!("Book not found"))
            }
        }

        async fn delete(&self, item: book::Book) -> anyhow::Result<()> {
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
        async fn create(&self, item: publisher::Publisher) -> anyhow::Result<publisher::Publisher> {
            self.store.lock().unwrap().push(item.clone());
            Ok(item)
        }
        async fn update(
            &self,
            _item: publisher::Publisher,
        ) -> anyhow::Result<publisher::Publisher> {
            panic!("Not implemented")
        }
        async fn delete(&self, _item: publisher::Publisher) -> anyhow::Result<()> {
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
        async fn create(&self, item: shop::Shop) -> anyhow::Result<shop::Shop> {
            self.store.lock().unwrap().push(item.clone());
            Ok(item)
        }
        async fn update(&self, _item: shop::Shop) -> anyhow::Result<shop::Shop> {
            panic!("Not implemented")
        }
        async fn delete(&self, _item: shop::Shop) -> anyhow::Result<()> {
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
        let created = service
            .create(&ctx, dto)
            .await
            .expect("Failed to create book");
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
        let created = service
            .create(&ctx, dto)
            .await
            .expect("Failed to create book");

        service
            .delete(&ctx, created.pub_id)
            .await
            .expect("Failed to delete");
        assert!(service.get(&ctx, created.pub_id).await.is_err());
    }
}
