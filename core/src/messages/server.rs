use crate::error::CoreResult;
use crate::game::play_move::PlayMove;
use crate::game::play_session::PlaySession;
use crate::types::friend_info::{FriendInfo, FriendRequestInfo};
use crate::types::session_id::SessionId;
use crate::types::session_request::{SessionRequest, SessionRequestId};
use crate::types::user_id::UserId;
use crate::types::user_info::{PrivateUserInfo, PublicUserInfo};
use giga_chess::prelude::Color;

#[derive(Clone)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub enum ServerMessage {
    ActiveSessions(Vec<PlaySession>),
    Error(String),
    FriendRequestIncoming(FriendRequestInfo),
    FriendRequestDeclinedByPeer(UserId),
    FriendshipEstablished(FriendInfo),
    FriendshipRemovedByPeer(UserId),
    FriendRequestSendOk(FriendRequestInfo),
    FriendRequestDeclineOk(UserId),
    FriendRequestRemoveOk(UserId),
    FriendRequestRemovedByPeer(UserId),
    FriendRemoveOk(UserId),
    Friends(Vec<FriendInfo>),
    IncomingFriendRequests(Vec<FriendRequestInfo>),
    IncomingSessionRequests(Vec<SessionRequest>),
    OutgoingFriendRequests(Vec<FriendRequestInfo>),
    OutgoingSessionRequests(Vec<SessionRequest>),
    Pong {
        client_time: u64,
        server_time: u64,
    },
    PrivateUserInfo(PrivateUserInfo),
    PublicSessionRequests(Vec<SessionRequest>),
    PublicUserInfo(PublicUserInfo),
    Session(PlaySession),
    Sessions(Vec<PlaySession>),
    SessionStart {
        session: PlaySession,
        request_id: SessionRequestId,
    },
    SessionRequest(SessionRequest),
    SessionRequestCreateOk(SessionRequest),
    SessionRequestDeclinedByPeer(SessionRequestId),
    SessionRequestDeclineOk(SessionRequestId),
    SessionRequestIncoming(SessionRequest),
    SessionRequestRemoveOk(SessionRequestId),
    SessionRequestRemovedByPeer(SessionRequestId),
    SessionUpdate {
        session_id: SessionId,
        color: Color,
        mv: PlayMove,
        at: u64,
    },
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
            Self::FriendRequestRemoveOk(_) => "friend_request_remove_ok",
            Self::FriendRequestRemovedByPeer(_) => "friend_request_remove_by_peer",
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
            Self::Sessions(_) => "sessions",
            Self::SessionStart { .. } => "session_start",
            Self::SessionRequest(_) => "session_request",
            Self::SessionRequestCreateOk(_) => "session_request_create_ok",
            Self::SessionRequestDeclinedByPeer(_) => "session_request_declined_by_peer",
            Self::SessionRequestDeclineOk(_) => "session_request_decline_ok",
            Self::SessionRequestIncoming(_) => "session_request_incoming",
            Self::SessionRequestRemoveOk(_) => "session_request_remove_ok",
            Self::SessionRequestRemovedByPeer(_) => "session_request_removed_by_peer",
            Self::SessionUpdate { .. } => "session_update",
        }
    }
}
