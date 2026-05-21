use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum Lingo {
    Categories,
    FetchingUserInfo,
    General,
    Latency,
    Logout,
    ServerTime,
    Settings,
    UiScale,
}
