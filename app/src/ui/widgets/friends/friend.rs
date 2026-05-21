use crate::ui::icons;
use checkmade_core::types::user_info::PublicUserInfo;
use chrono::DateTime;
use egui::{Frame, Response, Ui, Widget};

pub struct FriendWidget<'a> {
    info: &'a PublicUserInfo,
    ws: &'a mut crate::ws::Ws,
    since: DateTime<chrono::Utc>,
}

impl<'a> FriendWidget<'a> {
    pub fn new(
        info: &'a PublicUserInfo,
        ws: &'a mut crate::ws::Ws,
        since: DateTime<chrono::Utc>,
    ) -> Self {
        Self { info, ws, since }
    }
}

impl<'a> Widget for FriendWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        Frame::group(ui.style())
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(icons::USER);
                    ui.label(&self.info.username);
                });
            })
            .response
    }
}
