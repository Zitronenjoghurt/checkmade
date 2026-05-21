use crate::data::entity::user;
use crate::data::service::friendship::generate_friend_code;
use crate::data::store::Store;
use crate::error::CoreResult;
use petname::petname;
use sea_orm::DatabaseConnection;
use sea_orm::{ColumnTrait, Set};

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

    pub async fn find_by_discord_id(
        &self,
        discord_id: impl AsRef<str>,
    ) -> CoreResult<Option<user::Model>> {
        self.find_one_by(user::Column::DiscordId.eq(discord_id.as_ref()))
            .await
    }

    pub async fn find_by_friend_code(&self, friend_code: &str) -> CoreResult<Option<user::Model>> {
        self.find_one_by(user::Column::FriendCode.eq(friend_code))
            .await
    }

    pub async fn create_from_discord(
        &self,
        discord_id: impl AsRef<str>,
    ) -> CoreResult<user::Model> {
        let username = petname(3, "-").unwrap();
        let friend_code = generate_friend_code();
        let new = user::ActiveModel {
            discord_id: Set(Some(discord_id.as_ref().to_string())),
            username: Set(username),
            friend_code: Set(friend_code),
            ..Default::default()
        };
        self.insert(new).await
    }
}
