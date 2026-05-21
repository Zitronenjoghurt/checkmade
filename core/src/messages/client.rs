use crate::error::CoreResult;
use crate::types::user_id::UserId;

#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub enum ClientMessage {
    AcceptFriendRequest(UserId),
    DeclineFriendRequest(UserId),
    Friends,
    IncomingFriendRequests,
    OutgoingFriendRequests,
    Ping { client_time: u64 },
    PrivateUserInfo,
    PublicUserInfo(UserId),
    RemoveFriend(UserId),
    SendFriendRequest { friend_code: String },
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
            Self::AcceptFriendRequest(_) => 3.0,
            Self::DeclineFriendRequest(_) => 3.0,
            Self::Friends => 12.0,
            Self::IncomingFriendRequests => 12.0,
            Self::OutgoingFriendRequests => 12.0,
            Self::Ping { .. } => 1.0,
            Self::PrivateUserInfo => 3.0,
            Self::PublicUserInfo(_) => 3.0,
            Self::RemoveFriend(_) => 3.0,
            Self::SendFriendRequest { .. } => 3.0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::AcceptFriendRequest(_) => "accept_friend_request",
            Self::DeclineFriendRequest(_) => "decline_friend_request",
            Self::Friends => "friends",
            Self::IncomingFriendRequests => "incoming_friend_requests",
            Self::OutgoingFriendRequests => "outgoing_friend_requests",
            Self::Ping { .. } => "ping",
            Self::PrivateUserInfo => "private_user_info",
            Self::PublicUserInfo(_) => "public_user_info",
            Self::RemoveFriend(_) => "remove_friend",
            Self::SendFriendRequest { .. } => "send_friend_request",
        }
    }
}
