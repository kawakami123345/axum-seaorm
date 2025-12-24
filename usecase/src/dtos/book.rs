use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BookCreateDto {
    pub title: String,
    pub author: String,
    pub publisher_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookUpdateDto {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub publisher_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookResponseDto {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub publisher_id: i32,
}

impl From<domain::Book> for BookResponseDto {
    fn from(book: domain::Book) -> Self {
        Self {
            id: book.id,
            title: book.title,
            author: book.author,
            publisher_id: book.publisher_id,
        }
    }
}
