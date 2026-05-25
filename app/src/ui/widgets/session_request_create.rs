use crate::i18n::Translatable;
use crate::ui::widgets::generic_select::GenericSelect;
use crate::ui::widgets::session_config_data::SessionConfigDataWidget;
use checkmade_core::lingo::Lingo::{Opponent, Public, PublicGameInfo};
use checkmade_core::types::session_request::CreateSessionRequest;
use egui::{Response, Ui};

pub struct SessionRequestCreate<'a> {
    request: &'a mut CreateSessionRequest,
    store: &'a mut crate::store::Store,
}

impl<'a> SessionRequestCreate<'a> {
    pub fn new(request: &'a mut CreateSessionRequest, store: &'a mut crate::store::Store) -> Self {
        Self { request, store }
    }
}

impl<'a> egui::Widget for SessionRequestCreate<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let store = &*self.store;
        let Some(friend_ids) = store
            .friends
            .value
            .as_ref()
            .map(|friends| friends.keys().copied().collect::<Vec<_>>())
        else {
            return ui.spinner();
        };

        ui.vertical(|ui| {
            SessionConfigDataWidget::new(&mut self.request.config).ui(ui);
            ui.separator();
            if !self.request.public {
                GenericSelect::new_optional(
                    &mut self.request.opponent_id,
                    friend_ids,
                    "session_request_create_opponent_select",
                    |id| {
                        if let Some(user) = store.users.peek(&id) {
                            user.username.clone()
                        } else {
                            "???".to_string()
                        }
                    },
                )
                .label(Opponent.t().as_ref())
                .ui(ui);
            };
            ui.vertical(|ui| {
                if ui
                    .checkbox(&mut self.request.public, format!("{}?", Public.t()))
                    .changed()
                    && self.request.public
                {
                    self.request.opponent_id = None;
                };
                ui.small(PublicGameInfo.t());
            });
        })
        .response
    }
}
