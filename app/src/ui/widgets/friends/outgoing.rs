use crate::i18n::Translatable;
use crate::server_time::ServerTime;
use crate::store::Store;
use crate::tl;
use crate::ui::icons;
use crate::utils::fmt::fmt_duration;
use checkmade_core::lingo::Lingo::*;
use eframe::emath::Align;
use egui::{Frame, Layout, Response, ScrollArea, Ui};

pub struct FriendOutgoing<'a> {
    server_time: &'a ServerTime,
    store: &'a mut Store,
    ws: &'a mut crate::ws::Ws,
}

impl<'a> FriendOutgoing<'a> {
    pub fn new(
        server_time: &'a ServerTime,
        store: &'a mut Store,
        ws: &'a mut crate::ws::Ws,
    ) -> Self {
        Self {
            server_time,
            store,
            ws,
        }
    }
}

impl egui::Widget for FriendOutgoing<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Some(outgoing) = self.store.outgoing_friend_requests.value.as_ref() else {
            return ui.spinner();
        };

        if outgoing.is_empty() {
            return ui.label(NoFriendRequests.t());
        };

        ScrollArea::vertical()
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    for (id, since) in outgoing {
                        let elapsed = self.server_time.elapsed_since(*since);
                        Frame::group(ui.style()).show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            if let Some(info) = self.store.users.get(*id) {
                                ui.horizontal(|ui| {
                                    ui.strong(&info.username);
                                    ui.separator();
                                    ui.small(tl!(XAgo, x = fmt_duration(elapsed)));
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if ui.button(icons::TRASH).clicked() {
                                            self.ws.remove_friend_request(info.id);
                                        }
                                    });
                                });
                            } else {
                                ui.spinner();
                            }
                        });
                    }
                })
                .response
            })
            .inner
    }
}
