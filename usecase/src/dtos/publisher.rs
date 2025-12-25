use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublisherCreateDto {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublisherUpdateDto {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
