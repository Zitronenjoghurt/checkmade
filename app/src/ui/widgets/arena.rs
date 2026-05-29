use crate::i18n::Translatable;
use crate::store::Store;
use crate::tl;
use crate::ui::icons;
use crate::ui::modal::Modal;
use crate::ui::state::analysis::AnalysisState;
use crate::ui::state::arena::{ArenaActions, ArenaSource, ArenaState};
use crate::ui::widgets::arena::move_list::{MoveListEvent, MoveListWidget};
use crate::ui::widgets::arena::user::ArenaUser;
use crate::ui::widgets::board::{board_action, BoardAction, BoardWidget};
use crate::utils::fmt::{fmt_color, fmt_outcome};
use crate::utils::images::Images;
use crate::ws::Ws;
use checkmade_core::game::misc_action::MiscAction;
use checkmade_core::giga_chess::prelude::{Color, Piece};
use checkmade_core::lingo::Lingo::*;
use egui::RichText;

mod move_list;
mod user;

pub struct ArenaWidget<'a> {
    analysis: &'a mut AnalysisState,
    images: &'a mut Images,
    server_time: &'a crate::server_time::ServerTime,
    settings: &'a crate::ui::state::settings::Settings,
    state: &'a mut ArenaState,
    store: &'a mut Store,
    ws: &'a mut Ws,
}

impl<'a> ArenaWidget<'a> {
    pub fn new(
        analysis: &'a mut AnalysisState,
        images: &'a mut Images,
        server_time: &'a crate::server_time::ServerTime,
        settings: &'a crate::ui::state::settings::Settings,
        state: &'a mut ArenaState,
        store: &'a mut Store,
        ws: &'a mut Ws,
    ) -> Self {
        Self {
            analysis,
            images,
            server_time,
            settings,
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
            MiscAction::ClaimDraw | MiscAction::ClaimTimeout => (ClaimDraw, ClaimDrawInfo),
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

    fn check_for_timeout_claim(&mut self, ui: &mut egui::Ui, now_ms: u64) {
        if let Some(session_id) = self.state.session_id() {
            let memory_id = ui.id().with("timeout_claimed").with(session_id);
            let last_claim_ms = ui.data(|d| d.get_temp::<u64>(memory_id).unwrap_or(0));
            if now_ms.saturating_sub(last_claim_ms) > 5000
                && self.state.outcome(self.store).is_none()
            {
                for color in [Color::White, Color::Black] {
                    if let Some((time_ms, _)) = self.state.time(self.store, color, now_ms)
                        && time_ms == 0
                    {
                        ui.data_mut(|d| d.insert_temp(memory_id, now_ms));
                        self.state
                            .handle_action(MiscAction::ClaimTimeout, self.store, self.ws);
                        break;
                    }
                }
            }
        }
    }
}

impl egui::Widget for ArenaWidget<'_> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let now_ms = self.server_time.now();
        self.check_for_timeout_claim(ui, now_ms);

        let Some(me_id) = self.store.me.value.as_ref().map(|v| v.public.id) else {
            return ui.spinner();
        };

        let Some(visuals) = self.state.visuals(me_id, self.store) else {
            return ui.spinner();
        };

        let mut board_visuals = visuals.board;
        if self.state.is_sandbox()
            && let Some(game) = self.state.current_game(self.store)
            && !game.is_over()
        {
            let moves = game.history();
            let current_ply = self
                .state
                .move_history
                .current_index()
                .unwrap_or((moves.len() as isize).max(0) as usize);
            self.analysis.sync_game(moves, current_ply, ui.ctx());
            if let Some(eval) = &self.analysis.eval
                && let Some((from, to)) = eval.best_move
            {
                board_visuals.best_move_from = Some(from);
                board_visuals.best_move_to = Some(to);
            }
        }

        let (promo_modal, chosen_piece) = Self::promotion_modal(ui);
        Self::confirm_modal(ui, self.state, self.store, self.ws);

        if let Some(piece) = chosen_piece {
            if let Some((from, to)) = self.state.pending_promotion.take() {
                self.state.handle_move(from, to, Some(piece), self.ws);
            }
        } else if !promo_modal.is_open() {
            self.state.pending_promotion = None;
        }

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

        let total_height = ui.available_height();
        let spacing = ui.spacing().item_spacing;

        let player_panel_h = 52.0;
        let player_count =
            visuals.top_player.is_some() as u8 + visuals.bottom_player.is_some() as u8;
        let player_spacing = if player_count > 0 {
            player_count as f32 * spacing.y
        } else {
            0.0
        };
        let budget_from_height =
            total_height - player_count as f32 * player_panel_h - player_spacing;
        let board_size = budget_from_height.max(0.0);

        let board_column_h = board_size + player_count as f32 * player_panel_h;

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_max_width(board_size);

                if let Some((user_id, color)) = visuals.top_player {
                    ArenaUser::new(self.state, user_id, color, now_ms, self.images, self.store)
                        .width(board_size - 14.0)
                        .ui(ui);
                }

                let (can_move, restriction) = self.state.movement(self.store, me_id);
                let mut board = BoardWidget::new(self.images, &board_visuals, "arena_board")
                    .size(board_size)
                    .can_move(can_move)
                    .move_restriction(restriction);
                if self.settings.display_legal_targets {
                    board = board.legal_targets_fn(|sq| self.state.legal_targets(sq, self.store));
                }
                board.ui(ui);

                if let Some((user_id, color)) = visuals.bottom_player {
                    ArenaUser::new(self.state, user_id, color, now_ms, self.images, self.store)
                        .width(board_size - 14.0)
                        .ui(ui);
                }

                if let Some(BoardAction::Move { from, to }) = board_action(ui, "arena_board") {
                    if self.state.source.needs_promotion(
                        from,
                        to,
                        self.store,
                        self.state.move_history.current_index(),
                    ) {
                        self.state.pending_promotion = Some((from, to));
                        promo_modal.open();
                    } else {
                        self.state.handle_move(from, to, None, self.ws);
                    }
                }
            });

            let frame = egui::Frame::group(ui.style());
            let frame_margins = frame.inner_margin.sum() + frame.outer_margin.sum();
            let inner_height = (board_column_h - frame_margins.y).max(0.0);

            frame.show(ui, |ui| {
                ui.set_height(inner_height);

                ui.vertical(|ui| {
                    let san_history = self.state.source.san_history(self.store).to_vec();
                    let total_moves = san_history.len();

                    let body_h = ui.text_style_height(&egui::TextStyle::Body);
                    let total_rounds = total_moves.div_ceil(2).max(1);
                    let num_w = ui.fonts_mut(|f| {
                        f.layout_no_wrap(
                            format!("{}.", total_rounds),
                            egui::FontId::monospace(body_h),
                            egui::Color32::WHITE,
                        )
                        .size()
                        .x
                    }) + 8.0;
                    let list_width = num_w + 56.0 * 2.0;

                    ui.set_max_width(list_width);
                    ui.set_max_height(board_column_h);

                    if let ArenaSource::Sandbox(sandbox) = &mut self.state.source {
                        ui.vertical_centered(|ui| {
                            ui.label("Sandbox Game");
                        });

                        ui.separator();

                        ui.horizontal(|ui| {
                            if ui.button(icons::DEVICE_ROTATE).clicked() {
                                sandbox.perspective = sandbox.perspective.opposite();
                            }
                            if !sandbox.previous_lines.is_empty()
                                && ui
                                    .button(format!(
                                        "{} ({})",
                                        icons::ARROW_U_UP_LEFT,
                                        sandbox.previous_lines.len()
                                    ))
                                    .clicked()
                                && let Some(fork_index) = sandbox.restore_previous_line()
                            {
                                let total = sandbox.san_history.len();
                                self.state.move_history.go_to(fork_index, total);
                            }
                        });
                        ui.separator();
                    }

                    let move_list =
                        MoveListWidget::new(&san_history, self.state.move_history.current_index());
                    if let Some(event) = move_list.show(ui) {
                        match event {
                            MoveListEvent::Move(idx) => {
                                self.state.move_history.go_to(idx, total_moves)
                            }
                            MoveListEvent::Back => self.state.move_history.go_back(total_moves),
                            MoveListEvent::Forward => {
                                self.state.move_history.go_forward(total_moves)
                            }
                            MoveListEvent::Start => self.state.move_history.go_to(0, total_moves),
                            MoveListEvent::Present => self.state.move_history.snap_to_present(),
                        }
                    }
                });
            });
        })
        .response
    }
}
