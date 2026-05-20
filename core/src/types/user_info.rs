use crate::types::permissions::Permissions;

#[derive(Clone)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PublicUserInfo {
    pub username: String,
}

#[derive(Clone)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PrivateUserInfo {
    pub public: PublicUserInfo,
    pub permissions: i64,
}

impl PrivateUserInfo {
    pub fn permissions(&self) -> Permissions {
        Permissions::from_bits_truncate(self.permissions)
    }
}
