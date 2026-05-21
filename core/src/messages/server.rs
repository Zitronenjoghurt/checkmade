use crate::error::CoreResult;
use crate::types::friend_info::FriendInfo;
use crate::types::user_id::UserId;
use crate::types::user_info::{PrivateUserInfo, PublicUserInfo};

#[derive(Clone)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub enum ServerMessage {
    Error(String),
    FriendRequestIncoming(FriendInfo),
    FriendRequestDeclinedByPeer(UserId),
    FriendshipEstablished(FriendInfo),
    FriendshipRemovedByPeer(UserId),
    FriendRequestSendOk(FriendInfo),
    FriendRequestDeclineOk(UserId),
    FriendRemoveOk(UserId),
    Friends(Vec<FriendInfo>),
    IncomingFriendRequests(Vec<FriendInfo>),
    OutgoingFriendRequests(Vec<FriendInfo>),
    Pong { client_time: u64, server_time: u64 },
    PrivateUserInfo(PrivateUserInfo),
    PublicUserInfo(PublicUserInfo),
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
            Self::Error(_) => "error",
            Self::FriendRequestIncoming(_) => "friend_request_incoming",
            Self::FriendRequestDeclinedByPeer(_) => "friend_request_declined_by_peer",
            Self::FriendshipEstablished(_) => "friendship_established",
            Self::FriendshipRemovedByPeer(_) => "friendship_removed_by_peer",
            Self::FriendRequestSendOk(_) => "friend_request_send_ok",
            Self::FriendRequestDeclineOk(_) => "friend_request_decline_ok",
            Self::FriendRemoveOk(_) => "friend_remove_ok",
            Self::Friends(_) => "friends",
            Self::IncomingFriendRequests(_) => "incoming_friend_requests",
            Self::OutgoingFriendRequests(_) => "outgoing_friend_requests",
            Self::Pong { .. } => "pong",
            Self::PrivateUserInfo(_) => "private_user_info",
            Self::PublicUserInfo(_) => "public_user_info",
        }
    }
}
