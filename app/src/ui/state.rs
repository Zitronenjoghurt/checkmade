pub mod settings;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct UiState {
    pub settings: settings::Settings,
}

impl UiState {
    pub fn update(&mut self, ctx: &egui::Context) {
        self.settings.apply(ctx);
    }
}
