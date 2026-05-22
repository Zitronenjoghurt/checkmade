use crate::ui::widgets::friends::FriendsTab;
use checkmade_core::game::visuals::BoardVisuals;

pub mod settings;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct UiState {
    pub friends_tab: FriendsTab,
    pub sandbox_board: BoardVisuals,
    pub settings: settings::Settings,
}

impl UiState {
    pub fn update(&mut self, ctx: &egui::Context) {
        self.settings.apply(ctx);
    }
}
