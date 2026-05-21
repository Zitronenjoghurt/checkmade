use uuid::Uuid;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UserId([u8; 16]);

impl From<Uuid> for UserId {
    fn from(value: Uuid) -> Self {
        Self(value.into_bytes())
    }
}

impl From<UserId> for Uuid {
    fn from(value: UserId) -> Self {
        Self::from_bytes(value.0)
    }
}
