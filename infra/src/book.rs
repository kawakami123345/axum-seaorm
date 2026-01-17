use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::StringLen;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter};

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
    #[sea_orm(has_one)]
    pub publisher: HasOne<super::publisher::Entity>,
    #[sea_orm(has_one)]
    pub shop: HasOne<super::shop::Entity>,
    pub applied_at: Option<chrono::DateTime<chrono::Utc>>,
    #[sea_orm(column_type = "String(StringLen::N(32))")]
    pub format: String,
    pub price: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub user_id: Uuid,
}

impl ActiveModelBehavior for ActiveModel {}

pub struct SqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl SqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn to_domain(model: ModelEx) -> anyhow::Result<book::Book> {
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

        let publisher = match model.publisher.as_ref() {
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

        let shop = match model.shop.as_ref() {
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
            model.id,
            model.pub_id,
            title,
            author,
            publisher,
            shop,
            model.applied_at,
            format,
            price,
            model.created_at,
            model.updated_at,
            model.created_by,
            model.updated_by,
            model.user_id,
        ))
    }
}

#[async_trait]
impl book::Repository for SqlRepository {
    async fn find_all(&self) -> anyhow::Result<Vec<book::Book>> {
        let books = Entity::load()
            .with(super::publisher::Entity)
            .with(super::shop::Entity)
            .all(&self.db)
            .await?
            .into_iter()
            .map(Self::to_domain)
            .collect::<anyhow::Result<Vec<book::Book>>>()?;
        Ok(books)
    }

    async fn find_by_pub_id(&self, pub_id: uuid::Uuid) -> anyhow::Result<Option<book::Book>> {
        let result = Entity::load()
            .filter_by_pub_id(pub_id)
            .with(super::publisher::Entity)
            .with(super::shop::Entity)
            .one(&self.db)
            .await?
            .map(Self::to_domain)
            .transpose()?;
        Ok(result)
    }

    async fn create(&self, item: book::Book) -> anyhow::Result<book::Book> {
        let txn = self.db.begin_with_user(&item.updated_by()).await?;

        let publisher_model = super::publisher::Entity::load()
            .filter_by_pub_id(item.publisher().pub_id())
            .one(&txn)
            .await?
            .ok_or(anyhow::anyhow!("Publisher not found"))?;

        let shop_model = if let Some(s) = item.shop() {
            Some(
                super::shop::Entity::load()
                    .filter_by_pub_id(s.pub_id())
                    .one(&txn)
                    .await?
                    .ok_or(anyhow::anyhow!("Shop not found"))?,
            )
        } else {
            None
        };

        let mut active_model = ActiveModel::builder()
            .set_pub_id(item.pub_id())
            .set_title(item.title().to_string())
            .set_author(item.author().to_string())
            .set_price(item.price())
            .set_applied_at(item.applied_at())
            .set_format(item.format().to_string())
            .set_created_at(item.created_at())
            .set_updated_at(item.updated_at())
            .set_created_by(item.created_by().clone())
            .set_updated_by(item.updated_by().clone())
            .set_user_id(item.user_id().clone())
            .set_publisher(publisher_model.into_active_model());
        if let Some(shop) = shop_model {
            active_model = active_model.set_shop(shop.into_active_model());
        }
        let active_model = active_model.insert(&txn).await?;

        txn.commit().await?;

        Ok(Self::to_domain(active_model)?)
    }

    async fn update(&self, item: book::Book) -> anyhow::Result<book::Book> {
        let txn = self.db.begin_with_user(&item.updated_by()).await?;

        let book = Entity::load()
            .filter_by_pub_id(item.pub_id())
            .with(super::publisher::Entity)
            .with(super::shop::Entity)
            .one(&txn)
            .await?
            .ok_or(anyhow::anyhow!("Book not found"))?;

        let mut active_model = book.clone().into_active_model();

        active_model = active_model
            .set_title(item.title().to_string())
            .set_author(item.author().to_string())
            .set_price(item.price())
            .set_applied_at(item.applied_at())
            .set_format(item.format().to_string())
            .set_updated_at(item.updated_at())
            .set_updated_by(item.updated_by().clone());

        if book.publisher.as_ref().unwrap().pub_id != item.publisher().pub_id() {
            let publisher_model = super::publisher::Entity::load()
                .filter_by_pub_id(item.publisher().pub_id())
                .one(&txn)
                .await?
                .ok_or(anyhow::anyhow!("Publisher not found"))?;
            active_model = active_model.set_publisher(publisher_model.into_active_model());
        }
        match (book.shop.as_ref(), item.shop()) {
            (Some(b), Some(s)) if b.pub_id != s.pub_id() => {
                let shop_model = super::shop::Entity::load()
                    .filter_by_pub_id(s.pub_id())
                    .one(&txn)
                    .await?
                    .ok_or(anyhow::anyhow!("Shop not found"))?;
                active_model = active_model.set_shop(shop_model.into_active_model());
            }
            (Some(_), Some(_)) => {}
            (None, Some(s)) => {
                let shop_model = super::shop::Entity::load()
                    .filter_by_pub_id(s.pub_id())
                    .one(&txn)
                    .await?
                    .ok_or(anyhow::anyhow!("Shop not found"))?;
                active_model = active_model.set_shop(shop_model.into_active_model());
            }
            (Some(_), &None) => {
                // TODO: 消し方わからない
                todo!()
                // active_model = active_model.set_shop(None);
            }
            (None, None) => {}
        }
        let active_model = active_model.update(&txn).await?;

        txn.commit().await?;
        Ok(Self::to_domain(active_model)?)
    }

    async fn delete(&self, item: book::Book, deleted_by: uuid::Uuid) -> anyhow::Result<()> {
        let txn = self.db.begin_with_user(&deleted_by).await?;

        Entity::delete_by_id(item.id()).exec(&txn).await?;
        txn.commit().await?;
        Ok(())
    }
}
