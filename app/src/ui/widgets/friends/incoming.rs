use crate::i18n::Translatable;
use crate::server_time::ServerTime;
use crate::store::Store;
use crate::tl;
use crate::ui::icons;
use checkmade_core::lingo::Lingo::{NoFriendRequests, XAgo};
use egui::{Frame, Response, ScrollArea, Ui};

pub struct FriendIncoming<'a> {
    server_time: &'a ServerTime,
    store: &'a mut Store,
    ws: &'a mut crate::ws::Ws,
}

impl<'a> FriendIncoming<'a> {
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

impl egui::Widget for FriendIncoming<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Some(incoming) = self.store.incoming_friend_requests.value.as_ref() else {
            return ui.spinner();
        };

        if incoming.is_empty() {
            return ui.label(NoFriendRequests.t());
        };

        ScrollArea::vertical()
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    for (id, since) in incoming {
                        let elapsed = self.server_time.elapsed_since(*since);
                        Frame::group(ui.style()).show(ui, |ui| {
                            if let Some(info) = self.store.users.get(*id) {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        if ui.button(icons::CHECK_CIRCLE).clicked() {
                                            self.ws.accept_friend_request(*id);
                                        }
                                        if ui.button(icons::X_CIRCLE).clicked() {
                                            self.ws.decline_friend_request(*id);
                                        }
                                        ui.separator();
                                        ui.strong(&info.username);
                                    });
                                    ui.small(tl!(XAgo, x = format!("{elapsed:?}")));
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
