use strum::{EnumIter, FromRepr};

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, EnumIter, FromRepr)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(i16)]
pub enum PieceSet {
    #[default]
    Regular = 0,
}

impl PieceSet {
    pub fn name(&self) -> &'static str {
        match self {
            PieceSet::Regular => "regular",
        }
    }
}
