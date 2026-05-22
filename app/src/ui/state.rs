use crate::ui::widgets::friends::FriendsTab;

pub mod sandbox;
pub mod settings;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct UiState {
    pub friends_tab: FriendsTab,
    pub sandbox_board: sandbox::SandboxState,
    pub settings: settings::Settings,
}

impl UiState {
    pub fn update(&mut self, ctx: &egui::Context) {
        self.settings.apply(ctx);
    }
}
