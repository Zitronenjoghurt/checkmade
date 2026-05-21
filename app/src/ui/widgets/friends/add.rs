use crate::i18n::Translatable;
use crate::store::Store;
use crate::ui::icons;
use crate::ui::widgets::validated_input::ValidatedInput;
use checkmade_core::lingo::Lingo::{AddFriend, FriendCode};
use checkmade_core::types::friend_code::{
    friend_code_char_filter, is_valid_friend_code, FRIEND_CODE_LENGTH,
};
use egui::{Id, Response, Ui};

pub struct FriendAdd<'a> {
    store: &'a mut Store,
    ws: &'a mut crate::ws::Ws,
}

impl<'a> FriendAdd<'a> {
    pub fn new(store: &'a mut Store, ws: &'a mut crate::ws::Ws) -> Self {
        Self { store, ws }
    }
}

impl egui::Widget for FriendAdd<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let id = Id::new("friend_add");
        let mut code_visible = ui.data(|d| d.get_temp::<bool>(id).unwrap_or(false));
        let mut input = ui.data(|d| d.get_temp::<String>(id.with("input")).unwrap_or_default());

        let response = ui
            .horizontal(|ui| {
                ui.group(|ui| {
                    ui.label(FriendCode.t());
                    ui.horizontal(|ui| {
                        if let Some(code) = self.store.me.value.as_ref().map(|me| &me.friend_code) {
                            let display = if code_visible {
                                code.clone()
                            } else {
                                "••••••••••••••••".into()
                            };

                            if ui.button(&display).clicked() {
                                code_visible = !code_visible;
                            }

                            let icon = if code_visible {
                                icons::EYE_SLASH
                            } else {
                                icons::EYE
                            };
                            if ui.button(icon).clicked() {
                                code_visible = !code_visible;
                            }

                            if code_visible && ui.button(icons::COPY).clicked() {
                                ui.ctx().copy_text(display);
                            }
                        }
                    });
                });

                ui.separator();

                ui.group(|ui| {
                    ui.label(AddFriend.t());
                    ui.horizontal(|ui| {
                        let resp = ui.add(
                            ValidatedInput::new(&mut input, is_valid_friend_code)
                                .char_limit(FRIEND_CODE_LENGTH)
                                .char_filter(friend_code_char_filter)
                                .uppercase()
                                .width(150.0),
                        );

                        let can_send = !input.trim().is_empty();
                        let enter =
                            resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                        if (ui
                            .add_enabled(can_send, egui::Button::new(icons::PAPER_PLANE_RIGHT))
                            .clicked()
                            || (enter && can_send))
                        {
                            self.ws.send_friend_request(input.trim().to_string());
                            input.clear();
                        }
                    });
                });
            })
            .response;

        ui.data_mut(|d| {
            d.insert_temp(id, code_visible);
            d.insert_temp(id.with("input"), input);
        });

        response
    }
}
