use crate::config::CoreConfig;
use crate::data::entity::session_request;
use crate::data::store::{Paginate, Store};
use crate::error::{CoreResult, DomainError};
use crate::types::session_request::CreateSessionRequest;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, EntityTrait,
    ModelTrait, Set, TransactionTrait,
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

    pub fn paginate_incoming(
        &self,
        user_id: Uuid,
        page_size: u64,
    ) -> Paginate<'_, session_request::Entity> {
        self.paginate()
            .filter(Condition::all().add(session_request::Column::AddresseeId.eq(Some(user_id))))
            .page_size(page_size)
    }

    pub fn paginate_public(&self, page_size: u64) -> Paginate<'_, session_request::Entity> {
        self.paginate()
            .filter(Condition::all().add(session_request::Column::AddresseeId.is_null()))
            .page_size(page_size)
    }

    pub fn paginate_outgoing(
        &self,
        user_id: Uuid,
        page_size: u64,
    ) -> Paginate<'_, session_request::Entity> {
        self.paginate()
            .filter(Condition::all().add(session_request::Column::RequesterId.eq(user_id)))
            .page_size(page_size)
    }

    pub async fn count_incoming<C: ConnectionTrait>(conn: &C, user_id: Uuid) -> CoreResult<u64> {
        Self::count_with(conn, session_request::Column::AddresseeId.eq(Some(user_id))).await
    }

    pub async fn count_outgoing<C: ConnectionTrait>(conn: &C, user_id: Uuid) -> CoreResult<u64> {
        Self::count_with(conn, session_request::Column::RequesterId.eq(user_id)).await
    }

    pub async fn create(
        &self,
        requester_id: Uuid,
        request: CreateSessionRequest,
    ) -> CoreResult<session_request::Model> {
        let txn = self.connection.begin().await?;

        if !request.public && request.opponent_id.is_none() {
            return Err(DomainError::PrivateSessionsRequireOpponent.into());
        }

        let outgoing = Self::count_outgoing(&txn, requester_id).await?;
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
            requester_id: Set(requester_id),
            addressee_id: Set(request.opponent_id.map(Into::into)),
            config: Set(request.config.to_bytes()?),
            public: Set(request.public),
            ..Default::default()
        };

        let created = new.insert(&txn).await?;
        txn.commit().await?;
        Ok(created)
    }

    /// Returns the user id of the original creator
    pub async fn decline(&self, opponent_id: Uuid, request_id: Uuid) -> CoreResult<Uuid> {
        let txn = self.connection.begin().await?;

        let Some(request) = session_request::Entity::find_by_id(request_id)
            .one(&txn)
            .await?
        else {
            return Err(DomainError::SessionRequestNotFound.into());
        };

        let Some(addressee_id) = request.addressee_id else {
            return Err(DomainError::SessionRequestNotFound.into());
        };

        if addressee_id != opponent_id {
            return Err(DomainError::SessionRequestNotFound.into());
        };

        let requester_id = request.requester_id;
        request.delete(&txn).await?;
        txn.commit().await?;
        Ok(requester_id)
    }

    /// Returns the user id of the original addressee
    pub async fn remove(&self, user_id: Uuid, request_id: Uuid) -> CoreResult<Option<Uuid>> {
        let txn = self.connection.begin().await?;
        let Some(request) = session_request::Entity::find_by_id(request_id)
            .one(&txn)
            .await?
        else {
            return Err(DomainError::SessionRequestNotFound.into());
        };
        if request.requester_id != user_id {
            return Err(DomainError::SessionRequestNotFound.into());
        };
        let addressee_id = request.addressee_id;
        request.delete(&txn).await?;
        txn.commit().await?;
        Ok(addressee_id)
    }
}
