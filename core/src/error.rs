pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[cfg(feature = "data")]
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::error::DbErr),
}
