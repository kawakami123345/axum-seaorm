use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::error::ApiError;
pub struct Service {
    repo: Arc<dyn book::Repository>,
}

impl Service {
    pub fn new(repo: Arc<dyn book::Repository>) -> Self {
        Self { repo }
    }

    pub async fn get_all(&self) -> Result<Vec<ResponseDto>, ApiError> {
        let books = self
            .repo
            .find_all()
            .await
            .map_err(|_| ApiError::DatabaseError)?;
        Ok(books.into_iter().map(ResponseDto::from).collect())
    }

    pub async fn get(&self, id: i32) -> Result<ResponseDto, ApiError> {
        let book = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|_| ApiError::DatabaseError)?
            .ok_or(ApiError::NotFound(format!("Book Id = {}", id)))?;
        Ok(ResponseDto::from(book))
    }

    pub async fn create(&self, dto: CreateDto) -> Result<ResponseDto, ApiError> {
        let book = book::Book {
            id: 0,
            title: dto.title,
            author: dto.author,
            publisher_id: dto.publisher_id,
        };
        let result = self
            .repo
            .create(book)
            .await
            .map_err(|_| ApiError::DatabaseError)?;
        Ok(ResponseDto::from(result))
    }

    pub async fn update(&self, dto: UpdateDto) -> Result<ResponseDto, ApiError> {
        let book = book::Book {
            id: dto.id,
            title: dto.title,
            author: dto.author,
            publisher_id: dto.publisher_id,
        };
        let result = self
            .repo
            .update(book)
            .await
            .map_err(|_| ApiError::DatabaseError)?;
        Ok(ResponseDto::from(result))
    }

    pub async fn delete(&self, id: i32) -> Result<(), ApiError> {
        self.repo
            .delete(id)
            .await
            .map_err(|_| ApiError::DatabaseError)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = BookCreateDto)]
pub struct CreateDto {
    pub title: String,
    pub author: String,
    pub publisher_id: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = BookUpdateDto)]
pub struct UpdateDto {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub publisher_id: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = BookResponseDto)]
pub struct ResponseDto {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub publisher_id: i32,
}

impl From<book::Book> for ResponseDto {
    fn from(book: book::Book) -> Self {
        Self {
            id: book.id,
            title: book.title,
            author: book.author,
            publisher_id: book.publisher_id,
        }
    }
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

        async fn find_by_id(&self, id: i32) -> anyhow::Result<Option<book::Book>> {
            let store = self.store.lock().unwrap();
            Ok(store.iter().find(|b| b.id == id).cloned())
        }

        async fn create(&self, mut item: book::Book) -> anyhow::Result<book::Book> {
            let mut store = self.store.lock().unwrap();
            let new_id = store.iter().map(|b| b.id).max().unwrap_or(0) + 1;
            item.id = new_id;
            store.push(item.clone());
            Ok(item)
        }

        async fn update(&self, item: book::Book) -> anyhow::Result<book::Book> {
            let mut store = self.store.lock().unwrap();
            if let Some(index) = store.iter().position(|b| b.id == item.id) {
                store[index] = item.clone();
                Ok(item)
            } else {
                Err(anyhow::anyhow!("Book not found"))
            }
        }

        async fn delete(&self, id: i32) -> anyhow::Result<()> {
            let mut store = self.store.lock().unwrap();
            store.retain(|b| b.id != id);
            Ok(())
        }
    }

    #[fixture]
    async fn service() -> Service {
        let repo = FakeRepository::new();
        Service::new(Arc::new(repo))
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_and_get(#[future] service: Service) {
        let service = service.await;
        let dto = CreateDto {
            title: "Test Book".to_string(),
            author: "Author 1".to_string(),
            publisher_id: 1,
        };

        // Create
        let created = service.create(dto).await.expect("Failed to create book");
        assert_eq!(created.title, "Test Book");
        assert_eq!(created.author, "Author 1");
        assert_eq!(created.publisher_id, 1);
        assert!(created.id > 0);

        // Get
        let fetched = service.get(created.id).await.expect("Failed to get book");
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.title, "Test Book");
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_all(#[future] service: Service) {
        let service = service.await;
        let dto1 = CreateDto {
            title: "Book 1".to_string(),
            author: "Author 1".to_string(),
            publisher_id: 1,
        };
        let dto2 = CreateDto {
            title: "Book 2".to_string(),
            author: "Author 2".to_string(),
            publisher_id: 2,
        };

        service.create(dto1).await.expect("Failed to create book 1");
        service.create(dto2).await.expect("Failed to create book 2");

        let all = service.get_all().await.expect("Failed to get all");
        assert_eq!(all.len(), 2);
    }

    #[rstest]
    #[tokio::test]
    async fn test_update(#[future] service: Service) {
        let service = service.await;
        let dto = CreateDto {
            title: "Original Title".to_string(),
            author: "Original Author".to_string(),
            publisher_id: 1,
        };
        let created = service.create(dto).await.expect("Failed to create");

        let update_dto = UpdateDto {
            id: created.id,
            title: "Updated Title".to_string(),
            author: "Original Author".to_string(),
            publisher_id: 1,
        };

        let updated = service.update(update_dto).await.expect("Failed to update");
        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.author, "Original Author");

        let fetched = service.get(created.id).await.expect("Failed to get");
        assert_eq!(fetched.title, "Updated Title");
    }

    #[rstest]
    #[tokio::test]
    async fn test_delete(#[future] service: Service) {
        let service = service.await;
        let dto = CreateDto {
            title: "To Delete".to_string(),
            author: "Author".to_string(),
            publisher_id: 1,
        };
        let created = service.create(dto).await.expect("Failed to create");

        service.delete(created.id).await.expect("Failed to delete");

        let result = service.get(created.id).await;
        assert!(result.is_err());
        match result {
            Err(ApiError::NotFound(_)) => (),
            _ => panic!("Expected NotFound error"),
        }
    }
}
