use checkmade_core::giga_chess::prelude::{Color, Game};
use checkmade_core::types::user_id::UserId;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct SandboxState {
    pub game: Game,
    pub black_id: Option<UserId>,
    pub white_id: Option<UserId>,
    pub perspective: Color,
}

impl SandboxState {
    pub fn top_player(&self) -> Option<(UserId, Color)> {
        if self.perspective == Color::White {
            Some((self.black_id?, Color::White))
        } else {
            Some((self.white_id?, Color::Black))
        }
    }

    pub fn bottom_player(&self) -> Option<(UserId, Color)> {
        if self.perspective == Color::White {
            Some((self.white_id?, Color::Black))
        } else {
            Some((self.black_id?, Color::White))
        }
    }
}
