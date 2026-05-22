use crate::config::CoreConfig;
use crate::data::entity::friendship;
use crate::data::store::{Paginate, Store};
use crate::error::{CoreError, CoreResult, DomainError};
use crate::types::friendship_status::FriendshipStatus;
use futures::stream::BoxStream;
use futures::StreamExt;
use sea_orm::sea_query::IntoCondition;
use sea_orm::{ColumnTrait, ExprTrait, Set};
use sea_orm::{Condition, DatabaseConnection, EntityTrait};
use sea_orm::{QueryFilter, QuerySelect};
use std::sync::Arc;
use uuid::Uuid;

pub struct FriendshipStore {
    config: Arc<CoreConfig>,
    connection: DatabaseConnection,
}

impl Store for FriendshipStore {
    type Entity = friendship::Entity;
    type ActiveModel = friendship::ActiveModel;

    fn db(&self) -> &DatabaseConnection {
        &self.connection
    }
}

impl FriendshipStore {
    pub fn new(config: &Arc<CoreConfig>, connection: &DatabaseConnection) -> Self {
        Self {
            config: Arc::clone(config),
            connection: connection.clone(),
        }
    }

    pub async fn find_by_user_ids(
        &self,
        id_a: Uuid,
        id_b: Uuid,
    ) -> CoreResult<Option<friendship::Model>> {
        self.find_one_by(
            Condition::any()
                .add(
                    Condition::all()
                        .add(friendship::Column::RequesterId.eq(id_a))
                        .add(friendship::Column::AddresseeId.eq(id_b)),
                )
                .add(
                    Condition::all()
                        .add(friendship::Column::RequesterId.eq(id_b))
                        .add(friendship::Column::AddresseeId.eq(id_a)),
                ),
        )
        .await
    }

    pub async fn find_by_user_ids_with_status(
        &self,
        id_a: Uuid,
        id_b: Uuid,
        status: FriendshipStatus,
    ) -> CoreResult<Option<friendship::Model>> {
        self.find_one_by(
            Condition::all()
                .add(
                    Condition::any()
                        .add(
                            Condition::all()
                                .add(friendship::Column::RequesterId.eq(id_a))
                                .add(friendship::Column::AddresseeId.eq(id_b)),
                        )
                        .add(
                            Condition::all()
                                .add(friendship::Column::RequesterId.eq(id_b))
                                .add(friendship::Column::AddresseeId.eq(id_a)),
                        ),
                )
                .add(friendship::Column::Status.eq(status as i16)),
        )
        .await
    }

    pub async fn create(&self, requester: Uuid, addressee: Uuid) -> CoreResult<friendship::Model> {
        let requester_friends = self.count_friends(requester).await?;
        if requester_friends >= self.config.friend_limit {
            return Err(DomainError::FriendLimitReached(self.config.friend_limit).into());
        }

        let addressee_friends = self.count_friends(addressee).await?;
        if addressee_friends >= self.config.friend_limit {
            return Err(DomainError::FriendLimitReached(self.config.friend_limit).into());
        }

        let requester_friend_requests = self.count_outgoing_friend_requests(requester).await?;
        if requester_friend_requests >= self.config.friend_request_limit {
            return Err(
                DomainError::FriendRequestLimitReached(self.config.friend_request_limit).into(),
            );
        }

        let addressee_friend_requests = self.count_incoming_friend_requests(addressee).await?;
        if addressee_friend_requests >= self.config.friend_request_limit {
            return Err(
                DomainError::FriendRequestLimitReached(self.config.friend_request_limit).into(),
            );
        }

        let new = friendship::ActiveModel {
            requester_id: Set(requester),
            addressee_id: Set(addressee),
            ..Default::default()
        };
        self.insert(new).await
    }

    pub fn paginate_by_user(&self, id: Uuid, page_size: u64) -> Paginate<'_, friendship::Entity> {
        self.paginate()
            .filter(
                Condition::any()
                    .add(friendship::Column::RequesterId.eq(id))
                    .add(friendship::Column::AddresseeId.eq(id)),
            )
            .page_size(page_size)
    }

    pub async fn stream_friend_ids_of(
        &self,
        id: Uuid,
    ) -> CoreResult<BoxStream<'_, CoreResult<Uuid>>> {
        let filter = Self::condition_any_of(id, FriendshipStatus::Accepted);

        let stream = friendship::Entity::find()
            .filter(filter)
            .select_only()
            .column(friendship::Column::RequesterId)
            .column(friendship::Column::AddresseeId)
            .into_tuple::<(Uuid, Uuid)>()
            .stream(self.db())
            .await
            .map_err(Into::<CoreError>::into)?;

        Ok(stream
            .map(move |result| {
                result
                    .map(|(requester, addressee)| {
                        if requester == id {
                            addressee
                        } else {
                            requester
                        }
                    })
                    .map_err(Into::into)
            })
            .boxed())
    }

    pub async fn count_friends(&self, id: Uuid) -> CoreResult<u64> {
        self.count(Self::condition_any_of(id, FriendshipStatus::Accepted))
            .await
    }

    pub async fn count_incoming_friend_requests(&self, id: Uuid) -> CoreResult<u64> {
        self.count(Self::condition_addressee_of(id, FriendshipStatus::Pending))
            .await
    }

    pub async fn count_outgoing_friend_requests(&self, id: Uuid) -> CoreResult<u64> {
        self.count(Self::condition_requester_of(id, FriendshipStatus::Pending))
            .await
    }
}

// Conditions
impl FriendshipStore {
    fn condition_requester_of(id: Uuid, status: FriendshipStatus) -> impl IntoCondition {
        friendship::Column::RequesterId
            .eq(id)
            .and(friendship::Column::Status.eq(status as i16))
    }

    fn condition_addressee_of(id: Uuid, status: FriendshipStatus) -> impl IntoCondition {
        friendship::Column::AddresseeId
            .eq(id)
            .and(friendship::Column::Status.eq(status as i16))
    }

    fn condition_any_of(id: Uuid, status: FriendshipStatus) -> impl IntoCondition {
        Condition::all()
            .add(
                Condition::any()
                    .add(friendship::Column::RequesterId.eq(id))
                    .add(friendship::Column::AddresseeId.eq(id)),
            )
            .add(friendship::Column::Status.eq(status as i16))
    }
}
