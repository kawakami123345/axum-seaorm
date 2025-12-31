#[cfg(test)]
mod tests {
    use infra::publisher::fake::FakeRepository as PublisherRepo;
    use std::sync::Arc;
    use usecase::book::{CreateDto as BookCreateDto, Service as BookService};
    use usecase::publisher::{CreateDto as PublisherCreateDto, Service as PublisherService};

    #[tokio::test]
    async fn test_book_usecase() {
        // Arrange
        let book_repo = Arc::new(infra::book::fake::FakeRepository::new(vec![]));
        let service = BookService::new(book_repo);

        // Act - Create
        let dto = BookCreateDto {
            title: "Test Book".to_string(),
            author: "Tester".to_string(),
            publisher_id: 1,
        };
        let created = service.create(dto).await.expect("failed to create");

        // Assert
        assert_eq!(created.title, "Test Book");
        assert_eq!(created.id, 1);

        // Act - Get All
        let books = service.get_all().await.expect("failed to get all");

        // Assert
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].title, "Test Book");
    }

    #[tokio::test]
    async fn test_publisher_usecase() {
        // Arrange
        let pub_repo = Arc::new(PublisherRepo::new(vec![]));
        let service = PublisherService::new(pub_repo);

        // Act - Create
        let dto = PublisherCreateDto {
            name: "Test Publisher".to_string(),
        };
        let created = service.create(dto).await.expect("failed to create");

        // Assert
        assert_eq!(created.name, "Test Publisher");
        assert_eq!(created.id, 1);

        // Act - Get All
        let pubs = service.get_all().await.expect("failed to get all");

        // Assert
        assert_eq!(pubs.len(), 1);
        assert_eq!(pubs[0].name, "Test Publisher");
    }
}
