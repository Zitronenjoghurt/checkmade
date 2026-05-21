use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum Lingo {
    AddFriend,
    Categories,
    FetchingUserInfo,
    FriendRequestDeclined,
    FriendRequestReceived,
    FriendRequestSent,
    FriendRemoved,
    Friends,
    FriendAdded,
    FriendCode,
    General,
    Latency,
    List,
    Logout,
    NoFriends,
    NoFriendRequests,
    Pending,
    Requests,
    ServerTime,
    Settings,
    UiScale,
    XAgo,
}
