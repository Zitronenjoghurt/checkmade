use crate::i18n::Translatable;
use crate::ui::icons;
use checkmade_core::lingo::Lingo::NoSessionRequests;
use eframe::emath::Align;
use egui::{Frame, Layout, ScrollArea};

pub struct GamesOutgoing<'a> {
    store: &'a mut crate::store::Store,
    ws: &'a mut crate::ws::Ws,
}

impl<'a> GamesOutgoing<'a> {
    pub fn new(store: &'a mut crate::store::Store, ws: &'a mut crate::ws::Ws) -> Self {
        Self { store, ws }
    }
}

impl egui::Widget for GamesOutgoing<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let Some(requests) = self.store.outgoing_session_requests.value.as_ref() else {
            return ui.spinner();
        };

        if requests.is_empty() {
            return ui.label(NoSessionRequests.t());
        };

        ScrollArea::vertical()
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    for request in requests.values() {
                        Frame::group(ui.style()).show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                if let Some(opponent) = request.opponent_id {
                                    if let Some(info) = self.store.users.get(opponent) {
                                        ui.strong(format!("vs. {}", info.username));
                                    } else {
                                        ui.horizontal(|ui| {
                                            ui.strong("vs.");
                                            ui.spinner();
                                        });
                                    }
                                } else {
                                    ui.strong(format!("vs. {}", icons::GLOBE));
                                }
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.button(icons::TRASH).clicked() {
                                        self.ws.remove_session_request(request.id);
                                    }
                                });
                            });
                        });
                    }
                })
                .response
            })
            .inner
    }
}
