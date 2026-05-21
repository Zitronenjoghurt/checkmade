use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum Lingo {
    Categories,
    FetchingUserInfo,
    FriendRequestDeclined,
    FriendRequestReceived,
    FriendRequestSent,
    FriendRemoved,
    General,
    Latency,
    Logout,
    ServerTime,
    Settings,
    UiScale,
}
