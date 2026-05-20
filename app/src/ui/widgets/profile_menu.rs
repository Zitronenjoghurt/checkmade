use crate::http::Http;
use crate::store::Store;
use crate::ui::icons;
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
        ui.horizontal(|ui| match &self.store.user_info.value {
            Some(info) => {
                let label = format!("{} {}", icons::USER, info.public.username);
                ui.menu_button(label, |ui| {
                    if ui.button(format!("{} Logout", icons::SIGN_OUT)).clicked() {
                        self.http.do_logout(ui.ctx());
                        ui.close_menu();
                    }
                });
            }
            None => {
                ui.spinner();
                ui.label("Fetching user info...");
            }
        })
        .response
    }
}
