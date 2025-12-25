use crate::entities::book;
use async_trait::async_trait;
use domain::{Book, interfaces::BookRepository, models::DomainError};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

pub struct BookRepositoryImpl {
    pub(crate) db: DatabaseConnection,
}

impl BookRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<book::Model> for Book {
    fn from(model: book::Model) -> Self {
        Book {
            id: model.id,
            title: model.title,
            author: model.author,
            publisher_id: model.publisher_id,
        }
    }
}

use domain::interfaces::RepositoryBase;

#[async_trait]
impl RepositoryBase<Book> for BookRepositoryImpl {
    async fn find_all(&self) -> Result<Vec<Book>, DomainError> {
        let books = book::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(books.into_iter().map(Book::from).collect())
    }

    async fn find_by_id(&self, id: i32) -> Result<Book, DomainError> {
        let book = book::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?
            .ok_or(DomainError::NotFound)?;

        Ok(Book::from(book))
    }

    async fn create(&self, item: Book) -> Result<Book, DomainError> {
        let active_model = book::ActiveModel {
            title: Set(item.title),
            author: Set(item.author),
            publisher_id: Set(item.publisher_id),
            ..Default::default() // id is ignored/auto-incremented
        };

        let result = active_model
            .insert(&self.db)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(Book::from(result))
    }

    async fn update(&self, item: Book) -> Result<Book, DomainError> {
        let active_model = book::ActiveModel {
            id: Set(item.id),
            title: Set(item.title),
            author: Set(item.author),
            publisher_id: Set(item.publisher_id),
        };

        let result = active_model
            .update(&self.db)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(Book::from(result))
    }

    async fn delete(&self, id: i32) -> Result<(), DomainError> {
        let book = book::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?
            .ok_or(DomainError::NotFound)?;

        let result = book::Entity::delete_by_id(book.id)
            .exec(&self.db)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        if result.rows_affected == 0 {
            return Err(DomainError::NotFound);
        }

        Ok(())
    }
}

#[async_trait]
impl BookRepository for BookRepositoryImpl {}
