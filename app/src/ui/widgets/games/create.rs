use crate::i18n::Translatable;
use crate::ui::widgets::session_request_create::SessionRequestCreate;
use checkmade_core::lingo::Lingo::Create;
use checkmade_core::types::session_request::CreateSessionRequest;
use egui::{Button, Response, Ui};

pub struct GamesCreate<'a> {
    request: &'a mut CreateSessionRequest,
    store: &'a mut crate::store::Store,
    ws: &'a mut crate::ws::Ws,
}

impl<'a> GamesCreate<'a> {
    pub fn new(
        request: &'a mut CreateSessionRequest,
        store: &'a mut crate::store::Store,
        ws: &'a mut crate::ws::Ws,
    ) -> Self {
        Self { request, store, ws }
    }
}

impl egui::Widget for GamesCreate<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            SessionRequestCreate::new(self.request, self.store).ui(ui);
            ui.separator();
            let can_create = self.request.public || self.request.opponent_id.is_some();
            let create_button = ui.add_enabled(can_create, Button::new(Create.t()));
            if create_button.clicked() {
                self.ws.create_session_request(self.request.clone());
            }
        })
        .response
    }
}
