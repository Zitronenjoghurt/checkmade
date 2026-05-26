use crate::ui::tabs::TabViewer;
use crate::ui::widgets::arena::ArenaWidget;
use egui::Widget;

pub fn show(v: &mut TabViewer, ui: &mut egui::Ui) {
    let max_height = ui.available_height();
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_max_height(max_height - 24.0);
            ArenaWidget::new(
                v.images,
                v.server_time,
                &v.state.settings,
                &mut v.state.arena,
                v.store,
                v.ws,
            )
            .ui(ui);
        });
}
