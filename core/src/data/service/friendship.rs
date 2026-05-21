use crate::data::entity::friendship;
use crate::data::store::{Paginate, Store};
use crate::data::Data;
use crate::error::{CoreResult, DomainError};
use crate::types::friendship_status::FriendshipStatus;
use sea_orm::{ColumnTrait, ExprTrait};
use sea_orm::{IntoActiveModel, Set};
use std::sync::Arc;
use uuid::Uuid;

const FRIEND_CODE_ALPHABET: &[char] = &[
    '2', '3', '4', '6', '7', '8', '9', 'B', 'D', 'F', 'G', 'H', 'K', 'M', 'P', 'R', 'T', 'X',
];
const FRIEND_CODE_LENGTH: usize = 9;

pub fn generate_friend_code() -> String {
    nanoid::nanoid!(FRIEND_CODE_LENGTH, FRIEND_CODE_ALPHABET)
}

pub struct FriendshipService {
    data: Arc<Data>,
}

impl FriendshipService {
    pub fn new(data: &Arc<Data>) -> Self {
        Self {
            data: Arc::clone(data),
        }
    }
}

impl FriendshipService {
    pub async fn send_request(&self, source_id: Uuid, code: &str) -> CoreResult<friendship::Model> {
        let target = self
            .data
            .user
            .find_by_friend_code(code)
            .await?
            .ok_or(DomainError::InvalidFriendCode)?;

        if source_id == target.id {
            return Err(DomainError::FriendRequestBlocked.into());
        }

        match self
            .data
            .friends
            .find_by_user_ids(source_id, target.id)
            .await?
        {
            Some(fs) => match fs.status()? {
                FriendshipStatus::Pending => {
                    if fs.requester_id == source_id {
                        Err(DomainError::AlreadySentFriendRequest.into())
                    } else {
                        Err(DomainError::AlreadyReceivedFriendRequest.into())
                    }
                }
                FriendshipStatus::Accepted => Err(DomainError::AlreadyFriends.into()),
                FriendshipStatus::Blocked => {
                    if fs.addressee_id == source_id {
                        Ok(self.data.friends.create(source_id, target.id).await?)
                    } else {
                        Err(DomainError::FriendRequestBlocked.into())
                    }
                }
            },
            None => Ok(self.data.friends.create(source_id, target.id).await?),
        }
    }

    pub async fn accept_request(
        &self,
        source_id: Uuid,
        target_id: Uuid,
    ) -> CoreResult<friendship::Model> {
        let pending = self.pending_request(source_id, target_id).await?;
        let mut active = pending.into_active_model();
        active.status = Set(FriendshipStatus::Accepted as i16);
        self.data.friends.update(active).await
    }

    pub async fn reject_request(&self, source_id: Uuid, target_id: Uuid) -> CoreResult<()> {
        let pending = self.pending_request(source_id, target_id).await?;

        let mut active = pending.into_active_model();
        active.status = Set(FriendshipStatus::Blocked as i16);
        self.data.friends.update(active).await?;

        Ok(())
    }

    async fn pending_request(
        &self,
        source_id: Uuid,
        target_id: Uuid,
    ) -> CoreResult<friendship::Model> {
        let Some(existing) = self
            .data
            .friends
            .find_by_user_ids(source_id, target_id)
            .await?
        else {
            return Err(DomainError::NoFriendRequest.into());
        };

        if !matches!(existing.status()?, FriendshipStatus::Pending) {
            return Err(DomainError::NoFriendRequest.into());
        };

        if source_id == existing.requester_id {
            return Err(DomainError::NoFriendRequest.into());
        };

        Ok(existing)
    }

    pub async fn remove_friend(&self, source_id: Uuid, target_id: Uuid) -> CoreResult<()> {
        let Some(existing) = self
            .data
            .friends
            .find_by_user_ids_with_status(source_id, target_id, FriendshipStatus::Accepted)
            .await?
        else {
            return Err(DomainError::NotFriends.into());
        };

        self.data
            .friends
            .delete_by_id((existing.requester_id, existing.addressee_id))
            .await?;

        Ok(())
    }

    pub fn paginate_received_requests(
        &self,
        id: Uuid,
        page_size: u64,
    ) -> Paginate<'_, friendship::Entity> {
        self.data.friends.paginate_by_user(id, page_size).filter(
            friendship::Column::Status
                .eq(FriendshipStatus::Pending as i16)
                .and(friendship::Column::AddresseeId.eq(id)),
        )
    }

    pub fn paginate_sent_requests(
        &self,
        id: Uuid,
        page_size: u64,
    ) -> Paginate<'_, friendship::Entity> {
        self.data.friends.paginate_by_user(id, page_size).filter(
            friendship::Column::Status
                .eq(FriendshipStatus::Pending as i16)
                .and(friendship::Column::RequesterId.eq(id)),
        )
    }

    pub fn paginate_friends(&self, id: Uuid, page_size: u64) -> Paginate<'_, friendship::Entity> {
        self.data
            .friends
            .paginate_by_user(id, page_size)
            .filter(friendship::Column::Status.eq(FriendshipStatus::Accepted as i16))
    }
}
