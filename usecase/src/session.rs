use std::sync::Arc;

use async_trait::async_trait;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{UserContext, error::UseCaseError};

pub const SESSION_LIFETIME: time::Duration = time::Duration::hours(24);

#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub user: UserContext,
    pub expires_at: OffsetDateTime,
}

#[async_trait]
pub trait Repository: Send + Sync {
    async fn create(&self, session: Session) -> anyhow::Result<()>;
    async fn find(&self, id: Uuid) -> anyhow::Result<Option<Session>>;
    async fn revoke(&self, id: Uuid) -> anyhow::Result<()>;
}

pub struct Service {
    repo: Arc<dyn Repository>,
}

impl Service {
    pub fn new(repo: Arc<dyn Repository>) -> Self {
        Self { repo }
    }

    pub async fn create(&self, user: UserContext) -> Result<Session, UseCaseError> {
        let session = Session {
            id: Uuid::new_v4(),
            user,
            expires_at: OffsetDateTime::now_utc() + SESSION_LIFETIME,
        };
        self.repo
            .create(session.clone())
            .await
            .map_err(|e| UseCaseError::database("create session", e))?;
        Ok(session)
    }

    pub async fn authenticate(&self, id: Uuid) -> Result<Option<UserContext>, UseCaseError> {
        Ok(self
            .repo
            .find(id)
            .await
            .map_err(|e| UseCaseError::database("find session", e))?
            .filter(|session| session.expires_at > OffsetDateTime::now_utc())
            .map(|session| session.user))
    }

    pub async fn revoke(&self, id: Uuid) -> Result<(), UseCaseError> {
        self.repo
            .revoke(id)
            .await
            .map_err(|e| UseCaseError::database("revoke session", e))
    }
}
