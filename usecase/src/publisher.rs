use crate::{UserContext, error::UseCaseError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

pub struct Service {
    repo: Arc<dyn publisher::Repository>,
}

impl Service {
    pub fn new(repo: Arc<dyn publisher::Repository>) -> Self {
        Self { repo }
    }

    pub async fn get_all(&self) -> Result<Vec<ResponseDto>, UseCaseError> {
        let publishers = self.repo.find_all().await.map_err(|e| {
            eprintln!("Database error in list publishers: {:?}", e);
            UseCaseError::DatabaseError
        })?;

        Ok(publishers.into_iter().map(ResponseDto::from).collect())
    }

    pub async fn get(&self, pub_id: Uuid) -> Result<ResponseDto, UseCaseError> {
        let publisher = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|e| {
                eprintln!("Database error in create book (find publisher): {:?}", e);
                UseCaseError::DatabaseError
            })?
            .ok_or(UseCaseError::NotFound(format!(
                "Publisher not found with pub_id = {}",
                pub_id
            )))?;

        Ok(publisher.into())
    }

    pub async fn create(&self, ctx: &UserContext, dto: CreateDto) -> Result<(), UseCaseError> {
        let name = publisher::vo::PublisherName::new(dto.name)?;
        let publisher = publisher::Publisher::new(Uuid::now_v7(), name, *ctx.user_id());

        self.repo.create(publisher).await.map_err(|e| {
            eprintln!("Database error in create book (find publisher): {:?}", e);
            UseCaseError::DatabaseError
        })?;
        Ok(())
    }

    pub async fn update(
        &self,
        ctx: &UserContext,
        pub_id: Uuid,
        dto: UpdateDto,
    ) -> Result<(), UseCaseError> {
        let name = publisher::vo::PublisherName::new(dto.name)?;
        let mut publisher = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|e| {
                eprintln!("Database error in create book (find publisher): {:?}", e);
                UseCaseError::DatabaseError
            })?
            .ok_or(UseCaseError::NotFound(format!(
                "Publisher not found with pub_id = {}",
                pub_id
            )))?;

        publisher
            .update(name, *ctx.user_id())
            .map_err(|e| UseCaseError::DomainRuleViolation(e.to_string()))?;

        self.repo.update(publisher).await.map_err(|e| {
            eprintln!("Database error in create book (find publisher): {:?}", e);
            UseCaseError::DatabaseError
        })?;
        Ok(())
    }

    pub async fn delete(&self, ctx: &UserContext, pub_id: Uuid) -> Result<(), UseCaseError> {
        let publisher = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|e| {
                eprintln!("Database error in create book (find publisher): {:?}", e);
                UseCaseError::DatabaseError
            })?
            .ok_or(UseCaseError::NotFound(format!(
                "Publisher with pub_id = {} not found",
                pub_id
            )))?;

        self.repo
            .delete(publisher, *ctx.user_id())
            .await
            .map_err(|e| {
                eprintln!("Database error in create book (find publisher): {:?}", e);
                UseCaseError::DatabaseError
            })?;
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
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = PublisherResponseDto)]
pub struct ResponseDto {
    pub pub_id: Uuid,
    pub name: String,
}

impl From<publisher::Publisher> for ResponseDto {
    fn from(publisher: publisher::Publisher) -> Self {
        Self {
            pub_id: publisher.pub_id(),
            name: publisher.name().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use rstest::*;
    use std::{str::FromStr, sync::Mutex};

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

        async fn find_by_pub_id(
            &self,
            pub_id: Uuid,
        ) -> anyhow::Result<Option<publisher::Publisher>> {
            let store = self.store.lock().unwrap();
            Ok(store.iter().find(|p| p.pub_id() == pub_id).cloned())
        }

        async fn create(&self, item: publisher::Publisher) -> anyhow::Result<()> {
            let mut store = self.store.lock().unwrap();
            let new_id = store.iter().map(|p| p.id()).max().unwrap_or(0) + 1;

            let new_publisher = publisher::Publisher::reconstruct(
                new_id,
                item.pub_id(),
                publisher::vo::PublisherName::new(item.name().to_string()).unwrap(),
                item.created_at(),
                item.updated_at(),
                *item.created_by(),
                *item.updated_by(),
            );

            store.push(new_publisher);
            Ok(())
        }

        async fn update(&self, item: publisher::Publisher) -> anyhow::Result<()> {
            let mut store = self.store.lock().unwrap();
            if let Some(index) = store.iter().position(|p| p.id() == item.id()) {
                store[index] = item;
                Ok(())
            } else {
                Err(anyhow::anyhow!("Publisher not found"))
            }
        }

        async fn delete(
            &self,
            item: publisher::Publisher,
            _deleted_by: Uuid,
        ) -> anyhow::Result<()> {
            let mut store = self.store.lock().unwrap();
            store.retain(|p| p.pub_id() != item.pub_id());
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
        let ctx = UserContext::new(
            Uuid::from_str("11111111-1234-5678-90ab-cdef12345678").unwrap(),
            vec![],
        );
        let dto = CreateDto {
            name: "Test Publisher".to_string(),
        };

        service.create(&ctx, dto).await.expect("Failed to create");

        let mut all = service.get_all().await.expect("Failed to get all");
        assert_eq!(all.len(), 1);
        let created = all.remove(0);
        assert_eq!(created.name, "Test Publisher");

        let fetched = service.get(created.pub_id).await.expect("Failed to get");
        assert_eq!(fetched.name, "Test Publisher");
        assert_eq!(fetched.pub_id, created.pub_id);
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_all(#[future] service: Service) {
        let service = service.await;
        let ctx = UserContext::new(
            Uuid::from_str("11111111-1234-5678-90ab-cdef12345678").unwrap(),
            vec![],
        );
        let dto1 = CreateDto {
            name: "Publisher 1".to_string(),
        };
        let dto2 = CreateDto {
            name: "Publisher 2".to_string(),
        };

        service
            .create(&ctx, dto1)
            .await
            .expect("Failed to create 1");
        service
            .create(&ctx, dto2)
            .await
            .expect("Failed to create 2");

        let all = service.get_all().await.expect("Failed to get all");
        assert_eq!(all.len(), 2);
    }

    #[rstest]
    #[tokio::test]
    async fn test_other_user_can_update_shared_publisher(#[future] service: Service) {
        let service = service.await;
        let ctx = UserContext::new(
            Uuid::from_str("11111111-1234-5678-90ab-cdef12345678").unwrap(),
            vec![],
        );
        let dto = CreateDto {
            name: "Original Name".to_string(),
        };
        service.create(&ctx, dto).await.expect("Failed to create");

        let mut all = service.get_all().await.expect("Failed to get all");
        assert_eq!(all.len(), 1);
        let created = all.remove(0);

        let update_dto = UpdateDto {
            name: "Updated Name".to_string(),
        };

        let other_user = UserContext::new(Uuid::new_v4(), vec![]);
        service
            .update(&other_user, created.pub_id, update_dto)
            .await
            .expect("Failed to update");

        let fetched = service.get(created.pub_id).await.expect("Failed to get");
        assert_eq!(fetched.name, "Updated Name");
        let stored = service
            .repo
            .find_by_pub_id(created.pub_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.created_by(), ctx.user_id());
        assert_eq!(stored.updated_by(), other_user.user_id());
    }

    #[rstest]
    #[tokio::test]
    async fn test_other_user_can_delete_shared_publisher(#[future] service: Service) {
        let service = service.await;
        let ctx = UserContext::new(
            Uuid::from_str("11111111-1234-5678-90ab-cdef12345678").unwrap(),
            vec![],
        );
        let dto = CreateDto {
            name: "To Delete".to_string(),
        };
        service.create(&ctx, dto).await.expect("Failed to create");

        let mut all = service.get_all().await.expect("Failed to get all");
        assert_eq!(all.len(), 1);
        let created = all.remove(0);

        let other_user = UserContext::new(Uuid::new_v4(), vec![]);
        service
            .delete(&other_user, created.pub_id)
            .await
            .expect("Failed to delete");

        let result = service.get(created.pub_id).await;
        assert!(result.is_err());
        match result {
            Err(UseCaseError::NotFound(_)) => (),
            _ => panic!("Expected NotFound error"),
        }
    }
}
