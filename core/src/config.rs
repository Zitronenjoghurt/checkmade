use crate::error::CoreResult;

pub struct CoreConfig {
    pub friend_limit: u64,
    pub friend_request_limit: u64,
    pub session_limit: u64,
    pub session_request_limit: u64,
}

impl CoreConfig {
    pub fn from_env() -> CoreResult<Self> {
        Ok(Self {
            friend_limit: std::env::var("FRIEND_LIMIT")
                .unwrap_or("500".to_string())
                .parse::<u64>()?,
            friend_request_limit: std::env::var("FRIEND_REQUEST_LIMIT")
                .unwrap_or("500".to_string())
                .parse::<u64>()?,
            session_limit: std::env::var("SESSION_LIMIT")
                .unwrap_or("20".to_string())
                .parse::<u64>()?,
            session_request_limit: std::env::var("SESSION_REQUEST_LIMIT")
                .unwrap_or("5".to_string())
                .parse::<u64>()?,
        })
    }
}
