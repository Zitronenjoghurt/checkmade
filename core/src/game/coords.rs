use giga_chess::prelude::Square;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoardCoords {
    pub x: i16,
    pub y: i16,
}

impl From<Square> for BoardCoords {
    fn from(sq: Square) -> Self {
        Self {
            x: ((sq.file() - 1) as i16 * 2 - 7) << 11,
            y: ((sq.rank() - 1) as i16 * 2 - 7) << 11,
        }
    }
}
