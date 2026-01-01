use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "book")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pub_id: uuid::Uuid,
    pub title: String,
    pub author: String,
    pub publisher_id: i32,
    pub status: String,
    pub price: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::publisher::Entity",
        from = "Column::PublisherId",
        to = "super::publisher::Column::Id"
    )]
    Publisher,
}

impl Related<super::publisher::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Publisher.def()
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
    ) -> anyhow::Result<book::Book> {
        let title = book::vo::BookTitle::new(model.title)
            .map_err(|e| anyhow::anyhow!("Invalid title in DB: {}", e))?;
        let author = book::vo::BookAuthor::new(model.author)
            .map_err(|e| anyhow::anyhow!("Invalid author in DB: {}", e))?;
        let price = book::vo::BookPrice::new(model.price)
            .map_err(|e| anyhow::anyhow!("Invalid price in DB: {}", e))?;
        let status = match model.status.as_str() {
            "Unapplied" => book::vo::BookStatus::Unapplied,
            "Applied" => book::vo::BookStatus::Applied,
            _ => return Err(anyhow::anyhow!("Invalid status in DB: {}", model.status)),
        };

        let publisher_entity = if let Some(p) = publisher {
            publisher::Publisher {
                id: p.id,
                pub_id: p.pub_id,
                name: publisher::vo::PublisherName::new(p.name)
                    .map_err(|e| anyhow::anyhow!("Invalid publisher name in DB: {}", e))?,
            }
        } else {
            return Err(anyhow::anyhow!("Publisher not found for book {}", model.id));
        };

        Ok(book::Book {
            id: model.id,
            pub_id: model.pub_id,
            title,
            author,
            publisher: publisher_entity,
            status,
            price,
        })
    }
}

#[async_trait]
impl book::Repository for SqlRepository {
    async fn find_all(&self) -> anyhow::Result<Vec<book::Book>> {
        let results = Entity::find()
            .find_also_related(super::publisher::Entity)
            .all(&self.db)
            .await?;

        results
            .into_iter()
            .map(|(b, p)| Self::to_domain(b, p))
            .collect()
    }

    async fn find_by_pub_id(&self, pub_id: uuid::Uuid) -> anyhow::Result<Option<book::Book>> {
        let result = Entity::find()
            .filter(Column::PubId.eq(pub_id))
            .find_also_related(super::publisher::Entity)
            .one(&self.db)
            .await?;

        match result {
            Some((b, p)) => Ok(Some(Self::to_domain(b, p)?)),
            None => Ok(None),
        }
    }

    async fn create(&self, item: book::Book) -> anyhow::Result<book::Book> {
        let publisher = super::publisher::Entity::find()
            .filter(super::publisher::Column::PubId.eq(item.publisher.pub_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("Publisher with UUID {} not found", item.publisher.pub_id)
            })?;

        let active_model = ActiveModel {
            pub_id: Set(item.pub_id),
            title: Set(item.title.value().to_string()),
            author: Set(item.author.value().to_string()),
            publisher_id: Set(publisher.id),
            status: Set(item.status.value().to_string()),
            price: Set(item.price.value()),
            ..Default::default()
        };

        let result = active_model.insert(&self.db).await?;
        Self::to_domain(result, Some(publisher))
    }

    async fn update(&self, item: book::Book) -> anyhow::Result<book::Book> {
        let publisher = super::publisher::Entity::find()
            .filter(super::publisher::Column::PubId.eq(item.publisher.pub_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("Publisher with UUID {} not found", item.publisher.pub_id)
            })?;

        let active_model = ActiveModel {
            id: Set(item.id),
            pub_id: Set(item.pub_id),
            title: Set(item.title.value().to_string()),
            author: Set(item.author.value().to_string()),
            publisher_id: Set(publisher.id),
            status: Set(item.status.value().to_string()),
            price: Set(item.price.value()),
        };

        let result = active_model.update(&self.db).await?;
        Self::to_domain(result, Some(publisher))
    }

    async fn delete(&self, item: book::Book) -> anyhow::Result<()> {
        Entity::delete_by_id(item.id).exec(&self.db).await?;
        Ok(())
    }
}
