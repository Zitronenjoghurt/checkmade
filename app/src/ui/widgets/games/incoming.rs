use crate::i18n::Translatable;
use crate::ui::icons;
use checkmade_core::lingo::Lingo::NoSessionRequests;
use egui::{Frame, ScrollArea};

pub struct GamesIncoming<'a> {
    store: &'a mut crate::store::Store,
    ws: &'a mut crate::ws::Ws,
}

impl<'a> GamesIncoming<'a> {
    pub fn new(store: &'a mut crate::store::Store, ws: &'a mut crate::ws::Ws) -> Self {
        Self { store, ws }
    }
}

impl egui::Widget for GamesIncoming<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let Some(requests) = self.store.incoming_session_requests.value.as_ref() else {
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
                                if ui.button(icons::CHECK_CIRCLE).clicked() {
                                    self.ws.accept_session_request(request.id);
                                }
                                if ui.button(icons::X_CIRCLE).clicked() {
                                    self.ws.decline_session_request(request.id);
                                }
                                if let Some(info) = self.store.users.get(request.requester_id) {
                                    ui.strong(format!("vs. {}", info.username));
                                } else {
                                    ui.horizontal(|ui| {
                                        ui.strong("vs.");
                                        ui.spinner();
                                    });
                                }
                            });
                        });
                    }
                })
                .response
            })
            .inner
    }
}
