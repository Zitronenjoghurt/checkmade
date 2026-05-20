use crate::data::Data;
use std::sync::Arc;

mod user;

pub struct Services {
    pub user: user::UserService,
}

impl Services {
    pub fn new(data: &Arc<Data>) -> Self {
        Self {
            user: user::UserService::new(data),
        }
    }
}
