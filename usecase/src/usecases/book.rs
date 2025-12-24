use crate::UseCaseError;
use crate::dtos::book::{BookCreateDto, BookResponseDto, BookUpdateDto};
use domain::{Book, BookRepository};
use std::sync::Arc;

pub struct BookUseCase {
    repo: Arc<dyn BookRepository>,
}

impl BookUseCase {
    pub fn new(repo: Arc<dyn BookRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_all_books(&self) -> Result<Vec<BookResponseDto>, UseCaseError> {
        let books = self.repo.find_all().await.map_err(UseCaseError::from)?;
        Ok(books.into_iter().map(BookResponseDto::from).collect())
    }

    pub async fn get_book(&self, id: i32) -> Result<BookResponseDto, UseCaseError> {
        let book = self.repo.find_by_id(id).await.map_err(UseCaseError::from)?;
        Ok(BookResponseDto::from(book))
    }

    pub async fn create_book(&self, dto: BookCreateDto) -> Result<BookResponseDto, UseCaseError> {
        let book = Book {
            id: 0,
            title: dto.title,
            author: dto.author,
            publisher_id: dto.publisher_id,
        };
        let result = self.repo.create(book).await.map_err(UseCaseError::from)?;
        Ok(BookResponseDto::from(result))
    }

    pub async fn update_book(&self, dto: BookUpdateDto) -> Result<BookResponseDto, UseCaseError> {
        let book = Book {
            id: dto.id,
            title: dto.title,
            author: dto.author,
            publisher_id: dto.publisher_id,
        };
        let result = self.repo.update(book).await.map_err(UseCaseError::from)?;
        Ok(BookResponseDto::from(result))
    }

    pub async fn delete_book(&self, id: i32) -> Result<(), UseCaseError> {
        self.repo.delete(id).await.map_err(UseCaseError::from)
    }
}
