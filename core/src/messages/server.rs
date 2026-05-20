use crate::error::CoreResult;
use crate::types::user_info::PrivateUserInfo;

#[derive(Clone)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub enum ServerMessage {
    Pong { client_time: u64, server_time: u64 },
    Error(String),
    UserInfo(PrivateUserInfo),
}

impl ServerMessage {
    #[cfg(feature = "bitcode")]
    pub fn as_bytes(&self) -> Vec<u8> {
        bitcode::encode(self)
    }

    #[cfg(feature = "bitcode")]
    pub fn from_bytes(bytes: &[u8]) -> CoreResult<Self> {
        Ok(bitcode::decode(bytes)?)
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Pong { .. } => "pong",
            Self::Error(_) => "error",
            Self::UserInfo(_) => "user_info",
        }
    }
}
