use crate::error::ApiError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

pub struct Service {
    repo: Arc<dyn publisher::Repository>,
}

impl Service {
    pub fn new(repo: Arc<dyn publisher::Repository>) -> Self {
        Self { repo }
    }

    pub async fn get_all(&self) -> Result<Vec<ResponseDto>, ApiError> {
        let publishers = self
            .repo
            .find_all()
            .await
            .map_err(|_| ApiError::DatabaseError)?;
        Ok(publishers.into_iter().map(ResponseDto::from).collect())
    }

    pub async fn get(&self, id: i32) -> Result<ResponseDto, ApiError> {
        let publisher = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|_| ApiError::DatabaseError)?
            .ok_or(ApiError::NotFound(format!("Publisher Id = {}", id)))?;
        Ok(ResponseDto::from(publisher))
    }

    pub async fn create(&self, dto: CreateDto) -> Result<ResponseDto, ApiError> {
        let publisher = publisher::Publisher {
            id: 0,
            name: dto.name,
        };
        let result = self
            .repo
            .create(publisher)
            .await
            .map_err(|_| ApiError::DatabaseError)?;
        Ok(ResponseDto::from(result))
    }

    pub async fn update(&self, dto: UpdateDto) -> Result<ResponseDto, ApiError> {
        let publisher = publisher::Publisher {
            id: dto.id,
            name: dto.name,
        };
        let result = self
            .repo
            .update(publisher)
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
#[schema(as = PublisherCreateDto)]
pub struct CreateDto {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = PublisherUpdateDto)]
pub struct UpdateDto {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = PublisherResponseDto)]
pub struct ResponseDto {
    pub id: i32,
    pub name: String,
}

impl From<publisher::Publisher> for ResponseDto {
    fn from(publisher: publisher::Publisher) -> Self {
        Self {
            id: publisher.id,
            name: publisher.name,
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
        store: Arc<Mutex<Vec<publisher::Publisher>>>,
    }

    impl FakeRepository {
        fn new() -> Self {
            Self {
                store: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl publisher::Repository for FakeRepository {
        async fn find_all(&self) -> anyhow::Result<Vec<publisher::Publisher>> {
            let store = self.store.lock().unwrap();
            Ok(store.clone())
        }

        async fn find_by_id(&self, id: i32) -> anyhow::Result<Option<publisher::Publisher>> {
            let store = self.store.lock().unwrap();
            Ok(store.iter().find(|p| p.id == id).cloned())
        }

        async fn create(
            &self,
            mut item: publisher::Publisher,
        ) -> anyhow::Result<publisher::Publisher> {
            let mut store = self.store.lock().unwrap();
            let new_id = store.iter().map(|p| p.id).max().unwrap_or(0) + 1;
            item.id = new_id;
            store.push(item.clone());
            Ok(item)
        }

        async fn update(&self, item: publisher::Publisher) -> anyhow::Result<publisher::Publisher> {
            let mut store = self.store.lock().unwrap();
            if let Some(index) = store.iter().position(|p| p.id == item.id) {
                store[index] = item.clone();
                Ok(item)
            } else {
                Err(anyhow::anyhow!("Publisher not found"))
            }
        }

        async fn delete(&self, id: i32) -> anyhow::Result<()> {
            let mut store = self.store.lock().unwrap();
            store.retain(|p| p.id != id);
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
            name: "Test Publisher".to_string(),
        };

        let created = service.create(dto).await.expect("Failed to create");
        assert_eq!(created.name, "Test Publisher");
        assert!(created.id > 0);

        let fetched = service.get(created.id).await.expect("Failed to get");
        assert_eq!(fetched.name, "Test Publisher");
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_all(#[future] service: Service) {
        let service = service.await;
        let dto1 = CreateDto {
            name: "Publisher 1".to_string(),
        };
        let dto2 = CreateDto {
            name: "Publisher 2".to_string(),
        };

        service.create(dto1).await.expect("Failed to create 1");
        service.create(dto2).await.expect("Failed to create 2");

        let all = service.get_all().await.expect("Failed to get all");
        assert_eq!(all.len(), 2);
    }

    #[rstest]
    #[tokio::test]
    async fn test_update(#[future] service: Service) {
        let service = service.await;
        let dto = CreateDto {
            name: "Original Name".to_string(),
        };
        let created = service.create(dto).await.expect("Failed to create");

        let update_dto = UpdateDto {
            id: created.id,
            name: "Updated Name".to_string(),
        };

        let updated = service.update(update_dto).await.expect("Failed to update");
        assert_eq!(updated.name, "Updated Name");

        let fetched = service.get(created.id).await.expect("Failed to get");
        assert_eq!(fetched.name, "Updated Name");
    }

    #[rstest]
    #[tokio::test]
    async fn test_delete(#[future] service: Service) {
        let service = service.await;
        let dto = CreateDto {
            name: "To Delete".to_string(),
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
