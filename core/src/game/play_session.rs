use crate::error::{CoreError, CoreResult};
use crate::game::session_data::{SessionConfigData, SessionData};
use crate::types::session_id::SessionId;
use crate::types::user_id::UserId;
use giga_chess::prelude::Session;

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
