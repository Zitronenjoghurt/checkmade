use crate::ui::widgets::session_config::SessionConfigWidget;
use checkmade_core::game::session_data::SessionConfigData;
use egui::{Response, Ui};

pub struct SessionConfigDataWidget<'a> {
    config: &'a mut SessionConfigData,
}

impl<'a> SessionConfigDataWidget<'a> {
    pub fn new(config: &'a mut SessionConfigData) -> Self {
        Self { config }
    }
}

impl egui::Widget for SessionConfigDataWidget<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        match &mut self.config {
            SessionConfigData::Normal(session_config) => {
                SessionConfigWidget::new(session_config).ui(ui)
            }
        }
    }
}
