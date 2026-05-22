use crate::ui::tabs::TabViewer;
use crate::ui::widgets::board::BoardWidget;
use checkmade_core::game::visuals::BoardVisuals;
use egui::Widget;

pub fn show(v: &mut TabViewer, ui: &mut egui::Ui) {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            let size = ui.available_width().min(ui.available_height());
            BoardWidget::new(v.images, &BoardVisuals::default())
                .size(size)
                .ui(ui);
        });
}
