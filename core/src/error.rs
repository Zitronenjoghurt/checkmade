use giga_chess::error::SessionError;

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
    #[error("Error reading environment variable: {0}")]
    Env(#[from] std::env::VarError),
    #[error("Invalid friendship status: {0}")]
    InvalidFriendshipStatus(i16),
    #[error("Invalid session status: {0}")]
    InvalidSessionStatus(i16),
    #[error("Parse int error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Parse float error: {0}")]
    ParseFloat(#[from] std::num::ParseFloatError),
    #[error("Session config serialization error: {0}")]
    SessionConfigSerialization(String),
    #[error("Session config deserialization error: {0}")]
    SessionConfigDeserialization(String),
    #[error("Session creation error: {0}")]
    SessionCreation(String),
    #[error("Session serialization error: {0}")]
    SessionSerialization(String),
    #[error("Session deserialization error: {0}")]
    SessionDeserialization(String),
    #[error("Session restoration error: {0}")]
    SessionRestoration(String),
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
    #[error(
        "You or the other person have reached the friend limit (max {0}) and cannot add any more friends."
    )]
    FriendLimitReached(u64),
    #[error(
        "You or the other person have reached the friend request limit (max {0}) and cannot send or receive any more friend requests."
    )]
    FriendRequestLimitReached(u64),
    #[error("This friend code is invalid.")]
    InvalidFriendCode,
    #[error("This move is invalid.")]
    InvalidMove,
    #[error("You have no friend request from this user.")]
    NoFriendRequest,
    #[error("You are not friends with this user.")]
    NotFriends,
    #[error("You are not a participant in this session.")]
    NotSessionParticipant,
    #[error("You cannot create a private session without specifying an opponent.")]
    PrivateSessionsRequireOpponent,
    #[error("Session error: {0}")]
    Session(#[from] SessionError),
    #[error(
        "You or your opponent have reached their session limit (max {0}) and cannot create any more sessions."
    )]
    SessionLimitReached(u64),
    #[error("This session is invalid.")]
    SessionNotFound,
    #[error(
        "You or your opponent have reached their session request limit (max {0}) and cannot create or receive any more session requests."
    )]
    SessionRequestLimitReached(u64),
    #[error("This session request is invalid.")]
    SessionRequestNotFound,
}
