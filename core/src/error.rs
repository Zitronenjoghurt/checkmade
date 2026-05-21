pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[cfg(feature = "bitcode")]
    #[error("Bitcode error: {0}")]
    Bitcode(#[from] bitcode::Error),
    #[cfg(feature = "data")]
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::error::DbErr),
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error("Invalid friendship status: {0}")]
    InvalidFriendshipStatus(i16),
}

impl CoreError {
    pub fn is_user_error(&self) -> bool {
        matches!(self, CoreError::Domain(_))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("You are already friends with this user.")]
    AlreadyFriends,
    #[error("You have already sent a friend request to this user.")]
    AlreadySentFriendRequest,
    #[error("This user has already sent you a friend request.")]
    AlreadyReceivedFriendRequest,
    #[error("You cannot send a friend request to this user.")]
    FriendRequestBlocked,
    #[error("This friend code is invalid.")]
    InvalidFriendCode,
    #[error("You have no friend request from this user.")]
    NoFriendRequest,
    #[error("You are not friends with this user.")]
    NotFriends,
}
