use crate::http::Http;
use crate::i18n::Translatable;
use crate::store::Store;
use crate::ui::icons;
use checkmade_core::lingo::Lingo::*;
use egui::{Response, Ui, Widget};

pub struct ProfileMenu<'a> {
    store: &'a Store,
    http: &'a mut Http,
}

impl<'a> ProfileMenu<'a> {
    pub fn new(store: &'a Store, http: &'a mut Http) -> Self {
        Self { store, http }
    }
}

impl Widget for ProfileMenu<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| match &self.store.me.value {
            Some(info) => {
                let label = format!("{} {}", icons::USER, info.public.username);
                ui.menu_button(label, |ui| {
                    if ui
                        .button(format!("{} {}", icons::SIGN_OUT, Logout.t()))
                        .clicked()
                    {
                        self.http.do_logout(ui.ctx());
                    }
                });
            }
            None => {
                ui.spinner();
                ui.label(format!("{}...", FetchingUserInfo.t()));
            }
        })
        .response
    }
}
