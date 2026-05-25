use crate::i18n::Translatable;
use crate::store::Store;
use crate::tl;
use crate::ui::icons;
use crate::ui::modal::Modal;
use crate::ui::state::arena::{ArenaActions, ArenaState};
use crate::ui::widgets::arena::user::ArenaUser;
use crate::ui::widgets::board::{board_action, BoardAction, BoardWidget};
use crate::utils::fmt::{fmt_color, fmt_outcome};
use crate::utils::images::Images;
use crate::ws::Ws;
use checkmade_core::game::misc_action::MiscAction;
use checkmade_core::giga_chess::prelude::Piece;
use checkmade_core::lingo::Lingo::*;
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

    fn confirm_modal(ui: &mut egui::Ui, state: &mut ArenaState, store: &Store, ws: &mut Ws) {
        let modal = Modal::new(ui.ctx(), "action_confirm_modal");

        let Some(action) = state.pending_action.take() else {
            return;
        };

        let (title, description) = match &action {
            MiscAction::Resign => (Resign, ResignInfo),
            MiscAction::OfferDraw => (OfferDraw, OfferDrawInfo),
            MiscAction::AcceptDraw => (AcceptDraw, AcceptDrawInfo),
            MiscAction::DeclineDraw => (DeclineDraw, DeclineDrawInfo),
            MiscAction::ClaimDraw => (ClaimDraw, ClaimDrawInfo),
        };

        let mut confirmed = false;
        let mut cancelled = false;

        modal.show(|ui| {
            modal.title(ui, title.t());
            ui.label(description.t());
            modal.buttons(ui, |ui| {
                if modal.caution_button(ui, Confirm.t()).clicked() {
                    confirmed = true;
                    modal.close();
                }
                if modal.suggested_button(ui, Cancel.t()).clicked() {
                    cancelled = true;
                    modal.close();
                }
            });
        });

        if confirmed {
            state.handle_action(action, store, ws);
        } else if !cancelled && modal.is_open() {
            state.pending_action = Some(action);
        }
    }

    fn action_buttons(ui: &mut egui::Ui, actions: &ArenaActions, state: &mut ArenaState) {
        let modal = Modal::new(ui.ctx(), "action_confirm_modal");

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            if actions.can_accept_or_decline_draw {
                if ui
                    .small_button(format!("{} {}", icons::CHECK_CIRCLE, AcceptDraw.t()))
                    .clicked()
                {
                    state.pending_action = Some(MiscAction::AcceptDraw);
                    modal.open();
                }
                if ui
                    .small_button(format!("{} {}", icons::X_CIRCLE, DeclineDraw.t()))
                    .clicked()
                {
                    state.pending_action = Some(MiscAction::DeclineDraw);
                    modal.open();
                }
            }

            if actions.can_claim_draw
                && ui
                    .small_button(format!("{} {}", icons::GAVEL, ClaimDraw.t()))
                    .clicked()
            {
                state.pending_action = Some(MiscAction::ClaimDraw);
                modal.open();
            }

            if actions.can_offer_draw
                && !actions.can_accept_or_decline_draw
                && ui
                    .small_button(format!("{} {}", icons::HANDSHAKE, OfferDraw.t()))
                    .clicked()
            {
                state.pending_action = Some(MiscAction::OfferDraw);
                modal.open();
            }

            if actions.can_resign
                && ui
                    .small_button(format!("{} {}", icons::SKULL, Resign.t()))
                    .clicked()
            {
                state.pending_action = Some(MiscAction::Resign);
                modal.open();
            }
        });
    }
}

impl egui::Widget for ArenaWidget<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let Some(me_id) = self.store.me.value.as_ref().map(|v| v.public.id) else {
            return ui.spinner();
        };

        let Some(visuals) = self.state.visuals(me_id, self.store) else {
            return ui.spinner();
        };
        let board_visuals = visuals.board;

        let (promo_modal, chosen_piece) = Self::promotion_modal(ui);
        Self::confirm_modal(ui, self.state, self.store, self.ws);

        if let Some(piece) = chosen_piece {
            if let Some((from, to)) = self.state.pending_promotion.take() {
                self.state.handle_move(from, to, Some(piece), self.ws);
            }
        } else if !promo_modal.is_open() {
            self.state.pending_promotion = None;
        }

        ui.vertical(|ui| {
            if let Some(outcome) = self.state.outcome(self.store) {
                ui.heading(fmt_outcome(&outcome));
            } else if let Some(color) = self.state.color_to_move(self.store) {
                ui.horizontal(|ui| {
                    ui.heading(tl!(XToMove, x = fmt_color(color)));
                    let actions = self.state.actions(self.store, me_id);
                    Self::action_buttons(ui, &actions, self.state);
                });
            } else {
                ui.heading(OngoingGame.t().to_string());
            };
            ui.separator();

            let available_w = ui.available_width();
            let available_h = ui.available_height();

            let player_panel_h = 52.0;
            let player_count =
                visuals.top_player.is_some() as u8 + visuals.bottom_player.is_some() as u8;
            let budget_for_board = available_h - player_count as f32 * player_panel_h;

            let board_size = available_w.min(budget_for_board).max(0.0);

            if let Some((user_id, color)) = visuals.top_player {
                ArenaUser::new(self.state, user_id, color, self.images, self.store)
                    .width(board_size - 14.0)
                    .ui(ui);
            }

            let (can_move, restriction) = self.state.movement(self.store, me_id);
            BoardWidget::new(self.images, &board_visuals, "arena_board")
                .size(board_size)
                .can_move(can_move)
                .move_restriction(restriction)
                .ui(ui);

            if let Some((user_id, color)) = visuals.bottom_player {
                ArenaUser::new(self.state, user_id, color, self.images, self.store)
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
