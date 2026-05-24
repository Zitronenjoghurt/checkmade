use crate::ui::tabs::TabViewer;
use crate::ui::widgets::arena::ArenaWidget;
use egui::Widget;

pub fn show(v: &mut TabViewer, ui: &mut egui::Ui) {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ArenaWidget::new(v.images, &mut v.state.arena, v.store, v.ws).ui(ui);
        });
}
