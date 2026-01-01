use crate::error::UseCaseError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

pub struct Service {
    repo: Arc<dyn book::Repository>,
    publisher_repo: Arc<dyn publisher::Repository>,
}

impl Service {
    pub fn new(
        repo: Arc<dyn book::Repository>,
        publisher_repo: Arc<dyn publisher::Repository>,
    ) -> Self {
        Self {
            repo,
            publisher_repo,
        }
    }

    pub async fn get_all(&self) -> Result<Vec<ResponseDto>, UseCaseError> {
        let books = self
            .repo
            .find_all()
            .await
            .map_err(|_| UseCaseError::DatabaseError)?;

        let response_dtos = books.into_iter().map(ResponseDto::from).collect();
        Ok(response_dtos)
    }

    pub async fn get(&self, pub_id: uuid::Uuid) -> Result<ResponseDto, UseCaseError> {
        let book = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|_| UseCaseError::DatabaseError)?
            .ok_or(UseCaseError::NotFound(format!(
                "Book with pub_id = {}",
                pub_id
            )))?;

        Ok(book.into())
    }

    pub async fn create(&self, dto: CreateDto) -> Result<ResponseDto, UseCaseError> {
        let title = book::vo::BookTitle::new(dto.title)?;
        let author = book::vo::BookAuthor::new(dto.author)?;
        let price = book::vo::BookPrice::new(dto.price)?;

        let publisher = self
            .publisher_repo
            .find_by_pub_id(dto.publisher_id)
            .await
            .map_err(|_| UseCaseError::DatabaseError)?
            .ok_or(UseCaseError::NotFound(format!(
                "Publisher with pub_id = {} not found",
                dto.publisher_id
            )))?;

        let book = book::Book::new(
            uuid::Uuid::now_v7(),
            title,
            author,
            publisher,
            price,
            "test player".to_string(),
        );
        self.repo
            .create(book.clone())
            .await
            .map_err(|_| UseCaseError::DatabaseError)?;

        Ok(book.into())
    }

    pub async fn update(
        &self,
        pub_id: uuid::Uuid,
        dto: UpdateDto,
    ) -> Result<ResponseDto, UseCaseError> {
        let title = book::vo::BookTitle::new(dto.title)?;
        let author = book::vo::BookAuthor::new(dto.author)?;
        let price = book::vo::BookPrice::new(dto.price)?;

        let mut book = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|_| UseCaseError::DatabaseError)?
            .ok_or(UseCaseError::NotFound("Book not found".to_string()))?;

        if book.publisher().pub_id() != dto.publisher_id {
            let publisher = self
                .publisher_repo
                .find_by_pub_id(dto.publisher_id)
                .await
                .map_err(|_| UseCaseError::DatabaseError)?
                .ok_or(UseCaseError::NotFound(format!(
                    "Publisher with pub_id = {} not found",
                    dto.publisher_id
                )))?;
            book.update(title, author, publisher, price, "test player".to_string())
                .map_err(|e| UseCaseError::DomainRuleViolation(e.to_string()))?;
        } else {
            book.update(
                title,
                author,
                book.publisher(),
                price,
                "test player".to_string(),
            )
            .map_err(|e| UseCaseError::DomainRuleViolation(e.to_string()))?;
        }

        self.repo
            .update(book.clone())
            .await
            .map_err(|_| UseCaseError::DatabaseError)?;

        Ok(book.into())
    }

    pub async fn delete(&self, pub_id: uuid::Uuid) -> Result<(), UseCaseError> {
        let book = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|_| UseCaseError::DatabaseError)?
            .ok_or(UseCaseError::NotFound(format!(
                "Book with pub_id = {} not found",
                pub_id
            )))?;

        self.repo
            .delete(book)
            .await
            .map_err(|_| UseCaseError::DatabaseError)?;
        Ok(())
    }
    pub async fn switch_status(&self, pub_id: uuid::Uuid) -> Result<ResponseDto, UseCaseError> {
        let mut book = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|_| UseCaseError::DatabaseError)?
            .ok_or(UseCaseError::NotFound(format!(
                "Book with pub_id = {} not found",
                pub_id
            )))?;

        book.switch_status("test player".to_string())
            .map_err(|e| UseCaseError::DomainRuleViolation(e.to_string()))?;

        self.repo
            .update(book.clone())
            .await
            .map_err(|_| UseCaseError::DatabaseError)?;

        Ok(book.into())
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = BookCreateDto)]
pub struct CreateDto {
    pub title: String,
    pub author: String,
    pub publisher_id: uuid::Uuid,
    pub price: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = BookUpdateDto)]
pub struct UpdateDto {
    pub title: String,
    pub author: String,
    pub publisher_id: uuid::Uuid,
    pub price: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = BookResponseDto)]
pub struct ResponseDto {
    pub pub_id: uuid::Uuid,
    pub title: String,
    pub author: String,
    pub publisher: BookPublisherDto,
    #[schema(value_type = String, example = "Unapplied")]
    pub status: String,
    pub price: i32,
}

impl From<book::Book> for ResponseDto {
    fn from(book: book::Book) -> Self {
        Self {
            pub_id: book.pub_id(),
            title: book.title(),
            author: book.author(),
            publisher: BookPublisherDto {
                pub_id: book.publisher().pub_id(),
                name: book.publisher().name(),
            },
            status: book.status(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use rstest::*;
    use std::sync::Mutex;

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

            // We need to reconstruct to set the ID, since fields are private
            let new_book = book::Book::reconstruct(
                new_id,
                item.pub_id(),
                book::vo::BookTitle::new(item.title()).unwrap(),
                book::vo::BookAuthor::new(item.author()).unwrap(),
                item.publisher(),
                match item.status().as_str() {
                    "Applied" => book::vo::BookStatus::Applied,
                    _ => book::vo::BookStatus::Unapplied,
                },
                book::vo::BookPrice::new(item.price()).unwrap(),
                item.created_at(),
                item.updated_at(),
                item.created_by(),
                item.updated_by(),
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

    #[fixture]
    async fn service() -> (Service, Arc<FakePublisherRepository>) {
        let repo = FakeRepository::new();
        let pub_repo = FakePublisherRepository::new();
        let pub_repo_arc = Arc::new(pub_repo);
        (
            Service::new(Arc::new(repo), pub_repo_arc.clone()),
            pub_repo_arc,
        )
    }

    fn create_dummy_publisher(pub_id: uuid::Uuid) -> publisher::Publisher {
        publisher::Publisher::new(
            pub_id,
            publisher::vo::PublisherName::new("Test Publisher".to_string()).unwrap(),
            "test player".to_string(),
        )
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_and_get(#[future] service: (Service, Arc<FakePublisherRepository>)) {
        let (service, pub_repo) = service.await;
        let pub_id = uuid::Uuid::new_v4();

        // Setup publisher
        pub_repo.add(create_dummy_publisher(pub_id));

        let dto = CreateDto {
            title: "Test Book".to_string(),
            author: "Author 1".to_string(),
            publisher_id: pub_id,
            price: 1000,
        };

        // Create
        let created = service.create(dto).await.expect("Failed to create book");
        assert_eq!(created.title, "Test Book");
        assert_eq!(created.author, "Author 1");
        assert_eq!(created.publisher.pub_id, pub_id);
        assert_eq!(created.publisher.name, "Test Publisher");
        assert_eq!(created.price, 1000);
        assert!(!created.pub_id.is_nil());

        // Get
        let fetched = service
            .get(created.pub_id)
            .await
            .expect("Failed to get book");
        assert_eq!(fetched.pub_id, created.pub_id);
        assert_eq!(fetched.title, "Test Book");
        assert_eq!(fetched.publisher.name, "Test Publisher");
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_all(#[future] service: (Service, Arc<FakePublisherRepository>)) {
        let (service, pub_repo) = service.await;
        let pub_id_1 = uuid::Uuid::new_v4();
        let pub_id_2 = uuid::Uuid::new_v4();

        pub_repo.add(create_dummy_publisher(pub_id_1));
        pub_repo.add(create_dummy_publisher(pub_id_2));

        let dto1 = CreateDto {
            title: "Book 1".to_string(),
            author: "Author 1".to_string(),
            publisher_id: pub_id_1,
            price: 100,
        };
        let dto2 = CreateDto {
            title: "Book 2".to_string(),
            author: "Author 2".to_string(),
            publisher_id: pub_id_2,
            price: 200,
        };

        service.create(dto1).await.expect("Failed to create book 1");
        service.create(dto2).await.expect("Failed to create book 2");

        let all = service.get_all().await.expect("Failed to get all");
        assert_eq!(all.len(), 2);
    }

    #[rstest]
    #[tokio::test]
    async fn test_delete(#[future] service: (Service, Arc<FakePublisherRepository>)) {
        let (service, pub_repo) = service.await;
        let pub_id = uuid::Uuid::new_v4();
        pub_repo.add(create_dummy_publisher(pub_id));

        let dto = CreateDto {
            title: "Book To Delete".to_string(),
            author: "Author".to_string(),
            publisher_id: pub_id,
            price: 100,
        };
        let created = service.create(dto).await.expect("Failed to create book");

        service
            .delete(created.pub_id)
            .await
            .expect("Failed to delete book");

        let fetched = service.get(created.pub_id).await;
        match fetched {
            Err(UseCaseError::NotFound(_)) => assert!(true),
            _ => assert!(false, "Book should be deleted"),
        }
    }
}
