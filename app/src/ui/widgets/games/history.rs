use crate::event::{AppEvent, OpenSandboxEvent};
use crate::i18n::Translatable;
use crate::tl;
use crate::ui::icons;
use crate::ui::state::sandbox::SandboxState;
use crate::utils::fmt::{fmt_duration, fmt_outcome};
use checkmade_core::giga_chess::prelude::{Color, DecisiveReason, DrawReason, GameOutcome};
use checkmade_core::lingo::Lingo::{NoPastGames, XAgo};
use egui::{Frame, ScrollArea};

pub struct GamesHistory<'a> {
    server_time: &'a crate::server_time::ServerTime,
    store: &'a mut crate::store::Store,
    ws: &'a mut crate::ws::Ws,
}

impl<'a> GamesHistory<'a> {
    pub fn new(
        server_time: &'a crate::server_time::ServerTime,
        store: &'a mut crate::store::Store,
        ws: &'a mut crate::ws::Ws,
    ) -> Self {
        Self {
            server_time,
            store,
            ws,
        }
    }
}

impl egui::Widget for GamesHistory<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let Some(me_id) = self.store.me.value.as_ref().map(|i| i.public.id) else {
            return ui.spinner();
        };

        self.store.ensure_session_history(self.ws);
        let Some(sessions) = self.store.sessions.value.as_ref() else {
            return ui.spinner();
        };

        let mut history = sessions
            .values()
            .filter(|s| !s.is_ongoing())
            .collect::<Vec<_>>();
        history.sort_by_key(|b| std::cmp::Reverse(b.created));

        if history.is_empty() {
            return ui.label(NoPastGames.t());
        }

        ScrollArea::vertical()
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    for session in history {
                        let me_color = if session.black == me_id {
                            Color::Black
                        } else {
                            Color::White
                        };
                        let opponent_id = if session.white == me_id {
                            session.black
                        } else if session.black == me_id {
                            session.white
                        } else {
                            continue;
                        };
                        let opponent = self.store.users.get(opponent_id);
                        Frame::group(ui.style()).show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    if ui
                                        .button(outcome_icon(session.game().outcome(), me_color))
                                        .clicked()
                                    {
                                        OpenSandboxEvent(SandboxState {
                                            game: session.game().clone(),
                                            white_id: Some(session.white),
                                            black_id: Some(session.black),
                                            perspective: me_color,
                                        })
                                        .send(ui.ctx());
                                    }
                                    if let Some(opponent) = opponent {
                                        ui.label(format!("vs. {}", opponent.username));
                                    } else {
                                        ui.horizontal(|ui| {
                                            ui.label("vs.");
                                            ui.spinner();
                                        });
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.small(tl!(
                                        XAgo,
                                        x = fmt_duration(
                                            self.server_time.elapsed_since(session.updated)
                                        )
                                    ));
                                    if let Some(outcome) = session.game().outcome() {
                                        ui.separator();
                                        ui.small(fmt_outcome(&outcome));
                                    }
                                });
                            });
                        });
                    }
                })
                .response
            })
            .inner
    }
}

fn outcome_icon(outcome: Option<GameOutcome>, user_color: Color) -> String {
    let icons = match outcome {
        None => (icons::QUESTION, icons::CHECKERBOARD),
        Some(outcome) => match outcome {
            GameOutcome::Decisive { winner, reason } => {
                let first = if user_color == winner {
                    icons::TROPHY
                } else {
                    icons::SMILEY_SAD
                };
                let second = match reason {
                    DecisiveReason::Checkmate => icons::CROWN_CROSS,
                    DecisiveReason::Resignation => icons::FLAG,
                    DecisiveReason::Timeout => icons::HOURGLASS_LOW,
                };
                (first, second)
            }
            GameOutcome::Draw(reason) => {
                let first = icons::SCALES;
                let second = match reason {
                    DrawReason::Stalemate => icons::CASTLE_TURRET,
                    DrawReason::Agreement => icons::HANDSHAKE,
                    DrawReason::FiftyMoveRule | DrawReason::SeventyFiveMoveRule => {
                        icons::CLOCK_COUNTDOWN
                    }
                    DrawReason::ThreefoldRepetition | DrawReason::FivefoldRepetition => {
                        icons::REPEAT
                    }
                    DrawReason::InsufficientMaterial => icons::PLACEHOLDER,
                    DrawReason::TimeoutVsInsufficient => icons::HOURGLASS_LOW,
                };
                (first, second)
            }
        },
    };
    format!("{}{}", icons.0, icons.1)
}
