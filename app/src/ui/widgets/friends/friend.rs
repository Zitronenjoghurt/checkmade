use crate::i18n::Translatable;
use crate::tl;
use crate::ui::icons;
use crate::ui::modal::Modal;
use crate::ui::widgets::with_badge::WithBadge;
use crate::utils::fmt::fmt_duration;
use checkmade_core::lingo::Lingo::*;
use checkmade_core::types::user_info::PublicUserInfo;
use egui::{Align, Color32, Frame, Label, Layout, Response, Ui, Widget};

pub struct FriendWidget<'a> {
    info: &'a PublicUserInfo,
    ws: &'a mut crate::ws::Ws,
    ago: web_time::Duration,
}

impl<'a> FriendWidget<'a> {
    pub fn new(
        info: &'a PublicUserInfo,
        ws: &'a mut crate::ws::Ws,
        ago: web_time::Duration,
    ) -> Self {
        Self { info, ws, ago }
    }

    fn remove_modal(&mut self, ui: &mut Ui) -> Modal {
        let modal = Modal::new(ui.ctx(), "remove_friend_modal");

        modal.show(|ui| {
            modal.title(ui, ModalRemoveFriendTitle.t());
            modal.body(ui, tl!(ModalRemoveFriendBody, x = &self.info.username));
            modal.buttons(ui, |ui| {
                if modal.button(ui, Cancel.t()).clicked() {
                    modal.close();
                }
                if modal.caution_button(ui, Remove.t()).clicked() {
                    self.ws.remove_friend(self.info.id);
                    modal.close();
                }
            });
        });

        modal
    }
}

impl<'a> Widget for FriendWidget<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let remove_modal = self.remove_modal(ui);

        Frame::group(ui.style())
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    let color = if self.info.is_online {
                        Color32::GREEN
                    } else {
                        Color32::GRAY
                    };
                    ui.add(
                        WithBadge::new(Label::new(icons::USER))
                            .dot(true)
                            .color(color),
                    );

                    ui.vertical(|ui| {
                        ui.label(&self.info.username);
                        ui.small(format!(
                            "{}: {}",
                            FriendsSince.t(),
                            tl!(XAgo, x = fmt_duration(self.ago))
                        ));
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(icons::TRASH).clicked() {
                            remove_modal.open();
                        }
                    });
                });
            })
            .response
    }
}
