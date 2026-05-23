use crate::error::{CoreError, CoreResult, DomainError};
use crate::game::play_move::PlayMove;
use crate::game::session_data::{SessionConfigData, SessionData};
use crate::types::session_id::SessionId;
use crate::types::user_id::UserId;
use giga_chess::prelude::{Color, Session};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlaySession {
    pub id: SessionId,
    pub active: bool,
    pub public: bool,
    pub white: UserId,
    pub black: UserId,
    pub created: u64,
    pub updated: u64,
    pub kind: PlaySessionKind,
}

impl PlaySession {
    pub fn play(&mut self, color: Color, mv: PlayMove, unix_ms: u64) -> CoreResult<()> {
        match &mut self.kind {
            PlaySessionKind::Normal(session) => {
                let PlayMove::Normal(action) = mv else {
                    return Err(DomainError::InvalidMove.into());
                };
                session
                    .act(color, action, unix_ms)
                    .map_err(DomainError::Session)?;
            }
        }
        Ok(())
    }

    pub fn is_over(&self) -> bool {
        match &self.kind {
            PlaySessionKind::Normal(session) => session.game().is_over(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PlaySessionKind {
    Normal(Session),
}

impl TryFrom<SessionData> for PlaySessionKind {
    type Error = CoreError;

    fn try_from(value: SessionData) -> CoreResult<Self> {
        match value {
            SessionData::Normal(normal) => Ok(PlaySessionKind::Normal(normal.try_into()?)),
        }
    }
}

impl From<PlaySessionKind> for SessionData {
    fn from(value: PlaySessionKind) -> Self {
        match value {
            PlaySessionKind::Normal(session) => SessionData::Normal(session.into()),
        }
    }
}

impl TryFrom<SessionConfigData> for PlaySessionKind {
    type Error = CoreError;

    fn try_from(config: SessionConfigData) -> CoreResult<Self> {
        match config {
            SessionConfigData::Normal(config) => Ok(PlaySessionKind::Normal(
                Session::from_config(&config)
                    .map_err(|err| CoreError::SessionCreation(err.to_string()))?,
            )),
        }
    }
}
