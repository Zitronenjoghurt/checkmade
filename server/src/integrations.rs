use crate::error::ServerResult;

mod discord;

pub struct Integrations {
    pub discord: discord::DiscordIntegration,
}

impl Integrations {
    pub fn new(config: &IntegrationsConfig) -> ServerResult<Self> {
        Ok(Self {
            discord: discord::DiscordIntegration::new(&config.discord)?,
        })
    }
}

pub struct IntegrationsConfig {
    pub discord: discord::DiscordConfig,
}

impl IntegrationsConfig {
    pub fn from_env() -> ServerResult<Self> {
        Ok(Self {
            discord: discord::DiscordConfig::from_env()?,
        })
    }
}
