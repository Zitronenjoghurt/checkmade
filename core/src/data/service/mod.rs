use crate::data::Data;
use std::sync::Arc;

pub mod friendship;
mod user;

pub struct Services {
    pub friends: friendship::FriendshipService,
    pub user: user::UserService,
}

impl Services {
    pub fn new(data: &Arc<Data>) -> Self {
        Self {
            friends: friendship::FriendshipService::new(data),
            user: user::UserService::new(data),
        }
    }
}
