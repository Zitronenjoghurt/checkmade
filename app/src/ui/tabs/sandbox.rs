use crate::ui::tabs::TabViewer;
use crate::ui::widgets::sandbox::SandboxWidget;
use egui::Widget;

pub fn show(v: &mut TabViewer, ui: &mut egui::Ui) {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            SandboxWidget::new(v.images, &mut v.state.sandbox_board).ui(ui);
        });
}
