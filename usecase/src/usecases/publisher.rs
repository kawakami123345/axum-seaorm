use crate::UseCaseError;
use crate::dtos::publisher::{PublisherCreateDto, PublisherResponseDto, PublisherUpdateDto};
use domain::{Publisher, PublisherRepository};
use std::sync::Arc;

pub struct PublisherUseCase {
    repo: Arc<dyn PublisherRepository>,
}

impl PublisherUseCase {
    pub fn new(repo: Arc<dyn PublisherRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_all_publishers(&self) -> Result<Vec<PublisherResponseDto>, UseCaseError> {
        let publishers = self.repo.find_all().await.map_err(UseCaseError::from)?;
        Ok(publishers
            .into_iter()
            .map(PublisherResponseDto::from)
            .collect())
    }

    pub async fn get_publisher(&self, id: i32) -> Result<PublisherResponseDto, UseCaseError> {
        let publisher = self.repo.find_by_id(id).await.map_err(UseCaseError::from)?;
        Ok(PublisherResponseDto::from(publisher))
    }

    pub async fn create_publisher(
        &self,
        dto: PublisherCreateDto,
    ) -> Result<PublisherResponseDto, UseCaseError> {
        let publisher = Publisher {
            id: 0,
            name: dto.name,
        };
        let result = self
            .repo
            .create(publisher)
            .await
            .map_err(UseCaseError::from)?;
        Ok(PublisherResponseDto::from(result))
    }

    pub async fn update_publisher(
        &self,
        dto: PublisherUpdateDto,
    ) -> Result<PublisherResponseDto, UseCaseError> {
        let publisher = Publisher {
            id: dto.id,
            name: dto.name,
        };
        let result = self
            .repo
            .update(publisher)
            .await
            .map_err(UseCaseError::from)?;
        Ok(PublisherResponseDto::from(result))
    }

    pub async fn delete_publisher(&self, id: i32) -> Result<(), UseCaseError> {
        self.repo.delete(id).await.map_err(UseCaseError::from)
    }
}
