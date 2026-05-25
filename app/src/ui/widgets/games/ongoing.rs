use crate::event::{AppEvent, OpenSessionEvent};
use crate::i18n::Translatable;
use crate::tl;
use crate::ui::icons;
use crate::ui::widgets::with_badge::WithBadge;
use crate::utils::fmt::fmt_duration;
use checkmade_core::lingo::Lingo::*;
use egui::{Button, Frame, Response, ScrollArea, Ui};

pub struct GamesOngoing<'a> {
    server_time: &'a crate::server_time::ServerTime,
    store: &'a mut crate::store::Store,
}

impl<'a> GamesOngoing<'a> {
    pub fn new(
        server_time: &'a crate::server_time::ServerTime,
        store: &'a mut crate::store::Store,
    ) -> Self {
        Self { server_time, store }
    }
}

impl egui::Widget for GamesOngoing<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Some(sessions) = self.store.sessions.value.as_ref() else {
            return ui.spinner();
        };

        let mut active = sessions
            .values()
            .filter(|s| s.is_ongoing())
            .collect::<Vec<_>>();
        active.sort_by_key(|b| b.updated);

        let Some(me) = self.store.me.value.as_ref() else {
            return ui.spinner();
        };

        if sessions.is_empty() {
            return ui.label(NoOngoingGames.t());
        };

        ScrollArea::vertical()
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    for session in active {
                        let opponent_id = if session.white == me.public.id {
                            session.black
                        } else if session.black == me.public.id {
                            session.white
                        } else {
                            continue;
                        };
                        let opponent = self.store.users.get(opponent_id);
                        Frame::group(ui.style()).show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        let play_button = ui.add(
                                            WithBadge::new(Button::new(icons::CHECKERBOARD))
                                                .dot(session.can_move(me.public.id)),
                                        );
                                        if play_button.clicked() {
                                            OpenSessionEvent(session.id).send(ui.ctx());
                                        }
                                        if let Some(opponent) = opponent {
                                            ui.label(format!("vs. {}", opponent.username));
                                        } else {
                                            ui.horizontal(|ui| {
                                                ui.label("vs.");
                                                ui.spinner();
                                            });
                                        }
                                        ui.separator();
                                        if session.can_move(me.public.id) {
                                            ui.strong(YourTurn.t());
                                        } else {
                                            ui.strong(format!("{}...", WaitingForOpponent.t()));
                                        }
                                    });
                                    ui.separator();
                                    ui.horizontal(|ui| {
                                        ui.small(format!("{} {}", Turn.t(), session.move_count()));
                                        if session.move_count() > 1 {
                                            ui.separator();
                                            ui.small(format!(
                                                "{}: {}",
                                                LastTurn.t(),
                                                tl!(
                                                    XAgo,
                                                    x = fmt_duration(
                                                        self.server_time
                                                            .elapsed_since(session.updated)
                                                    )
                                                )
                                            ));
                                        }
                                    });
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
