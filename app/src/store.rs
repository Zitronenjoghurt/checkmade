use crate::ws::fetchable::Fetchable;
use checkmade_core::types::user_info::PrivateUserInfo;

#[derive(Default)]
pub struct Store {
    pub user_info: Fetchable<PrivateUserInfo>,
}

impl Store {
    pub fn update(&mut self, ws: &mut crate::ws::Ws) {
        self.user_info.request_if_needed(|| ws.request_user_info());
    }
}
