use async_trait::async_trait;
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

pub struct PostgresRepository {
    pub(crate) db: DatabaseConnection,
}

impl PostgresRepository {
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
impl book::Repository for PostgresRepository {
    async fn find_all(&self) -> anyhow::Result<Vec<book::Book>> {
        let books = Entity::find().all(&self.db).await?;
        Ok(books.into_iter().map(book::Book::from).collect())
    }

    async fn find_by_id(&self, id: i32) -> anyhow::Result<Option<book::Book>> {
        let book = Entity::find_by_id(id).one(&self.db).await?;
        Ok(book.map(book::Book::from))
    }

    async fn create(&self, item: book::Book) -> anyhow::Result<book::Book> {
        let active_model = ActiveModel {
            title: Set(item.title),
            author: Set(item.author),
            publisher_id: Set(item.publisher_id),
            ..Default::default() // id is ignored/auto-incremented
        };

        let result = active_model.insert(&self.db).await?;
        Ok(book::Book::from(result))
    }

    async fn update(&self, item: book::Book) -> anyhow::Result<book::Book> {
        let active_model = ActiveModel {
            id: Set(item.id),
            title: Set(item.title),
            author: Set(item.author),
            publisher_id: Set(item.publisher_id),
        };

        let result = active_model.update(&self.db).await?;
        Ok(book::Book::from(result))
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        let _ = Entity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod test {
    use async_trait::async_trait;
    use tokio::sync::Mutex;

    #[derive(Debug)]
    pub struct MockRepository {
        books: Mutex<Vec<book::Book>>,
    }

    impl MockRepository {
        pub fn new(initial: Vec<book::Book>) -> Self {
            Self {
                books: Mutex::new(initial),
            }
        }
    }

    #[async_trait]
    impl book::Repository for MockRepository {
        async fn find_all(&self) -> anyhow::Result<Vec<book::Book>> {
            Ok(self.books.lock().await.clone())
        }

        async fn find_by_id(&self, id: i32) -> anyhow::Result<Option<book::Book>> {
            Ok(self
                .books
                .lock()
                .await
                .iter()
                .find(|&b| b.id == id)
                .cloned())
        }

        async fn create(&self, mut item: book::Book) -> anyhow::Result<book::Book> {
            let mut v = self.books.lock().await;
            let next_id = v.iter().map(|b| b.id).max().unwrap_or(0) + 1;
            item.id = next_id;
            v.push(item.clone());
            Ok(item)
        }

        async fn update(&self, item: book::Book) -> anyhow::Result<book::Book> {
            let mut v = self.books.lock().await;
            if let Some(pos) = v.iter().position(|b| b.id == item.id) {
                v[pos] = item.clone();
                Ok(item)
            } else {
                Err(anyhow::anyhow!("not found"))
            }
        }

        async fn delete(&self, id: i32) -> anyhow::Result<()> {
            let mut v = self.books.lock().await;
            let original = v.len();
            v.retain(|b| b.id != id);
            if v.len() == original {
                Err(anyhow::anyhow!("not found"))
            } else {
                Ok(())
            }
        }
    }
}
