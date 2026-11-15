use async_trait::async_trait;
use sea_orm::{DatabaseConnection, Set, entity::prelude::*};
use usecase::session::{Repository, Session};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "auth_session")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_context: Json,
    #[sea_orm(indexed)]
    pub expires_at: time::OffsetDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub struct SqlRepository {
    db: DatabaseConnection,
}

impl SqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Repository for SqlRepository {
    async fn create(&self, session: Session) -> anyhow::Result<()> {
        // Remove expired sessions as new sessions are issued to bound storage usage.
        Entity::delete_many()
            .filter(Column::ExpiresAt.lte(time::OffsetDateTime::now_utc()))
            .exec(&self.db)
            .await?;
        ActiveModel {
            id: Set(session.id),
            user_context: Set(serde_json::to_value(session.user)?),
            expires_at: Set(session.expires_at),
        }
        .insert(&self.db)
        .await?;
        Ok(())
    }

    async fn find(&self, id: Uuid) -> anyhow::Result<Option<Session>> {
        Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .map(|model| {
                Ok(Session {
                    id: model.id,
                    user: serde_json::from_value(model.user_context)?,
                    expires_at: model.expires_at,
                })
            })
            .transpose()
    }

    async fn revoke(&self, id: Uuid) -> anyhow::Result<()> {
        Entity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }
}
