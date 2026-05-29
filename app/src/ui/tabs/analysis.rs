use crate::ui::tabs::TabViewer;
use crate::ui::widgets::analysis::AnalysisWidget;
use egui::Widget;

pub fn show(v: &mut TabViewer, ui: &mut egui::Ui) {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            AnalysisWidget::new(&mut v.state.analysis).ui(ui);
        });
}
