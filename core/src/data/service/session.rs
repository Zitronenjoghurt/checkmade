use crate::config::CoreConfig;
use crate::data::entity::session;
use crate::data::store::{Page, Store};
use crate::data::Data;
use crate::error::{CoreError, CoreResult};
use crate::game::play_session::PlaySession;
use crate::game::session_data::SessionData;
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
        active: Option<bool>,
        page_size: u64,
        page: u64,
    ) -> CoreResult<Page<PlaySession>> {
        self.load_session_page(
            self.data
                .session
                .paginate_by_user(user_id, active, page_size)
                .fetch_page(page)
                .await?,
        )
    }

    pub async fn public_page(
        &self,
        active: Option<bool>,
        page_size: u64,
        page: u64,
    ) -> CoreResult<Page<PlaySession>> {
        self.load_session_page(
            self.data
                .session
                .paginate_public(active, page_size)
                .fetch_page(page)
                .await?,
        )
    }

    fn load_session_page(&self, page: Page<session::Model>) -> CoreResult<Page<PlaySession>> {
        page.try_map(|model| self.load_session(model))
    }

    fn load_session(&self, model: session::Model) -> CoreResult<PlaySession> {
        PlaySession::try_from(model)
    }
}

impl TryFrom<session::Model> for PlaySession {
    type Error = CoreError;
    fn try_from(model: session::Model) -> CoreResult<Self> {
        let data = SessionData::from_bytes(&model.data)?;
        Ok(Self {
            id: model.id.into(),
            active: model.active,
            public: model.public,
            white: model.white_id.into(),
            black: model.black_id.into(),
            created: model.created_at.and_utc().timestamp_millis() as u64,
            updated: model.updated_at.and_utc().timestamp_millis() as u64,
            kind: data.try_into()?,
        })
    }
}
