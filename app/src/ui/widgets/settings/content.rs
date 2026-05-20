use crate::ui::state::settings::{Settings, SettingsTab};
use crate::ui::widgets::reset_slider::ResetSlider;
use egui::{Grid, Response, ScrollArea, Ui, Widget};

pub struct SettingsContent<'a> {
    settings: &'a mut Settings,
}

impl<'a> SettingsContent<'a> {
    pub fn new(settings: &'a mut Settings) -> Self {
        Self { settings }
    }
}

impl Widget for SettingsContent<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ScrollArea::vertical()
            .show(ui, |ui| {
                ui.vertical_centered(|ui| match self.settings.current_tab {
                    SettingsTab::General => self.general(ui),
                })
                .response
            })
            .inner
    }
}

impl SettingsContent<'_> {
    pub fn general(mut self, ui: &mut Ui) {
        ui.heading("General");
        ui.separator();

        let s = &mut self.settings;

        Grid::new("settings_general_grid")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("UI Scale");
                let response = ResetSlider::new(&mut s.ui_scale, 0.5..=5.0)
                    .step_by(0.1)
                    .default_value(Settings::DEFAULT_UI_SCALE)
                    .ui(ui);
                if response.drag_stopped() || (response.changed() && !response.dragged()) {
                    s.dirty = true;
                }
                ui.end_row();
            });
    }
}
