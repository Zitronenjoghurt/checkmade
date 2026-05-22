use checkmade_core::game::visuals::BoardVisuals;
use checkmade_core::giga_chess::prelude::{Color, Game};

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct SandboxState {
    pub perspective: Color,
    pub game: Game,
}

impl SandboxState {
    pub fn visuals(&self) -> BoardVisuals {
        BoardVisuals::from_game(self.perspective, &self.game)
    }
}
