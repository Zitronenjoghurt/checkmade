pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error(transparent)]
    Core(#[from] checkmade_core::error::CoreError),
    #[error("Error reading environment variable: {0}")]
    Env(#[from] std::env::VarError),
    #[error("Json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Parse int error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Parse float error: {0}")]
    ParseFloat(#[from] std::num::ParseFloatError),
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] oauth2::reqwest::Error),
    #[error("OAuth2 token request error: {0}")]
    TokenRequest(String),
    #[error("Error parsing URL: {0}")]
    Url(#[from] oauth2::url::ParseError),
    #[error(transparent)]
    User(#[from] UserError),
}

impl ServerError {
    pub fn message(&self) -> String {
        match self {
            Self::Core(e) => e.to_string(),
            Self::User(e) => e.to_string(),
            _ => "An unexpected error occurred".to_string(),
        }
    }

    pub fn is_user_error(&self) -> bool {
        match self {
            Self::Core(e) => e.is_user_error(),
            Self::User(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("User not found")]
    UserNotFound,
}
