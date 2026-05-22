use crate::event::{AppEvent, ReconnectedEvent};
use crate::ws::cache::FetchableCache;
use crate::ws::fetchable::Fetchable;
use checkmade_core::types::user_id::UserId;
use checkmade_core::types::user_info::{PrivateUserInfo, PublicUserInfo};
use std::collections::HashMap;

pub struct Store {
    pub friends: Fetchable<HashMap<UserId, u64>>,
    pub incoming_friend_requests: Fetchable<HashMap<UserId, u64>>,
    pub me: Fetchable<PrivateUserInfo>,
    pub outgoing_friend_requests: Fetchable<HashMap<UserId, u64>>,
    pub users: FetchableCache<UserId, PublicUserInfo>,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            friends: Fetchable::new(),
            incoming_friend_requests: Fetchable::new(),
            me: Fetchable::new(),
            outgoing_friend_requests: Fetchable::new(),
            users: FetchableCache::new().with_fetch_cooldown(web_time::Duration::from_millis(200)),
        }
    }
}

impl Store {
    pub fn update(&mut self, ctx: &egui::Context, ws: &mut crate::ws::Ws) {
        if ReconnectedEvent::fired(ctx) {
            self.invalidate();
        }

        self.friends.request_if_needed(|| ws.request_friends());
        self.incoming_friend_requests
            .request_if_needed(|| ws.request_incoming_friend_requests());
        self.me.request_if_needed(|| ws.request_private_user_info());
        self.outgoing_friend_requests
            .request_if_needed(|| ws.request_outgoing_friend_requests());
        self.users.update(|id| ws.request_public_user_info(id));
    }

    fn invalidate(&mut self) {
        self.friends.invalidate();
        self.incoming_friend_requests.invalidate();
        self.outgoing_friend_requests.invalidate();
        self.me.invalidate();
        self.users.invalidate();
    }

    pub fn friend_request_count(&self) -> usize {
        self.incoming_friend_requests
            .value
            .as_ref()
            .map(|v| v.len())
            .unwrap_or(0)
    }
}
