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
    pub fn update(&mut self, ws: &mut crate::ws::Ws) {
        self.friends.request_if_needed(|| ws.request_friends());
        self.incoming_friend_requests
            .request_if_needed(|| ws.request_incoming_friend_requests());
        self.me.request_if_needed(|| ws.request_private_user_info());
        self.outgoing_friend_requests
            .request_if_needed(|| ws.request_outgoing_friend_requests());
        self.users.update(|id| ws.request_public_user_info(id));
    }
}
