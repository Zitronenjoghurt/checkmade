use crate::ui::widgets::friends::friend::FriendWidget;
use egui::{Response, ScrollArea, Ui};

pub struct Friendlist<'a> {
    server_time: &'a crate::server_time::ServerTime,
    store: &'a mut crate::store::Store,
    ws: &'a mut crate::ws::Ws,
}

impl<'a> Friendlist<'a> {
    pub fn new(
        server_time: &'a crate::server_time::ServerTime,
        store: &'a mut crate::store::Store,
        ws: &'a mut crate::ws::Ws,
    ) -> Self {
        Self {
            server_time,
            store,
            ws,
        }
    }
}

impl<'a> egui::Widget for Friendlist<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Some(friends) = &self.store.friends.value else {
            return ui.spinner();
        };

        ScrollArea::vertical()
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    if friends.is_empty() {
                        ui.label("No friends");
                    } else {
                        for (id, fi) in friends {
                            if let Some(info) = self.store.users.get(*id) {
                                let elapsed = self.server_time.elapsed_since(fi.since);
                                FriendWidget::new(fi, info, self.ws, elapsed).ui(ui);
                            } else {
                                ui.spinner();
                            }
                        }
                    }
                })
                .response
            })
            .inner
    }
}
