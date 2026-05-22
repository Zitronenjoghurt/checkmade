use strum::FromRepr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(i16)]
pub enum IdentityProvider {
    Discord = 0,
    Google = 1,
    GitHub = 2,
    Mastodon = 3,
}
