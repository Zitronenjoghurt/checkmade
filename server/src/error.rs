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
}
