use crate::game::coords::BoardCoords;
use giga_chess::prelude::{ChessBoard, Color, Game, Piece, Square};

#[derive(Debug, Default, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoardVisuals {
    pub last_move_from: Option<Square>,
    pub last_move_to: Option<Square>,
    pub perspective: Color,
    pub pieces: Vec<PieceVisuals>,
    pub threat_targets: Vec<Square>,
    pub threat_sources: Vec<Square>,
}

impl BoardVisuals {
    pub fn from_game(perspective: Color, game: &Game) -> Self {
        let last_move = game.history().last();

        let mut threat_targets = Vec::new();
        let mut threat_sources = Vec::new();

        let white_threats = game.king_threats(Color::White);
        if !white_threats.is_empty() {
            let white_kings = game.position().board.piece_bb(Piece::King, Color::White);
            for king in white_kings {
                threat_targets.push(king);
            }
            threat_sources.extend(white_threats);
        }

        let black_threats = game.king_threats(Color::Black);
        if !black_threats.is_empty() {
            let black_kings = game.position().board.piece_bb(Piece::King, Color::Black);
            for king in black_kings {
                threat_targets.push(king);
            }
            threat_sources.extend(black_threats);
        }

        Self {
            last_move_from: last_move.map(|mv| mv.from()),
            last_move_to: last_move.map(|mv| mv.to()),
            perspective,
            pieces: PieceVisuals::from_board(&game.position().board),
            threat_targets,
            threat_sources,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PieceVisuals {
    pub color: Color,
    pub piece: Piece,
    pub coords: BoardCoords,
}

impl PieceVisuals {
    pub fn from_board(board: &ChessBoard) -> Vec<PieceVisuals> {
        let occupied = board.occupied_bb();
        let mut pieces = Vec::with_capacity(occupied.count_set() as usize);

        for sq in occupied {
            let Some((piece, color)) = board.piece_at(sq) else {
                continue;
            };
            pieces.push(PieceVisuals {
                color,
                piece,
                coords: sq.into(),
            })
        }

        pieces
    }
}
