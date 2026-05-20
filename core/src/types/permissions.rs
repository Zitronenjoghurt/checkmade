use bitflags::bitflags;

bitflags! {
    pub struct Permissions: i64 { }
}

impl Default for Permissions {
    fn default() -> Self {
        Self::empty()
    }
}
