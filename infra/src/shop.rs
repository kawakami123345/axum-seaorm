use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::StringLen;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "shop")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pub_id: uuid::Uuid,
    #[sea_orm(unique, column_type = "String(StringLen::N(32))")]
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[sea_orm(column_type = "String(StringLen::N(32))")]
    pub created_by: String,
    #[sea_orm(column_type = "String(StringLen::N(32))")]
    pub updated_by: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::book::Entity")]
    Book,
}

impl Related<super::book::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Book.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub struct SqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl SqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn to_domain(model: Model) -> anyhow::Result<shop::Shop> {
        let name = shop::vo::ShopName::new(model.name)
            .map_err(|e| anyhow::anyhow!("Invalid name in DB: {}", e))?;
        Ok(shop::Shop::reconstruct(
            model.id,
            model.pub_id,
            name,
            model.created_at,
            model.updated_at,
            model.created_by,
            model.updated_by,
        ))
    }
}

#[async_trait]
impl shop::Repository for SqlRepository {
    async fn find_all(&self) -> anyhow::Result<Vec<shop::Shop>> {
        let shops = Entity::find().all(&self.db).await?;
        shops.into_iter().map(Self::to_domain).collect()
    }

    async fn find_by_pub_id(&self, pub_id: uuid::Uuid) -> anyhow::Result<Option<shop::Shop>> {
        let shop = Entity::find()
            .filter(Column::PubId.eq(pub_id))
            .one(&self.db)
            .await?;
        match shop {
            Some(s) => Ok(Some(Self::to_domain(s)?)),
            None => Ok(None),
        }
    }

    async fn create(&self, item: shop::Shop) -> anyhow::Result<shop::Shop> {
        let active_model = ActiveModel {
            pub_id: Set(item.pub_id()),
            name: Set(item.name()),
            created_at: Set(item.created_at()),
            updated_at: Set(item.updated_at()),
            created_by: Set(item.created_by()),
            updated_by: Set(item.updated_by()),
            ..Default::default()
        };

        let result = active_model.insert(&self.db).await?;
        Ok(Self::to_domain(result)?)
    }

    async fn update(&self, item: shop::Shop) -> anyhow::Result<shop::Shop> {
        let active_model = ActiveModel {
            id: Set(item.id()),
            pub_id: Set(item.pub_id()),
            name: Set(item.name()),
            created_at: Set(item.created_at()),
            updated_at: Set(item.updated_at()),
            created_by: Set(item.created_by()),
            updated_by: Set(item.updated_by()),
        };

        let result = active_model.update(&self.db).await?;
        Ok(Self::to_domain(result)?)
    }

    async fn delete(&self, item: shop::Shop) -> anyhow::Result<()> {
        Entity::delete_by_id(item.id()).exec(&self.db).await?;
        Ok(())
    }
}
