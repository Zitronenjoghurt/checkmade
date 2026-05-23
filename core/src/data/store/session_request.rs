use crate::config::CoreConfig;
use crate::data::entity::session_request;
use crate::data::store::Store;
use crate::error::{CoreResult, DomainError};
use crate::types::session_request::SessionRequest;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, Set, TransactionTrait,
};
use std::sync::Arc;
use uuid::Uuid;

pub struct SessionRequestStore {
    config: Arc<CoreConfig>,
    connection: DatabaseConnection,
}

impl Store for SessionRequestStore {
    type Entity = session_request::Entity;
    type ActiveModel = session_request::ActiveModel;

    fn db(&self) -> &DatabaseConnection {
        &self.connection
    }
}

impl SessionRequestStore {
    pub fn new(config: &Arc<CoreConfig>, connection: &DatabaseConnection) -> Self {
        Self {
            config: Arc::clone(config),
            connection: connection.clone(),
        }
    }

    pub async fn count_outgoing<C: ConnectionTrait>(conn: &C, user_id: Uuid) -> CoreResult<u64> {
        Self::count_with(conn, session_request::Column::RequesterId.eq(user_id)).await
    }

    pub async fn count_incoming<C: ConnectionTrait>(conn: &C, user_id: Uuid) -> CoreResult<u64> {
        Self::count_with(conn, session_request::Column::AddresseeId.eq(Some(user_id))).await
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        request: SessionRequest,
    ) -> CoreResult<session_request::Model> {
        let txn = self.connection.begin().await?;

        let outgoing = Self::count_outgoing(&txn, user_id).await?;
        if outgoing >= self.config.session_request_limit {
            return Err(
                DomainError::SessionRequestLimitReached(self.config.session_request_limit).into(),
            );
        };

        if let Some(opponent_id) = request.opponent_id {
            let incoming = Self::count_incoming(&txn, opponent_id.into()).await?;
            if incoming >= self.config.session_request_limit {
                return Err(DomainError::SessionRequestLimitReached(
                    self.config.session_request_limit,
                )
                .into());
            };
        };

        let new = session_request::ActiveModel {
            requester_id: Set(user_id),
            addressee_id: Set(request.opponent_id.map(Into::into)),
            config: Set(request.config.to_bytes()?),
            public: Set(request.public),
            ..Default::default()
        };

        let created = new.insert(&txn).await?;
        txn.commit().await?;
        Ok(created)
    }
}
