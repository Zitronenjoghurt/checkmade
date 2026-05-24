use crate::config::CoreConfig;
use crate::data::entity::{session, session_request};
use crate::data::store::{Paginate, Store};
use crate::error::{CoreResult, DomainError};
use crate::game::play_move::PlayMove;
use crate::game::play_session::{PlaySession, PlaySessionKind};
use crate::game::session_data::{SessionConfigData, SessionData};
use crate::types::session_status::SessionStatus;
use giga_chess::prelude::Color;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, ModelTrait, Set, TransactionTrait,
};
use sea_orm::{Condition, DatabaseConnection};
use std::sync::Arc;
use uuid::Uuid;

pub struct SessionStore {
    config: Arc<CoreConfig>,
    connection: DatabaseConnection,
}

impl Store for SessionStore {
    type Entity = session::Entity;
    type ActiveModel = session::ActiveModel;

    fn db(&self) -> &DatabaseConnection {
        &self.connection
    }
}

impl SessionStore {
    pub fn new(config: &Arc<CoreConfig>, connection: &DatabaseConnection) -> Self {
        Self {
            config: Arc::clone(config),
            connection: connection.clone(),
        }
    }

    pub fn paginate_by_user(
        &self,
        id: Uuid,
        status: Option<SessionStatus>,
        page_size: u64,
    ) -> Paginate<'_, session::Entity> {
        let mut filter = Condition::all().add(
            Condition::any()
                .add(session::Column::WhiteId.eq(id))
                .add(session::Column::BlackId.eq(id)),
        );
        if let Some(status) = status {
            filter = filter.add(session::Column::Status.eq(status as i16));
        };
        self.paginate().filter(filter).page_size(page_size)
    }

    pub fn paginate_public(
        &self,
        status: Option<SessionStatus>,
        page_size: u64,
    ) -> Paginate<'_, session::Entity> {
        let mut filter = Condition::all().add(session::Column::Public.eq(true));
        if let Some(status) = status {
            filter = filter.add(session::Column::Status.eq(status as i16));
        };
        self.paginate().filter(filter).page_size(page_size)
    }

    pub async fn count_by_user<C: ConnectionTrait>(conn: &C, id: Uuid) -> CoreResult<u64> {
        Self::count_with(
            conn,
            Condition::any()
                .add(session::Column::WhiteId.eq(id))
                .add(session::Column::BlackId.eq(id)),
        )
        .await
    }

    pub async fn create(&self, opponent_id: Uuid, request_id: Uuid) -> CoreResult<session::Model> {
        let txn = self.connection.begin().await?;
        let Some(request) = session_request::Entity::find_by_id(request_id)
            .one(&txn)
            .await?
        else {
            return Err(DomainError::SessionRequestNotFound.into());
        };

        if let Some(addressee_id) = request.addressee_id
            && addressee_id != opponent_id
        {
            return Err(DomainError::SessionRequestNotFound.into());
        }

        let requester_count = Self::count_by_user(&txn, request.requester_id).await?;
        if requester_count >= self.config.session_limit {
            return Err(DomainError::SessionLimitReached(self.config.session_limit).into());
        };

        let opponent_count = Self::count_by_user(&txn, opponent_id).await?;
        if opponent_count >= self.config.session_limit {
            return Err(DomainError::SessionLimitReached(self.config.session_limit).into());
        };

        let (white_id, black_id) = if fastrand::bool() {
            (request.requester_id, opponent_id)
        } else {
            (opponent_id, request.requester_id)
        };

        let config = SessionConfigData::from_bytes(&request.config)?;
        let play_session_kind: PlaySessionKind = config.try_into()?;
        let session_data: SessionData = play_session_kind.into();

        let model = session::ActiveModel {
            public: Set(request.public),
            white_id: Set(white_id),
            black_id: Set(black_id),
            data: Set(session_data.to_bytes()?),
            ..Default::default()
        };

        request.delete(&txn).await?;

        let created = model.insert(&txn).await?;
        txn.commit().await?;
        Ok(created)
    }

    pub async fn play(
        &self,
        player_id: Uuid,
        session_id: Uuid,
        play_move: PlayMove,
        server_time: u64,
    ) -> CoreResult<session::Model> {
        let txn = self.connection.begin().await?;
        let model = session::Entity::find_by_id(session_id)
            .one(&txn)
            .await?
            .ok_or(DomainError::SessionNotFound)?;
        let mut session: PlaySession = model.try_into()?;

        let color = if session.white == player_id.into() {
            Color::White
        } else if session.black == player_id.into() {
            Color::Black
        } else {
            return Err(DomainError::NotSessionParticipant.into());
        };

        session.play(color, play_move, server_time)?;

        let to_save: session::ActiveModel = session.try_into()?;
        let model = to_save.update(&txn).await?;
        txn.commit().await?;
        Ok(model)
    }
}
