use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::StringLen;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use crate::BeginWithUser;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "book")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pub_id: Uuid,
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
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub user_id: Uuid,

    #[sea_orm(belongs_to, from = "publisher_id", to = "id")]
    pub publisher: HasOne<super::publisher::Entity>,
    #[sea_orm(belongs_to, from = "shop_id", to = "id")]
    pub shop: HasOne<super::shop::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl ModelEx {
    fn to_domain(&self) -> anyhow::Result<book::Book> {
        let title = book::vo::BookTitle::new(self.title.clone())
            .map_err(|e| anyhow::anyhow!("Invalid title in DB: {}", e))?;
        let author = book::vo::BookAuthor::new(self.author.clone())
            .map_err(|e| anyhow::anyhow!("Invalid author in DB: {}", e))?;
        let price = book::vo::BookPrice::new(self.price)
            .map_err(|e| anyhow::anyhow!("Invalid price in DB: {}", e))?;
        let format = match self.format.as_str() {
            "Real" => book::vo::BookFormat::Real,
            "EBook" => book::vo::BookFormat::EBook,
            _ => return Err(anyhow::anyhow!("Invalid format in DB: {}", self.format)),
        };

        let publisher = match self.publisher.as_ref() {
            Some(p) => {
                let name = publisher::vo::PublisherName::new(p.name.clone())
                    .map_err(|e| anyhow::anyhow!("Invalid publisher name in DB: {}", e))?;

                publisher::Publisher::reconstruct(
                    p.id,
                    p.pub_id,
                    name,
                    p.created_at,
                    p.updated_at,
                    p.created_by,
                    p.updated_by,
                )
            }
            None => return Err(anyhow::anyhow!("Publisher not found in DB")),
        };

        let shop = match self.shop.as_ref() {
            Some(s) => {
                let name = shop::vo::ShopName::new(s.name.clone())
                    .map_err(|e| anyhow::anyhow!("Invalid shop name in DB: {}", e))?;

                Some(shop::Shop::reconstruct(
                    s.id,
                    s.pub_id,
                    name,
                    s.created_at,
                    s.updated_at,
                    s.created_by,
                    s.updated_by,
                ))
            }
            None => None,
        };

        Ok(book::Book::reconstruct(
            self.id,
            self.pub_id,
            title,
            author,
            publisher,
            shop,
            self.applied_at,
            format,
            price,
            self.created_at,
            self.updated_at,
            self.created_by,
            self.updated_by,
            self.user_id,
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
impl book::Repository for SqlRepository {
    async fn find_all(&self) -> anyhow::Result<Vec<book::Book>> {
        Entity::load()
            .order_by(Column::Id, sea_orm::Order::Desc)
            .with(super::publisher::Entity)
            .with(super::shop::Entity)
            .all(&self.db)
            .await?
            .into_iter()
            .map(|m| m.to_domain())
            .collect::<anyhow::Result<Vec<book::Book>>>()
    }

    async fn find_by_pub_id(&self, pub_id: uuid::Uuid) -> anyhow::Result<Option<book::Book>> {
        Entity::load()
            .filter_by_pub_id(pub_id)
            .with(super::publisher::Entity)
            .with(super::shop::Entity)
            .one(&self.db)
            .await?
            .map(|m| m.to_domain())
            .transpose()
    }

    async fn create(&self, item: book::Book) -> anyhow::Result<book::Book> {
        let txn = self.db.begin_with_user(item.updated_by()).await?;

        let book_domain = ActiveModel::builder()
            .set_pub_id(item.pub_id())
            .set_title(item.title().to_string())
            .set_author(item.author().to_string())
            .set_price(item.price())
            .set_applied_at(item.applied_at())
            .set_format(item.format().to_string())
            .set_created_at(item.created_at())
            .set_updated_at(item.updated_at())
            .set_created_by(*item.created_by())
            .set_updated_by(*item.updated_by())
            .set_user_id(*item.user_id())
            // 基本的にset_publisher_idではなくset_publisherで更新する
            .set_publisher(super::publisher::ActiveModel::builder().set_id(item.publisher().id()))
            // shopはidだけでいい？Option<i32>だから？
            .set_shop_id(item.shop().clone().map(|s| s.id()))
            .insert(&txn)
            .await?
            .to_domain()?;

        txn.commit().await?;

        Ok(book_domain)
    }

    async fn update(&self, item: book::Book) -> anyhow::Result<book::Book> {
        let txn = self.db.begin_with_user(item.updated_by()).await?;

        let book_domain = ActiveModel::builder()
            .set_id(item.id())
            .set_pub_id(item.pub_id())
            .set_title(item.title().to_string())
            .set_author(item.author().to_string())
            .set_price(item.price())
            .set_applied_at(item.applied_at())
            .set_format(item.format().to_string())
            .set_created_at(item.created_at())
            .set_updated_at(item.updated_at())
            .set_created_by(*item.created_by())
            .set_updated_by(*item.updated_by())
            .set_user_id(*item.user_id())
            .set_publisher(super::publisher::ActiveModel::builder().set_id(item.publisher().id()))
            .set_shop_id(item.shop().clone().map(|s| s.id()))
            .update(&txn)
            .await?
            .to_domain()?;

        txn.commit().await?;
        Ok(book_domain)
    }

    async fn delete(&self, item: book::Book, deleted_by: uuid::Uuid) -> anyhow::Result<()> {
        let txn = self.db.begin_with_user(&deleted_by).await?;

        Entity::delete_by_id(item.id()).exec(&txn).await?;
        txn.commit().await?;
        Ok(())
    }
}
