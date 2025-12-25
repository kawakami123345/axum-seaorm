use domain::book;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::error::UseCaseError;
pub struct Service {
    repo: Arc<dyn book::Repository>,
}

impl Service {
    pub fn new(repo: Arc<dyn book::Repository>) -> Self {
        Self { repo }
    }

    pub async fn get_all(&self) -> Result<Vec<ResponseDto>, UseCaseError> {
        let books = self.repo.find_all().await.map_err(UseCaseError::from)?;
        Ok(books.into_iter().map(ResponseDto::from).collect())
    }

    pub async fn get(&self, id: i32) -> Result<ResponseDto, UseCaseError> {
        let book = self.repo.find_by_id(id).await.map_err(UseCaseError::from)?;
        Ok(ResponseDto::from(book))
    }

    pub async fn create(&self, dto: CreateDto) -> Result<ResponseDto, UseCaseError> {
        let book = book::Book {
            id: 0,
            title: dto.title,
            author: dto.author,
            publisher_id: dto.publisher_id,
        };
        let result = self.repo.create(book).await.map_err(UseCaseError::from)?;
        Ok(ResponseDto::from(result))
    }

    pub async fn update(&self, dto: UpdateDto) -> Result<ResponseDto, UseCaseError> {
        let book = book::Book {
            id: dto.id,
            title: dto.title,
            author: dto.author,
            publisher_id: dto.publisher_id,
        };
        let result = self.repo.update(book).await.map_err(UseCaseError::from)?;
        Ok(ResponseDto::from(result))
    }

    pub async fn delete(&self, id: i32) -> Result<(), UseCaseError> {
        self.repo.delete(id).await.map_err(UseCaseError::from)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = BookCreateDto)]
pub struct CreateDto {
    pub title: String,
    pub author: String,
    pub publisher_id: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = BookUpdateDto)]
pub struct UpdateDto {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub publisher_id: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = BookResponseDto)]
pub struct ResponseDto {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub publisher_id: i32,
}

impl From<book::Book> for ResponseDto {
    fn from(book: book::Book) -> Self {
        Self {
            id: book.id,
            title: book.title,
            author: book.author,
            publisher_id: book.publisher_id,
        }
    }
}
