use crate::game::coords::BoardCoords;
use giga_chess::prelude::{ChessBoard, Color, Piece, Square};

#[derive(Debug, Default, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoardVisuals {
    pub last_move_from: Option<Square>,
    pub last_move_to: Option<Square>,
    pub perspective: Color,
    pub pieces: Vec<PieceVisuals>,
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
