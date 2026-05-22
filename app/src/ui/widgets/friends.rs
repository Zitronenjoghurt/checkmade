use crate::i18n::Translatable;
use crate::ui::icons;
use checkmade_core::lingo::Lingo::*;
use strum::EnumIter;

pub mod add;
pub mod bar;
pub mod friend;
pub mod incoming;
pub mod list;
pub mod outgoing;

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, EnumIter,
)]
pub enum FriendsTab {
    #[default]
    List,
    Incoming,
    Outgoing,
    AddFriend,
}

impl FriendsTab {
    pub fn title(&self) -> String {
        match self {
            Self::List => List.t().to_string(),
            Self::Incoming => Requests.t().to_string(),
            Self::Outgoing => Pending.t().to_string(),
            Self::AddFriend => AddFriend.t().to_string(),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::List => icons::LIST_HEART,
            Self::Incoming => icons::ENVELOPE,
            Self::Outgoing => icons::TRAY_ARROW_UP,
            Self::AddFriend => icons::USER_PLUS,
        }
    }
}
