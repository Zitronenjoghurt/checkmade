use crate::data::entity::friendship;
use crate::error::{CoreError, CoreResult};
use crate::types::friendship_status::FriendshipStatus;

impl friendship::Model {
    pub fn status(&self) -> CoreResult<FriendshipStatus> {
        FriendshipStatus::from_repr(self.status)
            .ok_or(CoreError::InvalidFriendshipStatus(self.status))
    }
}
