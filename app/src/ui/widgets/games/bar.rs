use crate::ui::widgets::games::GamesTab;
use crate::ui::widgets::with_badge::WithBadge;
use egui::{Response, Ui};
use strum::IntoEnumIterator;

pub struct GamesBar<'a> {
    tab: &'a mut GamesTab,
    playable_count: usize,
    incoming_count: usize,
}

impl<'a> GamesBar<'a> {
    pub fn new(tab: &'a mut GamesTab, playable_count: usize, incoming_count: usize) -> Self {
        Self {
            tab,
            playable_count,
            incoming_count,
        }
    }
}

impl egui::Widget for GamesBar<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        //ScrollArea::horizontal()
        //    .show(ui, |ui| {
        ui.horizontal(|ui| {
            for tab in GamesTab::iter() {
                let selected = *self.tab == tab;
                let label = format!("{} {}", tab.icon(), tab.title());

                let mut widget = WithBadge::new(egui::Button::selectable(selected, label));

                if matches!(tab, GamesTab::Ongoing) {
                    widget = widget.count(self.playable_count);
                }

                if matches!(tab, GamesTab::Incoming) {
                    widget = widget.count(self.incoming_count);
                }

                if ui.add(widget).clicked() {
                    *self.tab = tab;
                }
            }
        })
        .response
        //})
        //.inner
    }
}
