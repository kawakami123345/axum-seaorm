use thiserror::Error;

#[derive(Debug, Error)]
#[error("Name already exists")]
pub struct DuplicateName;

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("Entity not found")]
    NotFound(String),
    #[error("Write conflict: {0}")]
    Conflict(String),
    #[error("Internal server error")]
    InternalServerError,
    #[error("Database execution failed")]
    DatabaseError,
    #[error("Domain rule violation: {0}")]
    DomainRuleViolation(String),
    #[error("Domain error occurred: {0}")]
    BookDomainError(#[from] book::DomainError),
    #[error("Domain error occurred: {0}")]
    PublisherDomainError(#[from] publisher::DomainError),
    #[error("Domain error occurred: {0}")]
    ShopDomainError(#[from] shop::DomainError),
}

impl UseCaseError {
    pub(crate) fn name_write(
        operation: &str,
        duplicate_message: &str,
        error: anyhow::Error,
    ) -> Self {
        if error.is::<DuplicateName>() {
            Self::DomainRuleViolation(duplicate_message.into())
        } else {
            Self::database(operation, error)
        }
    }

    pub(crate) fn check_revision(expected: i64, current: i64) -> Result<(), Self> {
        if expected < 0 || expected == i64::MAX {
            return Err(Self::DomainRuleViolation("Invalid revision".into()));
        }
        if expected != current {
            return Err(Self::write_conflict());
        }
        Ok(())
    }

    pub(crate) fn check_write(applied: bool) -> Result<(), Self> {
        if !applied {
            return Err(Self::write_conflict());
        }
        Ok(())
    }

    fn write_conflict() -> Self {
        Self::Conflict(
            "別の操作で更新または削除されました。再読み込みしてからやり直してください。".into(),
        )
    }

    pub(crate) fn database(operation: &str, error: anyhow::Error) -> Self {
        eprintln!("Database error in {operation}: {error:?}");
        Self::DatabaseError
    }
}
