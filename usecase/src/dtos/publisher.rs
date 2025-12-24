use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PublisherCreateDto {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublisherUpdateDto {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublisherResponseDto {
    pub id: i32,
    pub name: String,
}

impl From<domain::Publisher> for PublisherResponseDto {
    fn from(publisher: domain::Publisher) -> Self {
        Self {
            id: publisher.id,
            name: publisher.name,
        }
    }
}
