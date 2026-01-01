use crate::error::UseCaseError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

pub struct Service {
    repo: Arc<dyn shop::Repository>,
}

impl Service {
    pub fn new(repo: Arc<dyn shop::Repository>) -> Self {
        Self { repo }
    }

    pub async fn get_all(&self) -> Result<Vec<ResponseDto>, UseCaseError> {
        let shops = self
            .repo
            .find_all()
            .await
            .map_err(|_| UseCaseError::DatabaseError)?;
        let dtos = shops.into_iter().map(ResponseDto::from).collect();
        Ok(dtos)
    }

    pub async fn get(&self, pub_id: uuid::Uuid) -> Result<ResponseDto, UseCaseError> {
        let shop = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|_| UseCaseError::DatabaseError)?
            .ok_or(UseCaseError::NotFound(format!(
                "Shop with pub_id = {} not found",
                pub_id
            )))?;
        Ok(shop.into())
    }

    pub async fn create(&self, dto: CreateDto) -> Result<ResponseDto, UseCaseError> {
        let name = shop::vo::ShopName::new(dto.name)?;

        let shop = shop::Shop::new(uuid::Uuid::now_v7(), name, "test player".to_string());

        let created = self
            .repo
            .create(shop)
            .await
            .map_err(|_| UseCaseError::DatabaseError)?;

        Ok(created.into())
    }
    pub async fn update(
        &self,
        pub_id: uuid::Uuid,
        dto: UpdateDto,
    ) -> Result<ResponseDto, UseCaseError> {
        let mut shop = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|_| UseCaseError::DatabaseError)?
            .ok_or(UseCaseError::NotFound(format!(
                "Shop with pub_id = {} not found",
                pub_id
            )))?;

        let name = shop::vo::ShopName::new(dto.name)?;

        shop.update(name, "test player".to_string())
            .map_err(|e| UseCaseError::DomainRuleViolation(e.to_string()))?;

        let updated = self
            .repo
            .update(shop)
            .await
            .map_err(|_| UseCaseError::DatabaseError)?;

        Ok(updated.into())
    }

    pub async fn delete(&self, pub_id: uuid::Uuid) -> Result<(), UseCaseError> {
        let shop = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|_| UseCaseError::DatabaseError)?
            .ok_or(UseCaseError::NotFound(format!(
                "Shop with pub_id = {} not found",
                pub_id
            )))?;

        self.repo
            .delete(shop)
            .await
            .map_err(|_| UseCaseError::DatabaseError)?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = ShopCreateDto)]
pub struct CreateDto {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = ShopUpdateDto)]
pub struct UpdateDto {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = ShopResponseDto)]
pub struct ResponseDto {
    pub pub_id: uuid::Uuid,
    pub name: String,
}

impl From<shop::Shop> for ResponseDto {
    fn from(s: shop::Shop) -> Self {
        Self {
            pub_id: s.pub_id(),
            name: s.name(),
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
        store: Arc<Mutex<Vec<shop::Shop>>>,
    }

    impl FakeRepository {
        fn new() -> Self {
            Self {
                store: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl shop::Repository for FakeRepository {
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
    fn service() -> Service {
        let repo = FakeRepository::new();
        Service::new(Arc::new(repo))
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_and_get(service: Service) {
        let dto = CreateDto {
            name: "Test Shop".to_string(),
        };

        let created = service.create(dto).await.expect("Failed to create");
        assert_eq!(created.name, "Test Shop");

        let fetched = service.get(created.pub_id).await.expect("Failed to get");
        assert_eq!(fetched.pub_id, created.pub_id);
    }
}
