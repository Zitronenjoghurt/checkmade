use checkmade_core::types::session_id::SessionId;
use egui::{Response, Ui};

pub struct SessionWidget<'a> {
    pub session_id: SessionId,
    pub images: &'a mut crate::utils::images::Images,
    pub server_time: &'a crate::server_time::ServerTime,
    pub store: &'a mut crate::store::Store,
    pub ws: &'a mut crate::ws::Ws,
}

impl egui::Widget for SessionWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {}).response
    }
}
