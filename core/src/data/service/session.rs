use crate::config::CoreConfig;
use crate::data::entity::{session, session_request};
use crate::data::store::{Page, Store};
use crate::data::Data;
use crate::error::{CoreError, CoreResult};
use crate::game::play_session::PlaySession;
use crate::game::session_data::{SessionConfigData, SessionData};
use crate::types::session_request::SessionRequest;
use crate::types::session_status::SessionStatus;
use sea_orm::Set;
use std::sync::Arc;
use uuid::Uuid;

pub struct SessionService {
    config: Arc<CoreConfig>,
    data: Arc<Data>,
}

impl SessionService {
    pub fn new(config: &Arc<CoreConfig>, data: &Arc<Data>) -> Self {
        Self {
            config: Arc::clone(config),
            data: Arc::clone(data),
        }
    }

    pub async fn load_by_id(&self, session_id: Uuid) -> CoreResult<Option<PlaySession>> {
        match self.data.session.find_by_id(session_id).await? {
            Some(model) => Ok(Some(self.load_session(model)?)),
            None => Ok(None),
        }
    }

    pub async fn user_page(
        &self,
        user_id: Uuid,
        status: Option<SessionStatus>,
        page_size: u64,
        page: u64,
    ) -> CoreResult<Page<PlaySession>> {
        self.load_session_page(
            self.data
                .session
                .paginate_by_user(user_id, status, page_size)
                .fetch_page(page)
                .await?,
        )
    }

    pub async fn public_page(
        &self,
        status: Option<SessionStatus>,
        page_size: u64,
        page: u64,
    ) -> CoreResult<Page<PlaySession>> {
        self.load_session_page(
            self.data
                .session
                .paginate_public(status, page_size)
                .fetch_page(page)
                .await?,
        )
    }

    pub fn load_session_page(&self, page: Page<session::Model>) -> CoreResult<Page<PlaySession>> {
        page.try_map(|model| self.load_session(model))
    }

    pub fn load_session(&self, model: session::Model) -> CoreResult<PlaySession> {
        PlaySession::try_from(model)
    }

    pub async fn load_request_by_id(&self, request_id: Uuid) -> CoreResult<Option<SessionRequest>> {
        match self.data.session_request.find_by_id(request_id).await? {
            Some(model) => Ok(Some(model.try_into()?)),
            None => Ok(None),
        }
    }

    pub async fn incoming_requests_page(
        &self,
        user_id: Uuid,
        page_size: u64,
        page: u64,
    ) -> CoreResult<Page<SessionRequest>> {
        self.data
            .session_request
            .paginate_incoming(user_id, page_size)
            .fetch_page(page)
            .await?
            .try_map(|model| model.try_into())
    }

    pub async fn public_requests_page(
        &self,
        page_size: u64,
        page: u64,
    ) -> CoreResult<Page<SessionRequest>> {
        self.data
            .session_request
            .paginate_public(page_size)
            .fetch_page(page)
            .await?
            .try_map(|model| model.try_into())
    }

    pub async fn outgoing_requests_page(
        &self,
        user_id: Uuid,
        page_size: u64,
        page: u64,
    ) -> CoreResult<Page<SessionRequest>> {
        self.data
            .session_request
            .paginate_outgoing(user_id, page_size)
            .fetch_page(page)
            .await?
            .try_map(|model| model.try_into())
    }
}

impl TryFrom<session::Model> for PlaySession {
    type Error = CoreError;
    fn try_from(model: session::Model) -> CoreResult<Self> {
        let data = SessionData::from_bytes(&model.data)?;
        Ok(Self {
            id: model.id.into(),
            public: model.public,
            white: model.white_id.into(),
            black: model.black_id.into(),
            created: model.created_at.and_utc().timestamp_millis() as u64,
            updated: model.updated_at.and_utc().timestamp_millis() as u64,
            kind: data.try_into()?,
        })
    }
}

impl TryFrom<PlaySession> for session::ActiveModel {
    type Error = CoreError;

    fn try_from(value: PlaySession) -> CoreResult<Self> {
        let status = value.status();
        let data: SessionData = value.kind.into();
        Ok(Self {
            status: Set(status as i16),
            data: Set(data.to_bytes()?),
            ..Default::default()
        })
    }
}

impl TryFrom<session_request::Model> for SessionRequest {
    type Error = CoreError;

    fn try_from(value: session_request::Model) -> CoreResult<Self> {
        Ok(Self {
            id: value.id.into(),
            requester_id: value.requester_id.into(),
            opponent_id: value.addressee_id.map(Into::into),
            config: SessionConfigData::from_bytes(&value.config)?,
            public: value.public,
            created: value.created_at.and_utc().timestamp_millis() as u64,
        })
    }
}
