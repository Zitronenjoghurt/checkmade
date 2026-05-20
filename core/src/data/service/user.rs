use crate::data::entity::user;
use crate::data::store::Store;
use crate::data::Data;
use crate::error::CoreResult;
use crate::types::user_info::{PrivateUserInfo, PublicUserInfo};
use sea_orm::{IntoActiveModel, Set};
use std::sync::Arc;
use uuid::Uuid;

pub struct UserService {
    pub data: Arc<Data>,
}

impl UserService {
    pub fn new(data: &Arc<Data>) -> Self {
        Self {
            data: Arc::clone(data),
        }
    }

    pub fn public_info(&self, user: &user::Model) -> PublicUserInfo {
        PublicUserInfo {
            username: user.username.clone(),
        }
    }

    pub fn private_info(&self, user: &user::Model) -> PrivateUserInfo {
        PrivateUserInfo {
            public: self.public_info(user),
            permissions: user.permissions,
        }
    }

    pub async fn log_rate_limit_infraction(&self, user_id: Uuid) -> CoreResult<u16> {
        let Some(user) = self.data.user.find_by_id(user_id).await? else {
            return Ok(0);
        };

        let infractions = user.rate_limit_infractions.saturating_add(1);
        let mut active = user.into_active_model();
        active.rate_limit_infractions = Set(infractions);
        self.data.user.update(active).await?;

        Ok(infractions as u16)
    }
}
