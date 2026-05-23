use crate::game::session_data::SessionConfigData;
use crate::types::user_id::UserId;

#[derive(Clone)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SessionRequest {
    pub opponent_id: Option<UserId>,
    pub config: SessionConfigData,
    pub public: bool,
}
