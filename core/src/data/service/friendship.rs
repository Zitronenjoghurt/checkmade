use crate::config::CoreConfig;
use crate::data::entity::friendship;
use crate::data::store::{Page, Paginate, Store};
use crate::data::Data;
use crate::error::{CoreResult, DomainError};
use crate::types::friend_info::FriendInfo;
use crate::types::friendship_status::FriendshipStatus;
use sea_orm::{ColumnTrait, ExprTrait};
use sea_orm::{IntoActiveModel, Set};
use std::sync::Arc;
use uuid::Uuid;

pub struct FriendshipService {
    config: Arc<CoreConfig>,
    data: Arc<Data>,
}

impl FriendshipService {
    pub fn new(config: &Arc<CoreConfig>, data: &Arc<Data>) -> Self {
        Self {
            config: Arc::clone(config),
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
        let requester_friends = self.data.friends.count_friends(source_id).await?;
        if requester_friends >= self.config.friend_limit {
            return Err(DomainError::FriendLimitReached(self.config.friend_limit).into());
        }

        let addressee_friends = self.data.friends.count_friends(target_id).await?;
        if addressee_friends >= self.config.friend_limit {
            return Err(DomainError::FriendLimitReached(self.config.friend_limit).into());
        }

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

    pub async fn remove_request(&self, source_id: Uuid, target_id: Uuid) -> CoreResult<()> {
        let success = self
            .data
            .friends
            .delete_by_id((source_id, target_id))
            .await?;
        if success.rows_affected == 0 {
            return Err(DomainError::NoFriendRequest.into());
        }
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

    pub async fn friends_with_stats(
        &self,
        user_id: Uuid,
        page_size: u64,
        page: u64,
    ) -> CoreResult<Page<FriendInfo>> {
        self.data
            .friends
            .paginate_friends_with_stats(user_id, page, page_size)
            .await
            .map(|p| {
                p.map(|fs| {
                    let friend_id = if fs.requester_id == user_id {
                        fs.addressee_id
                    } else {
                        fs.requester_id
                    };
                    FriendInfo {
                        user_id: friend_id.into(),
                        since: fs.created_at.and_utc().timestamp_millis() as u64,
                        times_won: fs.times_won as u64,
                        times_lost: fs.times_lost as u64,
                        times_drawn: fs.times_drawn as u64,
                    }
                })
            })
    }

    pub async fn friend_info(
        &self,
        user_id: Uuid,
        friend_id: Uuid,
        since: u64,
    ) -> CoreResult<FriendInfo> {
        let (won, lost, drawn) = self.data.friends.pair_stats(user_id, friend_id).await?;
        Ok(FriendInfo {
            user_id: friend_id.into(),
            since,
            times_won: won as u64,
            times_lost: lost as u64,
            times_drawn: drawn as u64,
        })
    }
}
