use crate::i18n::Translatable;
use crate::store::Store;
use crate::ui::modal::Modal;
use crate::ui::state::arena::ArenaState;
use crate::ui::widgets::arena::user::ArenaUser;
use crate::ui::widgets::board::{board_action, BoardAction, BoardWidget};
use crate::utils::images::Images;
use crate::ws::Ws;
use checkmade_core::giga_chess::prelude::Piece;
use checkmade_core::lingo::Lingo::{Cancel, PawnPromotion};
use egui::RichText;

mod user;

pub struct ArenaWidget<'a> {
    images: &'a mut Images,
    state: &'a mut ArenaState,
    store: &'a mut Store,
    ws: &'a mut Ws,
}

impl<'a> ArenaWidget<'a> {
    pub fn new(
        images: &'a mut Images,
        state: &'a mut ArenaState,
        store: &'a mut Store,
        ws: &'a mut Ws,
    ) -> Self {
        Self {
            images,
            state,
            store,
            ws,
        }
    }

    fn promotion_modal(ui: &mut egui::Ui) -> (Modal, Option<Piece>) {
        let modal = Modal::new(ui.ctx(), "promotion_modal");
        let mut chosen = None;

        modal.show(|ui| {
            modal.title(ui, PawnPromotion.t());
            ui.horizontal(|ui| {
                for (piece, label) in [
                    (Piece::Queen, "♛"),
                    (Piece::Rook, "♜"),
                    (Piece::Bishop, "♝"),
                    (Piece::Knight, "♞"),
                ] {
                    if ui.button(RichText::new(label).size(24.0)).clicked() {
                        chosen = Some(piece);
                        modal.close();
                    }
                }
            });
            modal.buttons(ui, |ui| {
                if modal.button(ui, Cancel.t()).clicked() {
                    modal.close();
                }
            });
        });

        (modal, chosen)
    }
}

impl egui::Widget for ArenaWidget<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let Some(user_id) = self.store.me.value.as_ref().map(|v| v.public.id) else {
            return ui.spinner();
        };

        let Some(visuals) = self.state.visuals(user_id, self.store) else {
            return ui.spinner();
        };
        let board_visuals = visuals.board;

        let (promo_modal, chosen_piece) = Self::promotion_modal(ui);

        if let Some(piece) = chosen_piece {
            if let Some((from, to)) = self.state.pending_promotion.take() {
                self.state.handle_move(from, to, Some(piece), self.ws);
            }
        } else if !promo_modal.is_open() {
            self.state.pending_promotion = None;
        }

        ui.vertical(|ui| {
            let available_w = ui.available_width();
            let available_h = ui.available_height();

            let player_panel_h = 52.0;
            let player_count =
                visuals.top_player.is_some() as u8 + visuals.bottom_player.is_some() as u8;
            let budget_for_board = available_h - player_count as f32 * player_panel_h;

            let board_size = available_w.min(budget_for_board).max(0.0);

            if let Some((user_id, color)) = visuals.top_player
                && let Some(session_id) = self.state.session_id()
            {
                ArenaUser::new(session_id, user_id, color, self.images, self.store)
                    .width(board_size - 14.0)
                    .ui(ui);
            }

            let (can_move, restriction) = self.state.movement(self.store, user_id);
            BoardWidget::new(self.images, &board_visuals, "arena_board")
                .size(board_size)
                .can_move(can_move)
                .move_restriction(restriction)
                .ui(ui);

            if let Some((user_id, color)) = visuals.bottom_player
                && let Some(session_id) = self.state.session_id()
            {
                ArenaUser::new(session_id, user_id, color, self.images, self.store)
                    .width(board_size - 14.0)
                    .ui(ui);
            }

            if let Some(BoardAction::Move { from, to }) = board_action(ui, "arena_board") {
                if self.state.source.needs_promotion(from, to, self.store) {
                    self.state.pending_promotion = Some((from, to));
                    promo_modal.open();
                } else {
                    self.state.handle_move(from, to, None, self.ws);
                }
            }
        })
        .response
    }
}
