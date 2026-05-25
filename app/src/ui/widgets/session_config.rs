use crate::i18n::Translatable;
use crate::ui::widgets::generic_select::GenericSelect;
use crate::ui::widgets::time_control::TimeControlWidget;
use checkmade_core::giga_chess::prelude::config::SessionConfig;
use checkmade_core::giga_chess::prelude::mode::GameMode;
use checkmade_core::lingo::Lingo::GameMode as GameModeLingo;
use egui::{Response, Ui};
use strum::IntoEnumIterator;

pub struct SessionConfigWidget<'a> {
    config: &'a mut SessionConfig,
}

impl<'a> SessionConfigWidget<'a> {
    pub fn new(config: &'a mut SessionConfig) -> Self {
        Self { config }
    }
}

impl egui::Widget for SessionConfigWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            GenericSelect::new(
                &mut self.config.mode,
                GameMode::iter(),
                "session_config_game_mode_select",
                |gm| gm.t().to_string(),
            )
            .label(GameModeLingo.t().as_ref())
            .ui(ui);
            ui.separator();
            TimeControlWidget::new(&mut self.config.time_control).ui(ui);
        })
        .response
    }
}
