use crate::game::session_data::SessionConfigData;
use crate::types::user_id::UserId;
use uuid::Uuid;

#[derive(Clone)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SessionRequest {
    pub id: SessionRequestId,
    pub requester_id: UserId,
    pub opponent_id: Option<UserId>,
    pub config: SessionConfigData,
    pub public: bool,
    pub created: u64,
}

#[derive(Clone)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateSessionRequest {
    pub requester_id: UserId,
    pub opponent_id: Option<UserId>,
    pub config: SessionConfigData,
    pub public: bool,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SessionRequestId([u8; 16]);

impl From<Uuid> for SessionRequestId {
    fn from(value: Uuid) -> Self {
        Self(value.into_bytes())
    }
}

impl From<SessionRequestId> for Uuid {
    fn from(value: SessionRequestId) -> Self {
        Self::from_bytes(value.0)
    }
}
