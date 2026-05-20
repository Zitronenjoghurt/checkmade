use crate::error::CoreResult;

#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub enum ClientMessage {
    Ping { client_time: u64 },
    UserInfo,
}

impl ClientMessage {
    #[cfg(feature = "bitcode")]
    pub fn as_bytes(&self) -> Vec<u8> {
        bitcode::encode(self)
    }

    #[cfg(feature = "bitcode")]
    pub fn from_bytes(bytes: &[u8]) -> CoreResult<Self> {
        Ok(bitcode::decode(bytes)?)
    }

    pub fn cost(&self) -> f64 {
        match self {
            Self::Ping { .. } => 1.0,
            Self::UserInfo => 3.0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Ping { .. } => "ping",
            Self::UserInfo => "user_info",
        }
    }
}
