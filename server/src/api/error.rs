use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tracing::error;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    Core(#[from] checkmade_core::error::CoreError),
    #[error("Discord API unreachable: {0}")]
    DiscordUnreachable(String),
    #[error("Discord user invalid")]
    DiscordUserInvalid,
    #[error("Invalid CSRF token")]
    InvalidCsrfToken,
    #[error("Missing CSRF token")]
    MissingCsrfToken,
    #[error("Missing PKCE verifier")]
    MissingPkceVerifier,
    #[error("OAuth2 token exchange failed")]
    OauthTokenExchange,
    #[error("Session error: {0}")]
    Session(#[from] tower_sessions::session::Error),
    #[error("You have permanently banned for rate limit abuse")]
    BannedRateLimitAbuse,
    #[error("Too many connections")]
    TooManyConnections,
    #[error("Unauthorized")]
    Unauthorized,
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BannedRateLimitAbuse
            | Self::DiscordUserInvalid
            | Self::InvalidCsrfToken
            | Self::MissingCsrfToken
            | Self::MissingPkceVerifier
            | Self::TooManyConnections => StatusCode::BAD_REQUEST,
            Self::Core(_)
            | Self::DiscordUnreachable(_)
            | Self::OauthTokenExchange
            | Self::Session(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }

    pub fn should_log(&self) -> bool {
        match self {
            Self::BannedRateLimitAbuse
            | Self::DiscordUserInvalid
            | Self::InvalidCsrfToken
            | Self::MissingCsrfToken
            | Self::MissingPkceVerifier
            | Self::TooManyConnections
            | Self::Unauthorized => false,
            Self::Core(_)
            | Self::DiscordUnreachable(_)
            | Self::OauthTokenExchange
            | Self::Session(_) => true,
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            Self::BannedRateLimitAbuse
            | Self::DiscordUserInvalid
            | Self::InvalidCsrfToken
            | Self::MissingCsrfToken
            | Self::MissingPkceVerifier
            | Self::OauthTokenExchange
            | Self::TooManyConnections
            | Self::Unauthorized => self.to_string(),
            Self::Core(_) => "Internal server error".to_string(),
            Self::DiscordUnreachable(_) => "Discord API unreachable".to_string(),
            Self::Session(_) => "Session error".to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if self.should_log() {
            error!("{self}");
        };
        (self.status_code(), self.user_message()).into_response()
    }
}
