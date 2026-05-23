use crate::error::CoreResult;
use crate::game::play_move::PlayMove;
use crate::types::session_id::SessionId;
use crate::types::session_request::{CreateSessionRequest, SessionRequestId};
use crate::types::user_id::UserId;

#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub enum ClientMessage {
    AcceptFriendRequest(UserId),
    AcceptSessionRequest(SessionRequestId),
    ActiveSessions,
    CreateSessionRequest(Box<CreateSessionRequest>),
    DeclineFriendRequest(UserId),
    DeclineSessionRequest(SessionRequestId),
    Friends,
    IncomingFriendRequests,
    IncomingSessionRequests,
    OutgoingFriendRequests,
    OutgoingSessionRequests,
    Ping { client_time: u64 },
    PlayMove { session_id: SessionId, mv: PlayMove },
    PrivateUserInfo,
    PublicSessionRequests,
    PublicUserInfo(UserId),
    RemoveFriend(UserId),
    SendFriendRequest { friend_code: String },
    Session(SessionId),
    SessionRequest(SessionRequestId),
    SubscribeSession(SessionId),
    UnsubscribeSession(SessionId),
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
            Self::AcceptSessionRequest(_) => 3.0,
            Self::ActiveSessions => 6.0,
            Self::CreateSessionRequest(_) => 3.0,
            Self::DeclineFriendRequest(_) => 3.0,
            Self::DeclineSessionRequest(_) => 3.0,
            Self::Friends => 12.0,
            Self::IncomingFriendRequests => 12.0,
            Self::IncomingSessionRequests => 9.0,
            Self::OutgoingFriendRequests => 12.0,
            Self::OutgoingSessionRequests => 9.0,
            Self::Ping { .. } => 1.0,
            Self::PlayMove { .. } => 15.0,
            Self::PrivateUserInfo => 3.0,
            Self::PublicUserInfo(_) => 3.0,
            Self::PublicSessionRequests => 9.0,
            Self::RemoveFriend(_) => 3.0,
            Self::SendFriendRequest { .. } => 3.0,
            Self::Session(_) => 3.0,
            Self::SessionRequest(_) => 3.0,
            Self::SubscribeSession(_) => 3.0,
            Self::UnsubscribeSession(_) => 3.0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::AcceptFriendRequest(_) => "accept_friend_request",
            Self::AcceptSessionRequest(_) => "accept_session_request",
            Self::ActiveSessions => "active_sessions",
            Self::CreateSessionRequest(_) => "create_session_request",
            Self::DeclineFriendRequest(_) => "decline_friend_request",
            Self::DeclineSessionRequest(_) => "decline_session_request",
            Self::Friends => "friends",
            Self::IncomingFriendRequests => "incoming_friend_requests",
            Self::IncomingSessionRequests => "incoming_session_requests",
            Self::OutgoingFriendRequests => "outgoing_friend_requests",
            Self::OutgoingSessionRequests => "outgoing_session_requests",
            Self::Ping { .. } => "ping",
            Self::PlayMove { .. } => "play_move",
            Self::PrivateUserInfo => "private_user_info",
            Self::PublicUserInfo(_) => "public_user_info",
            Self::PublicSessionRequests => "public_session_requests",
            Self::RemoveFriend(_) => "remove_friend",
            Self::SendFriendRequest { .. } => "send_friend_request",
            Self::Session(_) => "session",
            Self::SessionRequest(_) => "session_request",
            Self::SubscribeSession(_) => "subscribe_session",
            Self::UnsubscribeSession(_) => "unsubscribe_session",
        }
    }
}
