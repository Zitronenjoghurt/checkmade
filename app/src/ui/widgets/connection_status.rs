use crate::i18n::Translatable;
use crate::server_time::ServerTime;
use crate::ui::icons;
use crate::ws::Ws;
use checkmade_core::lingo::Lingo::*;
use egui::{Color32, Response, RichText, Ui, Widget};

pub struct ConnectionStatus<'a> {
    st: &'a ServerTime,
    ws: &'a Ws,
}

impl<'a> ConnectionStatus<'a> {
    pub fn new(st: &'a ServerTime, ws: &'a Ws) -> Self {
        Self { st, ws }
    }
}

impl Widget for ConnectionStatus<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            if self.ws.is_connected() {
                let latency_ms = self.st.latency().as_millis();
                let (icon, color) = match latency_ms {
                    0..=100 => (icons::CELL_SIGNAL_FULL, Color32::DARK_GREEN),
                    101..=200 => (icons::CELL_SIGNAL_HIGH, Color32::YELLOW),
                    201..=500 => (icons::CELL_SIGNAL_MEDIUM, Color32::ORANGE),
                    _ => (icons::CELL_SIGNAL_LOW, Color32::RED),
                };
                ui.label(RichText::new(icon).size(14.0).color(color));
                ui.label(Connected.t());
                ui.label(format!("{}ms", latency_ms));
            } else {
                ui.label(
                    RichText::new(icons::CELL_SIGNAL_X)
                        .size(14.0)
                        .color(Color32::RED),
                );
                ui.label(Disconnected.t());
            }
        })
        .response
    }
}
