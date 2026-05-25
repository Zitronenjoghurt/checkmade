use crate::ui::icons;
use crate::ui::widgets::piece_icons::PieceIcons;
use checkmade_core::giga_chess::prelude::Color;
use checkmade_core::types::session_id::SessionId;
use checkmade_core::types::user_id::UserId;
use egui::{Frame, Response, Ui};

pub struct ArenaUser<'a> {
    session_id: SessionId,
    user_id: UserId,
    color: Color,
    images: &'a mut crate::utils::images::Images,
    store: &'a mut crate::store::Store,
    width: Option<f32>,
}

impl<'a> ArenaUser<'a> {
    pub fn new(
        session_id: SessionId,
        user_id: UserId,
        color: Color,
        images: &'a mut crate::utils::images::Images,
        store: &'a mut crate::store::Store,
    ) -> Self {
        Self {
            session_id,
            user_id,
            color,
            images,
            store,
            width: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl egui::Widget for ArenaUser<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Some(user) = self.store.users.get(self.user_id).cloned() else {
            return ui.spinner();
        };

        let Some(session) = self.store.active_sessions.get_entry(&self.session_id) else {
            return ui.spinner();
        };

        Frame::group(ui.style())
            .show(ui, |ui| {
                if let Some(width) = self.width {
                    ui.set_min_width(width);
                }

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(icons::USER);
                        ui.strong(user.username);
                    });

                    let captured_pieces = self
                        .store
                        .session_captured_pieces(self.session_id, self.color);
                    PieceIcons::new(captured_pieces, self.color.opposite(), self.images)
                        .overlap(2.0)
                        .ui(ui);
                })
                .response
            })
            .inner
    }
}
