use crate::config::Config;
use crate::error::ServerResult;
use crate::integrations::Integrations;
use checkmade_core::data::service::Services;
use checkmade_core::data::Data;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServerState {
    pub config: Arc<Config>,
    pub data: Arc<Data>,
    pub integrations: Arc<Integrations>,
    pub service: Arc<Services>,
}

impl ServerState {
    pub async fn new(config: Config) -> ServerResult<Self> {
        let data = Arc::new(Data::initialize(&config.database_url).await?);
        let service = Arc::new(Services::new(&data));
        Ok(Self {
            data,
            service,
            integrations: Arc::new(Integrations::new(&config.integrations)?),
            config: Arc::new(config),
        })
    }
}
