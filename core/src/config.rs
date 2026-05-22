use crate::error::CoreResult;

pub struct CoreConfig {
    pub friend_limit: u64,
    pub friend_request_limit: u64,
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
        })
    }
}
