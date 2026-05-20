use crate::error::CoreResult;
use migration::prelude::chrono;
use migration::{Migrator, MigratorTrait};
use sea_orm::sqlx::PgPool;
use sea_orm::{ConnectOptions, DatabaseConnection, DatabaseTransaction, TransactionTrait};
use tracing::info;

pub mod entity;
pub mod service;
pub mod store;

pub use sea_orm::{IntoActiveModel, Set};

pub struct Data {
    connection: DatabaseConnection,
    pub user: store::user::UserStore,
}

impl Data {
    pub async fn initialize(database_url: impl AsRef<str>) -> CoreResult<Self> {
        let options = ConnectOptions::new(database_url.as_ref());

        info!("Connecting to database...");
        let connection = sea_orm::Database::connect(options).await?;
        info!("Connected to database!");

        let data = Self {
            user: store::user::UserStore::new(&connection),
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
