use crate::ui::icons;
use crate::ui::state::arena::ArenaState;
use crate::ui::widgets::piece_icons::PieceIcons;
use crate::utils::fmt::fmt_clock;
use checkmade_core::giga_chess::prelude::Color;
use checkmade_core::types::user_id::UserId;
use egui::{Frame, Response, Ui};

pub struct ArenaUser<'a> {
    arena: &'a ArenaState,
    user_id: UserId,
    color: Color,
    now_ms: u64,
    images: &'a mut crate::utils::images::Images,
    store: &'a mut crate::store::Store,
    width: Option<f32>,
}

impl<'a> ArenaUser<'a> {
    pub fn new(
        arena: &'a ArenaState,
        user_id: UserId,
        color: Color,
        now_ms: u64,
        images: &'a mut crate::utils::images::Images,
        store: &'a mut crate::store::Store,
    ) -> Self {
        Self {
            arena,
            user_id,
            color,
            now_ms,
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
        let time_opt = self.arena.time(self.store, self.color, self.now_ms);

        let Some(user) = self.store.users.get(self.user_id).cloned() else {
            return ui.spinner();
        };

        Frame::group(ui.style())
            .show(ui, |ui| {
                if let Some(width) = self.width {
                    ui.set_min_width(width);
                    ui.set_max_width(width);
                }

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(icons::USER);
                        ui.strong(user.username);

                        if let Some((time_ms, inc_ms)) = time_opt {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let clock_str = fmt_clock(time_ms, inc_ms);

                                    ui.label(
                                        egui::RichText::new(clock_str)
                                            .strong()
                                            .monospace()
                                            .size(16.0),
                                    );
                                },
                            );
                        }
                    });

                    let captured_pieces = self.arena.captured_pieces(self.store, self.color);
                    PieceIcons::new(captured_pieces, self.color.opposite(), self.images)
                        .overlap(2.0)
                        .ui(ui);
                })
                .response
            })
            .inner
    }
}
