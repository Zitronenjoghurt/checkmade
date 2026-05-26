use crate::ui::tabs::TabViewer;
use crate::utils::markdown::MarkdownKind;

pub fn show(v: &mut TabViewer, ui: &mut egui::Ui) {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            v.md.show(MarkdownKind::Changelog, ui);
        });
}
