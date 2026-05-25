use crate::types::user_id::UserId;

#[derive(Clone)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FriendInfo {
    pub user_id: UserId,
    pub since: u64,
    pub times_won: u64,
    pub times_lost: u64,
    pub times_drawn: u64,
}

#[derive(Clone)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FriendRequestInfo {
    pub user_id: UserId,
    pub created: u64,
}
