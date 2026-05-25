use crate::event::{AppEvent, ReconnectedEvent};
use crate::ws::cache::FetchableCache;
use crate::ws::fetchable::Fetchable;
use crate::ws::Ws;
use checkmade_core::game::play_session::PlaySession;
use checkmade_core::giga_chess::prelude::{Color, Piece};
use checkmade_core::types::friend_info::FriendInfo;
use checkmade_core::types::session_id::SessionId;
use checkmade_core::types::session_request::{SessionRequest, SessionRequestId};
use checkmade_core::types::user_id::UserId;
use checkmade_core::types::user_info::{PrivateUserInfo, PublicUserInfo};
use std::collections::HashMap;

pub struct Store {
    pub sessions: Fetchable<HashMap<SessionId, PlaySession>>,
    pub friends: Fetchable<HashMap<UserId, FriendInfo>>,
    pub incoming_friend_requests: Fetchable<HashMap<UserId, u64>>,
    pub incoming_session_requests: Fetchable<HashMap<SessionRequestId, SessionRequest>>,
    pub me: Fetchable<PrivateUserInfo>,
    pub outgoing_friend_requests: Fetchable<HashMap<UserId, u64>>,
    pub outgoing_session_requests: Fetchable<HashMap<SessionRequestId, SessionRequest>>,
    pub public_session_requests: Fetchable<HashMap<SessionRequestId, SessionRequest>>,
    pub users: FetchableCache<UserId, PublicUserInfo>,
    has_session_history: bool,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            sessions: Fetchable::new(),
            friends: Fetchable::new(),
            incoming_friend_requests: Fetchable::new(),
            incoming_session_requests: Fetchable::new(),
            me: Fetchable::new(),
            outgoing_friend_requests: Fetchable::new(),
            outgoing_session_requests: Fetchable::new(),
            public_session_requests: Fetchable::new(),
            users: FetchableCache::new().with_fetch_cooldown(web_time::Duration::from_millis(200)),
            has_session_history: false,
        }
    }
}

impl Store {
    pub fn update(&mut self, ctx: &egui::Context, ws: &mut crate::ws::Ws) {
        if ReconnectedEvent::fired(ctx) {
            self.invalidate();
        }

        self.sessions
            .request_if_needed(|| ws.request_active_sessions());
        self.friends.request_if_needed(|| ws.request_friends());
        self.incoming_friend_requests
            .request_if_needed(|| ws.request_incoming_friend_requests());
        self.incoming_session_requests
            .request_if_needed(|| ws.request_incoming_session_requests());
        self.me.request_if_needed(|| ws.request_private_user_info());
        self.outgoing_friend_requests
            .request_if_needed(|| ws.request_outgoing_friend_requests());
        self.outgoing_session_requests
            .request_if_needed(|| ws.request_outgoing_session_requests());
        self.public_session_requests
            .request_if_needed(|| ws.request_public_session_requests());
        self.users.update(|id| ws.request_public_user_info(id));
    }

    fn invalidate(&mut self) {
        self.sessions.invalidate();
        self.friends.invalidate();
        self.incoming_friend_requests.invalidate();
        self.incoming_session_requests.invalidate();
        self.me.invalidate();
        self.outgoing_friend_requests.invalidate();
        self.outgoing_session_requests.invalidate();
        self.public_session_requests.invalidate();
        self.users.invalidate();
    }

    pub fn ensure_session_history(&mut self, ws: &mut Ws) {
        if !self.has_session_history {
            ws.request_session_history();
            self.has_session_history = true;
        }
    }
}

// Counts
impl Store {
    pub fn friend_request_count(&self) -> usize {
        self.incoming_friend_requests
            .value
            .as_ref()
            .map(|v| v.len())
            .unwrap_or(0)
    }

    pub fn game_request_count(&self) -> usize {
        self.incoming_session_requests
            .value
            .as_ref()
            .map(|v| v.len())
            .unwrap_or(0)
    }

    pub fn active_sessions_to_move_count(&self) -> usize {
        let Some(me) = self.me.value.as_ref() else {
            return 0;
        };
        self.sessions
            .value
            .as_ref()
            .map(|v| v.values().filter(|s| s.can_move(me.public.id)).count())
            .unwrap_or(0)
    }
}

// Session helpers
impl Store {
    pub fn session_captured_pieces(&self, session_id: SessionId, color: Color) -> &[Piece] {
        let Some(session) = self.sessions.get_entry(&session_id) else {
            return &[];
        };
        session.captured_pieces(color)
    }
}
