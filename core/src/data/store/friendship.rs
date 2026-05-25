use crate::config::CoreConfig;
use crate::data::entity::{friendship, session};
use crate::data::store::{Page, Paginate, Store};
use crate::error::{CoreError, CoreResult, DomainError};
use crate::types::friendship_status::FriendshipStatus;
use crate::types::session_status::SessionStatus;
use futures::stream::BoxStream;
use futures::StreamExt;
use sea_orm::prelude::Expr;
use sea_orm::sea_query::prelude::NaiveDateTime;
use sea_orm::sea_query::{Alias, ConditionType, IntoCondition, IntoIden, IntoTableRef, SimpleExpr};
use sea_orm::{
    ColumnTrait, ExprTrait, FromQueryResult, Identity, JoinType, RelationDef, RelationType, Set,
};
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

// With stats
impl FriendshipStore {
    fn session_join() -> RelationDef {
        RelationDef {
            rel_type: RelationType::HasMany,
            from_tbl: friendship::Entity.into_table_ref(),
            to_tbl: session::Entity.into_table_ref(),
            from_col: Identity::Binary(
                friendship::Column::RequesterId.into_iden(),
                friendship::Column::AddresseeId.into_iden(),
            ),
            to_col: Identity::Binary(
                session::Column::WhiteId.into_iden(),
                session::Column::BlackId.into_iden(),
            ),
            is_owner: false,
            skip_fk: false,
            on_delete: None,
            on_update: None,
            fk_name: None,
            condition_type: ConditionType::Any,
            on_condition: Some(std::sync::Arc::new(|f, s| {
                Condition::all()
                    .add(
                        Expr::col((f.clone(), friendship::Column::AddresseeId))
                            .equals((s.clone(), session::Column::WhiteId)),
                    )
                    .add(
                        Expr::col((f, friendship::Column::RequesterId))
                            .equals((s, session::Column::BlackId)),
                    )
            })),
        }
    }

    fn wins_case(user_id: Uuid) -> SimpleExpr {
        Expr::case(
            Condition::any()
                .add(
                    Expr::col((Alias::new("s"), session::Column::WhiteId))
                        .eq(user_id)
                        .and(
                            Expr::col((Alias::new("s"), session::Column::Status))
                                .eq(SessionStatus::WhiteWins as i16),
                        ),
                )
                .add(
                    Expr::col((Alias::new("s"), session::Column::BlackId))
                        .eq(user_id)
                        .and(
                            Expr::col((Alias::new("s"), session::Column::Status))
                                .eq(SessionStatus::BlackWins as i16),
                        ),
                ),
            1,
        )
        .finally(0)
        .into()
    }

    fn losses_case(user_id: Uuid) -> SimpleExpr {
        Expr::case(
            Condition::any()
                .add(
                    Expr::col((Alias::new("s"), session::Column::WhiteId))
                        .eq(user_id)
                        .and(
                            Expr::col((Alias::new("s"), session::Column::Status))
                                .eq(SessionStatus::BlackWins as i16),
                        ),
                )
                .add(
                    Expr::col((Alias::new("s"), session::Column::BlackId))
                        .eq(user_id)
                        .and(
                            Expr::col((Alias::new("s"), session::Column::Status))
                                .eq(SessionStatus::WhiteWins as i16),
                        ),
                ),
            1,
        )
        .finally(0)
        .into()
    }

    fn draws_case() -> SimpleExpr {
        Expr::case(
            Expr::col((Alias::new("s"), session::Column::Status)).eq(SessionStatus::Draw as i16),
            1,
        )
        .finally(0)
        .into()
    }

    pub async fn paginate_friends_with_stats(
        &self,
        user_id: Uuid,
        page: u64,
        page_size: u64,
    ) -> CoreResult<Page<FriendshipWithStats>> {
        let accepted_filter = Condition::all()
            .add(
                Condition::any()
                    .add(friendship::Column::RequesterId.eq(user_id))
                    .add(friendship::Column::AddresseeId.eq(user_id)),
            )
            .add(friendship::Column::Status.eq(FriendshipStatus::Accepted as i16));

        // Total count (cheap, no join needed)
        let total_items = Self::count_with(self.db(), accepted_filter.clone()).await?;
        let total_pages = if page_size == 0 {
            1
        } else {
            total_items.div_ceil(page_size)
        };

        let items = friendship::Entity::find()
            .join_as(JoinType::LeftJoin, Self::session_join(), Alias::new("s"))
            .select_only()
            .column(friendship::Column::RequesterId)
            .column(friendship::Column::AddresseeId)
            .column(friendship::Column::CreatedAt)
            .column_as(
                Expr::cust_with_expr("COALESCE(SUM($1), 0)", Self::wins_case(user_id)),
                "times_won",
            )
            .column_as(
                Expr::cust_with_expr("COALESCE(SUM($1), 0)", Self::losses_case(user_id)),
                "times_lost",
            )
            .column_as(
                Expr::cust_with_expr("COALESCE(SUM($1), 0)", Self::draws_case()),
                "times_drawn",
            )
            .filter(accepted_filter)
            .group_by(friendship::Column::RequesterId)
            .group_by(friendship::Column::AddresseeId)
            .group_by(friendship::Column::CreatedAt)
            .offset(Some(page * page_size))
            .limit(Some(page_size))
            .into_model::<FriendshipWithStats>()
            .all(self.db())
            .await?;

        Ok(Page {
            items,
            page,
            page_size,
            total_items,
            total_pages,
        })
    }

    pub async fn pair_stats(&self, user_id: Uuid, friend_id: Uuid) -> CoreResult<(i64, i64, i64)> {
        #[derive(FromQueryResult)]
        struct Stats {
            times_won: i64,
            times_lost: i64,
            times_drawn: i64,
        }

        let result = friendship::Entity::find()
            .join_as(JoinType::LeftJoin, Self::session_join(), Alias::new("s"))
            .select_only()
            .column_as(
                Expr::cust_with_expr("COALESCE(SUM($1), 0)", Self::wins_case(user_id)),
                "times_won",
            )
            .column_as(
                Expr::cust_with_expr("COALESCE(SUM($1), 0)", Self::losses_case(user_id)),
                "times_lost",
            )
            .column_as(
                Expr::cust_with_expr("COALESCE(SUM($1), 0)", Self::draws_case()),
                "times_drawn",
            )
            .filter(
                Condition::any()
                    .add(
                        friendship::Column::RequesterId
                            .eq(user_id)
                            .and(friendship::Column::AddresseeId.eq(friend_id)),
                    )
                    .add(
                        friendship::Column::RequesterId
                            .eq(friend_id)
                            .and(friendship::Column::AddresseeId.eq(user_id)),
                    ),
            )
            .into_model::<Stats>()
            .one(self.db())
            .await?;

        match result {
            Some(s) => Ok((s.times_won, s.times_lost, s.times_drawn)),
            None => Ok((0, 0, 0)),
        }
    }
}

#[derive(Debug, FromQueryResult)]
pub struct FriendshipWithStats {
    pub requester_id: Uuid,
    pub addressee_id: Uuid,
    pub created_at: NaiveDateTime,
    pub times_won: i64,
    pub times_lost: i64,
    pub times_drawn: i64,
}
