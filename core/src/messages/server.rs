use crate::error::CoreResult;
use crate::game::play_move::PlayMove;
use crate::game::play_session::PlaySession;
use crate::types::friend_info::FriendInfo;
use crate::types::session_id::SessionId;
use crate::types::session_request::{SessionRequest, SessionRequestId};
use crate::types::user_id::UserId;
use crate::types::user_info::{PrivateUserInfo, PublicUserInfo};

#[derive(Clone)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub enum ServerMessage {
    ActiveSessions(Vec<PlaySession>),
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
    IncomingSessionRequests(Vec<SessionRequest>),
    OutgoingFriendRequests(Vec<FriendInfo>),
    OutgoingSessionRequests(Vec<SessionRequest>),
    Pong { client_time: u64, server_time: u64 },
    PrivateUserInfo(PrivateUserInfo),
    PublicSessionRequests(Vec<SessionRequest>),
    PublicUserInfo(PublicUserInfo),
    Session(PlaySession),
    SessionStart(PlaySession),
    SessionRequest(SessionRequest),
    SessionRequestCreateOk(SessionRequest),
    SessionRequestDeclinedByPeer(SessionRequestId),
    SessionRequestDeclineOk(SessionRequestId),
    SessionRequestIncoming(SessionRequest),
    SessionUpdate { session_id: SessionId, mv: PlayMove },
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
            Self::ActiveSessions(_) => "active_sessions",
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
            Self::IncomingSessionRequests(_) => "incoming_session_requests",
            Self::OutgoingFriendRequests(_) => "outgoing_friend_requests",
            Self::OutgoingSessionRequests(_) => "outgoing_session_requests",
            Self::Pong { .. } => "pong",
            Self::PublicSessionRequests(_) => "public_session_requests",
            Self::PrivateUserInfo(_) => "private_user_info",
            Self::PublicUserInfo(_) => "public_user_info",
            Self::Session(_) => "session",
            Self::SessionStart(_) => "session_start",
            Self::SessionRequest(_) => "session_request",
            Self::SessionRequestCreateOk(_) => "session_request_create_ok",
            Self::SessionRequestDeclinedByPeer(_) => "session_request_declined_by_peer",
            Self::SessionRequestDeclineOk(_) => "session_request_decline_ok",
            Self::SessionRequestIncoming(_) => "session_request_incoming",
            Self::SessionUpdate { .. } => "session_update",
        }
    }
}
