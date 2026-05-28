use checkmade_core::giga_chess::prelude::{ChessMove, Color, Game};
use checkmade_core::types::user_id::UserId;

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct SandboxState {
    pub game: Game,
    pub black_id: Option<UserId>,
    pub white_id: Option<UserId>,
    pub perspective: Color,
    pub san_history: Vec<String>,
    pub previous_lines: Vec<(Vec<ChessMove>, usize)>,
}

impl SandboxState {
    pub fn game_at_index(&self, move_index: usize) -> Game {
        let moves = &self.game.history()[..move_index];
        Game::from_moves(moves).unwrap_or_default()
    }

    pub fn top_player(&self) -> Option<(UserId, Color)> {
        if self.perspective == Color::White {
            Some((self.black_id?, Color::Black))
        } else {
            Some((self.white_id?, Color::White))
        }
    }

    pub fn bottom_player(&self) -> Option<(UserId, Color)> {
        if self.perspective == Color::White {
            Some((self.white_id?, Color::White))
        } else {
            Some((self.black_id?, Color::Black))
        }
    }

    pub fn fork_at(&mut self, move_index: usize) {
        self.previous_lines
            .push((self.game.history().to_vec(), move_index));

        let moves: Vec<_> = self.game.history()[..move_index].to_vec();
        self.game = Game::from_moves(&moves).unwrap();
        self.san_history.truncate(move_index);
    }

    pub fn restore_previous_line(&mut self) -> Option<usize> {
        let (line, fork_index) = self.previous_lines.pop()?;
        self.game = Game::from_moves(&line).unwrap();
        self.san_history.clear();
        let mut tmp = Game::default();
        for mv in &line {
            if let Ok(san) = tmp.play_move_get_san(*mv) {
                self.san_history.push(san);
            }
        }
        Some(fork_index)
    }
}
