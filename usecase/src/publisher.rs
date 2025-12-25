use std::sync::Arc;

use crate::error::UseCaseError;
use domain::publisher;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub struct Service {
    repo: Arc<dyn publisher::Repository>,
}

impl Service {
    pub fn new(repo: Arc<dyn publisher::Repository>) -> Self {
        Self { repo }
    }

    pub async fn get_all(&self) -> Result<Vec<ResponseDto>, UseCaseError> {
        let publishers = self.repo.find_all().await.map_err(UseCaseError::from)?;
        Ok(publishers.into_iter().map(ResponseDto::from).collect())
    }

    pub async fn get(&self, id: i32) -> Result<ResponseDto, UseCaseError> {
        let publisher = self.repo.find_by_id(id).await.map_err(UseCaseError::from)?;
        Ok(ResponseDto::from(publisher))
    }

    pub async fn create(&self, dto: CreateDto) -> Result<ResponseDto, UseCaseError> {
        let publisher = publisher::Publisher {
            id: 0,
            name: dto.name,
        };
        let result = self
            .repo
            .create(publisher)
            .await
            .map_err(UseCaseError::from)?;
        Ok(ResponseDto::from(result))
    }

    pub async fn update(&self, dto: UpdateDto) -> Result<ResponseDto, UseCaseError> {
        let publisher = publisher::Publisher {
            id: dto.id,
            name: dto.name,
        };
        let result = self
            .repo
            .update(publisher)
            .await
            .map_err(UseCaseError::from)?;
        Ok(ResponseDto::from(result))
    }

    pub async fn delete(&self, id: i32) -> Result<(), UseCaseError> {
        self.repo.delete(id).await.map_err(UseCaseError::from)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateDto {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateDto {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ResponseDto {
    pub id: i32,
    pub name: String,
}

impl From<publisher::Publisher> for ResponseDto {
    fn from(publisher: publisher::Publisher) -> Self {
        Self {
            id: publisher.id,
            name: publisher.name,
        }
    }
}
