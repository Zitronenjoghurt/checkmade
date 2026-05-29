use crate::event::{AppEvent, SfCommandEvent, SfResponseEvent};
use checkmade_core::giga_chess::stockfish::command::SfCommand;
use checkmade_core::giga_chess::stockfish::event::SfEvent;
use checkmade_core::giga_chess::stockfish::StockfishManager;

pub mod stockfish;

pub trait Engine {
    fn send(&self, cmd: &str);
    fn drain(&self) -> Vec<String>;
}

pub struct StockFish {
    engine: Box<dyn Engine>,
    manager: StockfishManager,
}

impl Default for StockFish {
    fn default() -> Self {
        Self::init()
    }
}

impl StockFish {
    #[cfg(target_arch = "wasm32")]
    pub fn init() -> Self {
        Self {
            engine: Box::new(stockfish::StockfishEngine::new()),
            manager: StockfishManager::default(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn init() -> Self {
        unimplemented!()
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        for SfCommandEvent(cmd) in SfCommandEvent::recv(ctx) {
            self.send_command(cmd);
        }

        for event in self.poll() {
            SfResponseEvent(event).send(ctx);
        }
    }

    pub fn send_command(&mut self, cmd: SfCommand) {
        self.manager.send(cmd);
    }

    pub fn poll(&mut self) -> Vec<SfEvent> {
        for cmd in self.manager.drain_commands() {
            self.engine.send(&cmd.to_string());
        }
        self.engine
            .drain()
            .iter()
            .filter_map(|line| {
                self.manager.read_line(line).unwrap_or_else(|e| {
                    log::warn!("stockfish parse error: {e}");
                    None
                })
            })
            .collect()
    }
}
