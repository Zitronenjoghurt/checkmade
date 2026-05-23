use crate::config::CoreConfig;
use crate::error::CoreResult;
use migration::prelude::chrono;
use migration::{Migrator, MigratorTrait};
use sea_orm::sqlx::PgPool;
use sea_orm::{ConnectOptions, DatabaseConnection, DatabaseTransaction, TransactionTrait};
use std::sync::Arc;
use tracing::info;

pub mod entity;
mod ext;
pub mod service;
pub mod store;

pub use sea_orm::{IntoActiveModel, Set};

pub struct Data {
    config: Arc<CoreConfig>,
    connection: DatabaseConnection,
    pub friends: store::friendship::FriendshipStore,
    pub session: store::session::SessionStore,
    pub session_request: store::session_request::SessionRequestStore,
    pub user: store::user::UserStore,
}

impl Data {
    pub async fn initialize(
        config: &Arc<CoreConfig>,
        database_url: impl AsRef<str>,
    ) -> CoreResult<Self> {
        let options = ConnectOptions::new(database_url.as_ref());

        info!("Connecting to database...");
        let connection = sea_orm::Database::connect(options).await?;
        info!("Connected to database!");

        let data = Self {
            friends: store::friendship::FriendshipStore::new(config, &connection),
            session: store::session::SessionStore::new(config, &connection),
            session_request: store::session_request::SessionRequestStore::new(config, &connection),
            user: store::user::UserStore::new(&connection),
            config: Arc::clone(config),
            connection,
        };
        data.apply_migrations().await?;

        Ok(data)
    }

    pub fn pool(&self) -> &PgPool {
        self.connection.get_postgres_connection_pool()
    }

    pub async fn begin_txn(&self) -> CoreResult<DatabaseTransaction> {
        self.connection.begin().await.map_err(Into::into)
    }

    async fn apply_migrations(&self) -> CoreResult<()> {
        info!("Applying database migrations...");
        Migrator::up(&self.connection, None).await?;
        info!("Database migrations applied!");
        Ok(())
    }
}

pub fn chrono_now() -> chrono::NaiveDateTime {
    chrono::Utc::now().naive_utc()
}
