use async_trait::async_trait;
use domain::{DomainError, RepositoryBase, book};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "book")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub author: String,
    pub publisher_id: i32,
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

pub struct RepositoryImpl {
    pub(crate) db: DatabaseConnection,
}

impl RepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<Model> for book::Book {
    fn from(model: Model) -> Self {
        book::Book {
            id: model.id,
            title: model.title,
            author: model.author,
            publisher_id: model.publisher_id,
        }
    }
}

#[async_trait]
impl RepositoryBase<book::Book> for RepositoryImpl {
    async fn find_all(&self) -> Result<Vec<book::Book>, DomainError> {
        let books = Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| DomainError::InfraError(e.to_string()))?;

        Ok(books.into_iter().map(book::Book::from).collect())
    }

    async fn find_by_id(&self, id: i32) -> Result<book::Book, DomainError> {
        let book = Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| DomainError::InfraError(e.to_string()))?
            .ok_or(DomainError::NotFound)?;

        Ok(book::Book::from(book))
    }

    async fn create(&self, item: book::Book) -> Result<book::Book, DomainError> {
        let active_model = ActiveModel {
            title: Set(item.title),
            author: Set(item.author),
            publisher_id: Set(item.publisher_id),
            ..Default::default() // id is ignored/auto-incremented
        };

        let result = active_model
            .insert(&self.db)
            .await
            .map_err(|e| DomainError::InfraError(e.to_string()))?;

        Ok(book::Book::from(result))
    }

    async fn update(&self, item: book::Book) -> Result<book::Book, DomainError> {
        let active_model = ActiveModel {
            id: Set(item.id),
            title: Set(item.title),
            author: Set(item.author),
            publisher_id: Set(item.publisher_id),
        };

        let result = active_model
            .update(&self.db)
            .await
            .map_err(|e| DomainError::InfraError(e.to_string()))?;

        Ok(book::Book::from(result))
    }

    async fn delete(&self, id: i32) -> Result<(), DomainError> {
        let book = Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| DomainError::InfraError(e.to_string()))?
            .ok_or(DomainError::NotFound)?;

        let result = Entity::delete_by_id(book.id)
            .exec(&self.db)
            .await
            .map_err(|e| DomainError::InfraError(e.to_string()))?;

        if result.rows_affected == 0 {
            return Err(DomainError::NotFound);
        }

        Ok(())
    }
}

#[async_trait]
impl book::Repository for RepositoryImpl {}
