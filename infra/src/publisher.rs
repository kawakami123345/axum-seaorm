use crate::BeginWithUser;
use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::StringLen;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "publisher")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pub_id: Uuid,
    #[sea_orm(unique, column_type = "String(StringLen::N(32))")]
    pub name: String,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    #[sea_orm(default_value = 0)]
    pub revision: i64,

    #[sea_orm(has_many)]
    pub books: HasMany<super::book::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl ModelEx {
    pub fn to_domain(&self) -> anyhow::Result<publisher::Publisher> {
        let name = publisher::vo::PublisherName::new(self.name.clone())
            .map_err(|e| anyhow::anyhow!("Invalid name in DB: {}", e))?;
        Ok(publisher::Publisher::reconstruct(
            self.id,
            self.pub_id,
            name,
            self.created_at,
            self.updated_at,
            self.created_by,
            self.updated_by,
            self.revision,
        ))
    }
}

pub struct SqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl SqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl publisher::Repository for SqlRepository {
    async fn find_all(&self) -> anyhow::Result<Vec<publisher::Publisher>> {
        Entity::load()
            .order_by(Column::Id, sea_orm::Order::Desc)
            .all(&self.db)
            .await?
            .into_iter()
            .map(|m| m.to_domain())
            .collect()
    }

    async fn find_by_pub_id(
        &self,
        pub_id: uuid::Uuid,
    ) -> anyhow::Result<Option<publisher::Publisher>> {
        Entity::load()
            .filter_by_pub_id(pub_id)
            .one(&self.db)
            .await?
            .map(|m| m.to_domain())
            .transpose()
    }

    async fn create(&self, item: publisher::Publisher) -> anyhow::Result<()> {
        let txn = self.db.begin_with_user(item.updated_by()).await?;

        ActiveModel::builder()
            .set_pub_id(item.pub_id())
            .set_name(item.name().to_string())
            .set_created_at(item.created_at())
            .set_updated_at(item.updated_at())
            .set_created_by(*item.created_by())
            .set_updated_by(*item.updated_by())
            .set_revision(item.revision())
            .insert(&txn)
            .await
            .map_err(|e| crate::classify_name_error(e, "publisher_name_key"))?;

        txn.commit().await?;
        Ok(())
    }

    async fn update(&self, item: publisher::Publisher) -> anyhow::Result<bool> {
        let txn = self.db.begin_with_user(item.updated_by()).await?;

        let changes = ActiveModel::builder()
            .set_name(item.name().to_string())
            .set_updated_at(item.updated_at())
            .set_updated_by(*item.updated_by())
            .set_revision(
                item.revision()
                    .checked_add(1)
                    .ok_or_else(|| anyhow::anyhow!("Revision overflow"))?,
            );
        let result = Entity::update_many()
            .set(changes)
            .filter(Column::Id.eq(item.id()))
            .filter(Column::PubId.eq(item.pub_id()))
            .filter(Column::Revision.eq(item.revision()))
            .exec(&txn)
            .await
            .map_err(|e| crate::classify_name_error(e, "publisher_name_key"))?;

        txn.commit().await?;
        Ok(result.rows_affected == 1)
    }

    async fn delete(
        &self,
        item: publisher::Publisher,
        deleted_by: uuid::Uuid,
    ) -> anyhow::Result<bool> {
        let txn = self.db.begin_with_user(&deleted_by).await?;

        let result = Entity::delete_many()
            .filter(Column::Id.eq(item.id()))
            .filter(Column::PubId.eq(item.pub_id()))
            .filter(Column::Revision.eq(item.revision()))
            .exec(&txn)
            .await?;
        txn.commit().await?;
        Ok(result.rows_affected == 1)
    }
}
