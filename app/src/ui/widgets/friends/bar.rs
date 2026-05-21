use crate::ui::widgets::friends::FriendsTab;
use crate::ui::widgets::with_badge::WithBadge;
use egui::{Response, ScrollArea, Ui};
use strum::IntoEnumIterator;

pub struct FriendsBar<'a> {
    tab: &'a mut FriendsTab,
    incoming_count: usize,
}

impl<'a> FriendsBar<'a> {
    pub fn new(tab: &'a mut FriendsTab, incoming_count: usize) -> Self {
        Self {
            tab,
            incoming_count,
        }
    }
}

impl egui::Widget for FriendsBar<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ScrollArea::horizontal()
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for tab in FriendsTab::iter() {
                        let selected = *self.tab == tab;
                        let label = format!("{} {}", tab.icon(), tab.title());

                        let mut widget = WithBadge::new(egui::Button::selectable(selected, label));

                        if matches!(tab, FriendsTab::Incoming) {
                            widget = widget.count(self.incoming_count);
                        }

                        if ui.add(widget).clicked() {
                            *self.tab = tab;
                        }
                    }
                })
                .response
            })
            .inner
    }
}
