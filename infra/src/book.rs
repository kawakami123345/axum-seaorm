use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::StringLen;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "book")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pub_id: uuid::Uuid,
    #[sea_orm(column_type = "String(StringLen::N(32))")]
    pub title: String,
    #[sea_orm(column_type = "String(StringLen::N(32))")]
    pub author: String,
    pub publisher_id: i32,
    pub shop_id: Option<i32>,
    pub applied_at: Option<chrono::DateTime<chrono::Utc>>,
    #[sea_orm(column_type = "String(StringLen::N(32))")]
    pub format: String,
    pub price: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[sea_orm(column_type = "String(StringLen::N(32))")]
    pub created_by: String,
    #[sea_orm(column_type = "String(StringLen::N(32))")]
    pub updated_by: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::publisher::Entity",
        from = "Column::PublisherId",
        to = "super::publisher::Column::Id"
    )]
    Publisher,
    #[sea_orm(
        belongs_to = "super::shop::Entity",
        from = "Column::ShopId",
        to = "super::shop::Column::Id"
    )]
    Shop,
}

impl Related<super::publisher::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Publisher.def()
    }
}

impl Related<super::shop::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Shop.def()
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

    fn to_domain(
        model: Model,
        publisher: Option<super::publisher::Model>,
        shop: Option<super::shop::Model>,
    ) -> anyhow::Result<book::Book> {
        let title = book::vo::BookTitle::new(model.title)
            .map_err(|e| anyhow::anyhow!("Invalid title in DB: {}", e))?;
        let author = book::vo::BookAuthor::new(model.author)
            .map_err(|e| anyhow::anyhow!("Invalid author in DB: {}", e))?;
        let price = book::vo::BookPrice::new(model.price)
            .map_err(|e| anyhow::anyhow!("Invalid price in DB: {}", e))?;
        let format = match model.format.as_str() {
            "Real" => book::vo::BookFormat::Real,
            "EBook" => book::vo::BookFormat::EBook,
            _ => return Err(anyhow::anyhow!("Invalid format in DB: {}", model.format)),
        };

        let publisher_entity = if let Some(p) = publisher {
            publisher::Publisher::reconstruct(
                p.id,
                p.pub_id,
                publisher::vo::PublisherName::new(p.name)
                    .map_err(|e| anyhow::anyhow!("Invalid publisher name in DB: {}", e))?,
                p.created_at,
                p.updated_at,
                p.created_by,
                p.updated_by,
            )
        } else {
            return Err(anyhow::anyhow!("Publisher not found for book {}", model.id));
        };

        let shop_entity = if let Some(s) = shop {
            Some(shop::Shop::reconstruct(
                s.id,
                s.pub_id,
                shop::vo::ShopName::new(s.name)
                    .map_err(|e| anyhow::anyhow!("Invalid shop name in DB: {}", e))?,
                s.created_at,
                s.updated_at,
                s.created_by,
                s.updated_by,
            ))
        } else {
            None
        };

        Ok(book::Book::reconstruct(
            model.id,
            model.pub_id,
            title,
            author,
            publisher_entity,
            shop_entity,
            model.applied_at,
            format,
            price,
            model.created_at,
            model.updated_at,
            model.created_by,
            model.updated_by,
        ))
    }
}

#[async_trait]
impl book::Repository for SqlRepository {
    async fn find_all(&self) -> anyhow::Result<Vec<book::Book>> {
        let books_with_publishers = Entity::find()
            .find_also_related(super::publisher::Entity)
            .all(&self.db)
            .await?;

        let mut books = Vec::new();
        for (b, p) in books_with_publishers {
            let shop = if let Some(shop_id) = b.shop_id {
                super::shop::Entity::find_by_id(shop_id)
                    .one(&self.db)
                    .await?
            } else {
                None
            };
            books.push(Self::to_domain(b, p, shop)?);
        }

        Ok(books)
    }

    async fn find_by_pub_id(&self, pub_id: uuid::Uuid) -> anyhow::Result<Option<book::Book>> {
        let result = Entity::find()
            .filter(Column::PubId.eq(pub_id))
            .find_also_related(super::publisher::Entity)
            .one(&self.db)
            .await?;

        match result {
            Some((b, p)) => {
                let shop = if let Some(shop_id) = b.shop_id {
                    super::shop::Entity::find_by_id(shop_id)
                        .one(&self.db)
                        .await?
                } else {
                    None
                };
                Ok(Some(Self::to_domain(b, p, shop)?))
            }
            None => Ok(None),
        }
    }

    async fn create(&self, item: book::Book) -> anyhow::Result<book::Book> {
        let publisher_model = super::publisher::Entity::find()
            .filter(super::publisher::Column::PubId.eq(item.publisher().pub_id()))
            .one(&self.db)
            .await?
            .ok_or(anyhow::anyhow!("Publisher not found"))?;

        let shop_model = if let Some(s) = item.shop() {
            Some(
                super::shop::Entity::find()
                    .filter(super::shop::Column::PubId.eq(s.pub_id()))
                    .one(&self.db)
                    .await?
                    .ok_or(anyhow::anyhow!("Shop not found"))?,
            )
        } else {
            None
        };

        let active_model = ActiveModel {
            pub_id: Set(item.pub_id()),
            publisher_id: Set(publisher_model.id),
            shop_id: Set(shop_model.as_ref().map(|s| s.id)),
            title: Set(item.title()),
            author: Set(item.author()),
            price: Set(item.price()),
            applied_at: Set(item.applied_at()),
            format: Set(item.format().to_string()),
            created_at: Set(item.created_at()),
            updated_at: Set(item.updated_at()),
            created_by: Set(item.created_by()),
            updated_by: Set(item.updated_by()),
            ..Default::default()
        };
        let result = active_model.insert(&self.db).await?;
        Ok(Self::to_domain(result, Some(publisher_model), shop_model)?)
    }

    async fn update(&self, item: book::Book) -> anyhow::Result<book::Book> {
        let publisher_model = super::publisher::Entity::find()
            .filter(super::publisher::Column::PubId.eq(item.publisher().pub_id()))
            .one(&self.db)
            .await?
            .ok_or(anyhow::anyhow!("Publisher not found"))?;

        let shop_model = if let Some(s) = item.shop() {
            Some(
                super::shop::Entity::find()
                    .filter(super::shop::Column::PubId.eq(s.pub_id()))
                    .one(&self.db)
                    .await?
                    .ok_or(anyhow::anyhow!("Shop not found"))?,
            )
        } else {
            None
        };

        let active_model = ActiveModel {
            id: Set(item.id()),
            pub_id: Set(item.pub_id()),
            publisher_id: Set(publisher_model.id),
            shop_id: Set(shop_model.as_ref().map(|s| s.id)),
            title: Set(item.title()),
            author: Set(item.author()),
            price: Set(item.price()),
            applied_at: Set(item.applied_at()),
            format: Set(item.format().to_string()),
            created_at: Set(item.created_at()),
            updated_at: Set(item.updated_at()),
            created_by: Set(item.created_by()),
            updated_by: Set(item.updated_by()),
        };
        let result = active_model.update(&self.db).await?;

        Ok(Self::to_domain(result, Some(publisher_model), shop_model)?)
    }

    async fn delete(&self, item: book::Book) -> anyhow::Result<()> {
        Entity::delete_by_id(item.id()).exec(&self.db).await?;
        Ok(())
    }
}
