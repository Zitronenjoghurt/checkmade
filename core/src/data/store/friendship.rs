use crate::data::entity::friendship;
use crate::data::store::{Paginate, Store};
use crate::error::CoreResult;
use crate::types::friendship_status::FriendshipStatus;
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, Set};
use sea_orm::{Condition, DatabaseConnection, EntityTrait};
use uuid::Uuid;

pub struct FriendshipStore {
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
    pub fn new(connection: &DatabaseConnection) -> Self {
        Self {
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
}
