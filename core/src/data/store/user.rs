use crate::data::entity::{user, user_identity};
use crate::data::store::Store;
use crate::error::CoreResult;
use crate::types::friend_code::generate_friend_code;
use crate::types::identity_provider::IdentityProvider;
use petname::petname;
use sea_orm::{ActiveModelTrait, ColumnTrait, Set};
use sea_orm::{DatabaseConnection, EntityTrait};
use sea_orm::{QueryFilter, TransactionTrait};

pub struct UserStore {
    connection: DatabaseConnection,
}

impl Store for UserStore {
    type Entity = user::Entity;
    type ActiveModel = user::ActiveModel;

    fn db(&self) -> &DatabaseConnection {
        &self.connection
    }
}

impl UserStore {
    pub fn new(connection: &DatabaseConnection) -> Self {
        Self {
            connection: connection.clone(),
        }
    }

    pub async fn find_by_identity(
        &self,
        provider: IdentityProvider,
        identifier: &str,
    ) -> CoreResult<Option<user::Model>> {
        let result = user::Entity::find()
            .inner_join(user_identity::Entity)
            .filter(user_identity::Column::Provider.eq(provider as i16))
            .filter(user_identity::Column::Identifier.eq(identifier))
            .one(self.db())
            .await?;
        Ok(result)
    }

    pub async fn create_from_identity(
        &self,
        provider: IdentityProvider,
        identifier: &str,
    ) -> CoreResult<user::Model> {
        let txn = self.db().begin().await?;

        let username = petname(2, "-").unwrap();
        let friend_code = generate_friend_code();

        let new_user = user::ActiveModel {
            username: Set(username),
            friend_code: Set(friend_code),
            ..Default::default()
        };
        let user = new_user.insert(&txn).await?;

        let identity = user_identity::ActiveModel {
            user_id: Set(user.id),
            provider: Set(provider as i16),
            identifier: Set(identifier.to_string()),
            ..Default::default()
        };
        identity.insert(&txn).await?;

        txn.commit().await?;
        Ok(user)
    }

    pub async fn find_by_friend_code(&self, friend_code: &str) -> CoreResult<Option<user::Model>> {
        self.find_one_by(user::Column::FriendCode.eq(friend_code))
            .await
    }
}
