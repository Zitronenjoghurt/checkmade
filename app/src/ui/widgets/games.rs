use crate::i18n::Translatable;
use crate::ui::icons;
use checkmade_core::lingo::Lingo::*;
use strum::EnumIter;

pub mod bar;
pub mod create;
pub mod incoming;
pub mod ongoing;
pub mod outgoing;

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, EnumIter,
)]
pub enum GamesTab {
    #[default]
    Ongoing,
    Create,
    Incoming,
    Outgoing,
    PublicGames,
    PublicRequests,
}

impl GamesTab {
    pub fn title(&self) -> String {
        match self {
            Self::Ongoing => OngoingGames.t().to_string(),
            Self::Create => Create.t().to_string(),
            Self::Incoming => Requests.t().to_string(),
            Self::Outgoing => Pending.t().to_string(),
            Self::PublicGames => PublicGames.t().to_string(),
            Self::PublicRequests => PublicRequests.t().to_string(),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Ongoing => icons::CROWN,
            Self::Create => icons::PLUS_SQUARE,
            Self::Incoming => icons::ENVELOPE,
            Self::Outgoing => icons::TRAY_ARROW_UP,
            Self::PublicGames => icons::GLOBE,
            Self::PublicRequests => icons::BROADCAST,
        }
    }
}
