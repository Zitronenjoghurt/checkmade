use crate::types::user_id::UserId;

#[derive(Clone)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FriendInfo {
    pub user_id: UserId,
    pub since: u64,
}
