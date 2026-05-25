use crate::ui::widgets::friends::FriendsTab;
use crate::ui::widgets::games::GamesTab;
use checkmade_core::types::session_request::CreateSessionRequest;

pub mod arena;
pub mod sandbox;
pub mod settings;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct UiState {
    pub arena: arena::ArenaState,
    pub friends_tab: FriendsTab,
    pub games_tab: GamesTab,
    pub session_create: CreateSessionRequest,
    pub settings: settings::Settings,
}

impl UiState {
    pub fn update(&mut self, ctx: &egui::Context) {
        self.settings.apply(ctx);
    }
}
