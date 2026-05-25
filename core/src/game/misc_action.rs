#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MiscAction {
    Resign,
    OfferDraw,
    AcceptDraw,
    DeclineDraw,
    ClaimDraw,
    ClaimTimeout,
}
