use crate::i18n::Translatable;
use crate::ui::tabs::TabViewer;
use crate::ui::widgets::games::bar::GamesBar;
use crate::ui::widgets::games::create::GamesCreate;
use crate::ui::widgets::games::history::GamesHistory;
use crate::ui::widgets::games::incoming::GamesIncoming;
use crate::ui::widgets::games::ongoing::GamesOngoing;
use crate::ui::widgets::games::outgoing::GamesOutgoing;
use crate::ui::widgets::games::GamesTab;
use checkmade_core::lingo::Lingo::ComingSoon;
use egui::Widget;

pub fn show(v: &mut TabViewer, ui: &mut egui::Ui) {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                GamesBar::new(
                    &mut v.state.games_tab,
                    v.store.active_sessions_to_move_count(),
                    v.store.game_request_count(),
                )
                .ui(ui);

                ui.separator();

                match v.state.games_tab {
                    GamesTab::Ongoing => {
                        GamesOngoing::new(v.server_time, v.store).ui(ui);
                    }
                    GamesTab::History => {
                        GamesHistory::new(v.server_time, v.store, v.ws).ui(ui);
                    }
                    GamesTab::Create => {
                        GamesCreate::new(&mut v.state.session_create, v.store, v.ws).ui(ui);
                    }
                    GamesTab::Incoming => {
                        GamesIncoming::new(v.store, v.ws).ui(ui);
                    }
                    GamesTab::Outgoing => {
                        GamesOutgoing::new(v.store, v.ws).ui(ui);
                    }
                    GamesTab::PublicGames => {
                        ui.strong(ComingSoon.t());
                    }
                    GamesTab::PublicRequests => {
                        ui.strong(ComingSoon.t());
                    }
                }
            });
        });
}
