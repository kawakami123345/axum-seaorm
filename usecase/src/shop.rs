use crate::{UserContext, cedar, error::UseCaseError};
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};
use utoipa::ToSchema;
use uuid::Uuid;

pub struct Service {
    repo: Arc<dyn shop::Repository>,
}

impl Service {
    pub fn new(repo: Arc<dyn shop::Repository>) -> Self {
        Self { repo }
    }

    pub async fn get_all(&self, ctx: &UserContext) -> Result<Vec<ResponseDto>, UseCaseError> {
        let partial = cedar::partial_authorize_shop_list(ctx)?;
        if matches!(partial, cedar::PartialDecision::Deny) {
            return Ok(Vec::new());
        }

        let shops = self.repo.find_all().await.map_err(|e| {
            eprintln!("Database error in create book (find publisher): {:?}", e);
            UseCaseError::DatabaseError
        })?;
        let shops = match partial {
            cedar::PartialDecision::Allow => shops,
            cedar::PartialDecision::Residual(residuals) => {
                cedar::authorize_shop_list_batch(ctx, &residuals, &shops)?
            }
            cedar::PartialDecision::Deny => Vec::new(),
        };

        let dtos = shops.into_iter().map(ResponseDto::from).collect();
        Ok(dtos)
    }

    pub async fn get(
        &self,
        ctx: &UserContext,
        pub_id: uuid::Uuid,
    ) -> Result<ResponseDto, UseCaseError> {
        let shop = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|e| {
                eprintln!("Database error in create book (find publisher): {:?}", e);
                UseCaseError::DatabaseError
            })?
            .ok_or(UseCaseError::NotFound(format!(
                "Shop with pub_id = {} not found",
                pub_id
            )))?;

        cedar::authorize_shop_get(ctx, &shop)?;

        Ok(shop.into())
    }

    pub async fn create(&self, ctx: &UserContext, dto: CreateDto) -> Result<(), UseCaseError> {
        let name = shop::vo::ShopName::new(dto.name)?;

        let shop = shop::Shop::new(uuid::Uuid::now_v7(), name, *ctx.user_id());

        cedar::authorize_shop_create(ctx, &shop)?;

        self.repo.create(shop).await.map_err(|e| {
            eprintln!("Database error in create book (find publisher): {:?}", e);
            UseCaseError::DatabaseError
        })?;

        Ok(())
    }
    pub async fn update(
        &self,
        ctx: &UserContext,
        pub_id: uuid::Uuid,
        dto: UpdateDto,
    ) -> Result<(), UseCaseError> {
        let mut shop = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|e| {
                eprintln!("Database error in create book (find publisher): {:?}", e);
                UseCaseError::DatabaseError
            })?
            .ok_or(UseCaseError::NotFound(format!(
                "Shop with pub_id = {} not found",
                pub_id
            )))?;

        cedar::authorize_shop_update(ctx, &shop)?;

        let name = shop::vo::ShopName::new(dto.name)?;

        shop.update(name, *ctx.user_id())
            .map_err(|e| UseCaseError::DomainRuleViolation(e.to_string()))?;

        self.repo.update(shop).await.map_err(|e| {
            eprintln!("Database error in create book (find publisher): {:?}", e);
            UseCaseError::DatabaseError
        })?;

        Ok(())
    }

    pub async fn delete(&self, ctx: &UserContext, pub_id: uuid::Uuid) -> Result<(), UseCaseError> {
        let shop = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|e| {
                eprintln!("Database error in create book (find publisher): {:?}", e);
                UseCaseError::DatabaseError
            })?
            .ok_or(UseCaseError::NotFound(format!(
                "Shop with pub_id = {} not found",
                pub_id
            )))?;

        cedar::authorize_shop_delete(ctx, &shop)?;

        self.repo
            .delete(
                shop,
                Uuid::from_str("11111111-1234-5678-90ab-cdef12345678").unwrap(),
            )
            .await
            .map_err(|e| {
                eprintln!("Database error in create book (find publisher): {:?}", e);
                UseCaseError::DatabaseError
            })?;

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
            name: s.name().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use rstest::*;
    use std::{str::FromStr, sync::Mutex};
    use uuid::Uuid;

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
    fn service() -> Service {
        let repo = FakeRepository::new();
        Service::new(Arc::new(repo))
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_and_get(service: Service) {
        let ctx = UserContext::new(
            Uuid::from_str("11111111-1234-5678-90ab-cdef12345678").unwrap(),
            vec![],
        );
        let dto = CreateDto {
            name: "Test Shop".to_string(),
        };

        service.create(&ctx, dto).await.expect("Failed to create");

        let mut all = service.get_all(&ctx).await.expect("Failed to get all");
        assert_eq!(all.len(), 1);
        let created = all.remove(0);
        assert_eq!(created.name, "Test Shop");

        let fetched = service
            .get(&ctx, created.pub_id)
            .await
            .expect("Failed to get");
        assert_eq!(fetched.pub_id, created.pub_id);
    }
}
