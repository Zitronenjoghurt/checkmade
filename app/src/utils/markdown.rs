use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

const CHANGELOG: &str = include_str!("../../../CHANGELOG.md");

#[derive(Default)]
pub struct Markdown {
    changelog: CommonMarkCache,
}

impl Markdown {
    pub fn show(&mut self, kind: MarkdownKind, ui: &mut egui::Ui) -> egui::Response {
        match kind {
            MarkdownKind::Changelog => Self::render(ui, &mut self.changelog, CHANGELOG),
        }
    }

    fn render(ui: &mut egui::Ui, cache: &mut CommonMarkCache, content: &str) -> egui::Response {
        let available = ui.available_height();
        egui::ScrollArea::vertical()
            .max_height(available)
            .show(ui, |ui| {
                CommonMarkViewer::new().show(ui, cache, content).response
            })
            .inner
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MarkdownKind {
    Changelog,
}
