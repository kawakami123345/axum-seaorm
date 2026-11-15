use crate::{UserContext, error::UseCaseError};
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
            .map_err(|e| UseCaseError::database("list shops", e))?;

        Ok(shops.into_iter().map(ResponseDto::from).collect())
    }

    pub async fn get(&self, pub_id: uuid::Uuid) -> Result<ResponseDto, UseCaseError> {
        let shop = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|e| UseCaseError::database("find shop", e))?
            .ok_or(UseCaseError::NotFound(format!(
                "Shop with pub_id = {} not found",
                pub_id
            )))?;

        Ok(shop.into())
    }

    pub async fn create(&self, ctx: &UserContext, dto: CreateDto) -> Result<(), UseCaseError> {
        let name = shop::vo::ShopName::new(dto.name)?;

        let shop = shop::Shop::new(uuid::Uuid::now_v7(), name, *ctx.user_id());

        self.repo
            .create(shop)
            .await
            .map_err(|e| UseCaseError::database("create shop", e))
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
            .map_err(|e| UseCaseError::database("find shop", e))?
            .ok_or(UseCaseError::NotFound(format!(
                "Shop with pub_id = {} not found",
                pub_id
            )))?;

        UseCaseError::check_revision(dto.revision, shop.revision())?;
        let name = shop::vo::ShopName::new(dto.name)?;

        shop.update(name, *ctx.user_id())
            .map_err(|e| UseCaseError::DomainRuleViolation(e.to_string()))?;

        let updated = self
            .repo
            .update(shop)
            .await
            .map_err(|e| UseCaseError::database("update shop", e))?;
        UseCaseError::check_write(updated)
    }

    pub async fn delete(
        &self,
        ctx: &UserContext,
        pub_id: uuid::Uuid,
        revision: i64,
    ) -> Result<(), UseCaseError> {
        let shop = self
            .repo
            .find_by_pub_id(pub_id)
            .await
            .map_err(|e| UseCaseError::database("find shop", e))?
            .ok_or(UseCaseError::NotFound(format!(
                "Shop with pub_id = {} not found",
                pub_id
            )))?;

        UseCaseError::check_revision(revision, shop.revision())?;

        let deleted = self
            .repo
            .delete(shop, *ctx.user_id())
            .await
            .map_err(|e| UseCaseError::database("delete shop", e))?;
        UseCaseError::check_write(deleted)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = ShopCreateDto)]
#[serde(deny_unknown_fields)]
pub struct CreateDto {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = ShopUpdateDto)]
#[serde(deny_unknown_fields)]
pub struct UpdateDto {
    #[schema(minimum = 0)]
    pub revision: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = ShopResponseDto)]
pub struct ResponseDto {
    pub pub_id: uuid::Uuid,
    pub revision: i64,
    pub name: String,
}

impl From<shop::Shop> for ResponseDto {
    fn from(s: shop::Shop) -> Self {
        Self {
            pub_id: s.pub_id(),
            revision: s.revision(),
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
        async fn update(&self, item: shop::Shop) -> anyhow::Result<bool> {
            let mut store = self.store.lock().unwrap();
            let Some(stored) = store
                .iter_mut()
                .find(|shop| shop.pub_id() == item.pub_id() && shop.revision() == item.revision())
            else {
                return Ok(false);
            };
            *stored = shop::Shop::reconstruct(
                item.id(),
                item.pub_id(),
                shop::vo::ShopName::new(item.name().into())?,
                item.created_at(),
                item.updated_at(),
                *item.created_by(),
                *item.updated_by(),
                item.revision() + 1,
            );
            Ok(true)
        }
        async fn delete(&self, item: shop::Shop, _deleted_by: Uuid) -> anyhow::Result<bool> {
            let mut store = self.store.lock().unwrap();
            let len = store.len();
            store.retain(|shop| {
                shop.pub_id() != item.pub_id() || shop.revision() != item.revision()
            });
            Ok(store.len() != len)
        }
    }

    #[rstest]
    #[tokio::test]
    async fn stale_client_revisions_cannot_update_or_delete(service: Service) {
        let ctx = UserContext::new(Uuid::new_v4(), vec![]);
        service
            .create(
                &ctx,
                CreateDto {
                    name: "Original".into(),
                },
            )
            .await
            .unwrap();
        let original = service.get_all().await.unwrap().remove(0);
        service
            .update(
                &ctx,
                original.pub_id,
                UpdateDto {
                    revision: original.revision,
                    name: "Winner".into(),
                },
            )
            .await
            .unwrap();
        assert!(matches!(
            service
                .update(
                    &ctx,
                    original.pub_id,
                    UpdateDto {
                        revision: original.revision,
                        name: "Stale".into(),
                    }
                )
                .await,
            Err(UseCaseError::Conflict(_))
        ));
        assert!(matches!(
            service
                .delete(&ctx, original.pub_id, original.revision)
                .await,
            Err(UseCaseError::Conflict(_))
        ));
        let winner = service.get(original.pub_id).await.unwrap();
        assert_eq!(winner.name, "Winner");
        assert_eq!(winner.revision, original.revision + 1);
        for invalid in [-1, i64::MAX] {
            assert!(matches!(
                service
                    .update(
                        &ctx,
                        original.pub_id,
                        UpdateDto {
                            revision: invalid,
                            name: "Invalid".into(),
                        }
                    )
                    .await,
                Err(UseCaseError::DomainRuleViolation(_))
            ));
        }
        service
            .delete(&ctx, original.pub_id, winner.revision)
            .await
            .unwrap();
    }

    #[fixture]
    fn service() -> Service {
        let repo = FakeRepository::new();
        Service::new(Arc::new(repo))
    }

    #[rstest]
    #[tokio::test]
    async fn test_shared_shop_can_be_updated_and_deleted_by_another_user(service: Service) {
        let ctx = UserContext::new(
            Uuid::from_str("11111111-1234-5678-90ab-cdef12345678").unwrap(),
            vec![],
        );
        let dto = CreateDto {
            name: "Test Shop".to_string(),
        };

        service.create(&ctx, dto).await.expect("Failed to create");

        let mut all = service.get_all().await.expect("Failed to get all");
        assert_eq!(all.len(), 1);
        let created = all.remove(0);
        assert_eq!(created.name, "Test Shop");

        let fetched = service.get(created.pub_id).await.expect("Failed to get");
        assert_eq!(fetched.pub_id, created.pub_id);

        let other_user = UserContext::new(Uuid::new_v4(), vec![]);
        service
            .update(
                &other_user,
                created.pub_id,
                UpdateDto {
                    revision: created.revision,
                    name: "Updated Shop".into(),
                },
            )
            .await
            .unwrap();
        assert_eq!(
            service.get(created.pub_id).await.unwrap().name,
            "Updated Shop"
        );
        let stored = service
            .repo
            .find_by_pub_id(created.pub_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.created_by(), ctx.user_id());
        assert_eq!(stored.updated_by(), other_user.user_id());

        service
            .delete(&other_user, created.pub_id, stored.revision())
            .await
            .unwrap();
        assert!(matches!(
            service.get(created.pub_id).await,
            Err(UseCaseError::NotFound(_))
        ));
    }
}
