use crate::error::ServerResult;
use crate::integrations::IntegrationsConfig;

pub struct Config {
    pub database_url: String,
    pub dev_mode: bool,
    pub domain: String,
    pub integrations: IntegrationsConfig,
    pub session_secret: String,
    pub static_dir: String,
    pub max_user_connection_count: usize,
    pub max_ws_message_size_kb: usize,
    pub max_ws_outbound_buffer_size_kb: usize,
    pub ws_rate_limit_max_tokens: f64,
    pub ws_rate_limit_refill_rate: f64,
}

impl Config {
    pub fn from_env() -> ServerResult<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            dev_mode: env_bool("DEV_MODE").unwrap_or(false),
            domain: std::env::var("DOMAIN")?,
            integrations: IntegrationsConfig::from_env()?,
            session_secret: std::env::var("SESSION_SECRET")?,
            static_dir: std::env::var("STATIC_DIR")?,
            max_user_connection_count: std::env::var("MAX_USER_CONNECTION_COUNT")
                .unwrap_or("10".to_string())
                .parse::<usize>()?,
            max_ws_message_size_kb: std::env::var("MAX_WS_MESSAGE_SIZE_KB")
                .unwrap_or("1".to_string())
                .parse::<usize>()?,
            max_ws_outbound_buffer_size_kb: std::env::var("MAX_WS_OUTBOUND_BUFFER_SIZE_KB")
                .unwrap_or("512".to_string())
                .parse::<usize>()?,
            ws_rate_limit_max_tokens: std::env::var("WS_RATE_LIMIT_MAX_TOKENS")
                .unwrap_or("500".to_string())
                .parse::<f64>()?,
            ws_rate_limit_refill_rate: std::env::var("WS_RATE_LIMIT_REFILL_RATE")
                .unwrap_or("20".to_string())
                .parse::<f64>()?,
        })
    }
}

fn env_bool(name: &str) -> ServerResult<bool> {
    Ok(std::env::var(name).map(|s| s == "1" || s.eq_ignore_ascii_case("true"))?)
}
