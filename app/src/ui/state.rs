use crate::ui::widgets::friends::FriendsTab;
use crate::ui::widgets::games::GamesTab;
use checkmade_core::types::session_request::CreateSessionRequest;

pub mod sandbox;
pub mod settings;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct UiState {
    pub friends_tab: FriendsTab,
    pub games_tab: GamesTab,
    pub sandbox_board: sandbox::SandboxState,
    pub session_create: CreateSessionRequest,
    pub settings: settings::Settings,
}

impl UiState {
    pub fn update(&mut self, ctx: &egui::Context) {
        self.settings.apply(ctx);
    }
}
