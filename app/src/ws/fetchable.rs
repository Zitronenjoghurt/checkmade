use crate::client_time_ms;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum FetchState {
    #[default]
    Idle,
    Pending {
        since: u64,
    },
    Done {
        at: u64,
    },
}

pub struct Fetchable<T> {
    pub value: Option<T>,
    state: FetchState,
    refetch_interval: Option<u64>,
}

impl<T> Default for Fetchable<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Fetchable<T> {
    pub fn new() -> Self {
        Self {
            value: None,
            state: FetchState::Idle,
            refetch_interval: None,
        }
    }

    pub fn with_refetch(mut self, interval_ms: u64) -> Self {
        self.refetch_interval = Some(interval_ms);
        self
    }

    pub fn needs_fetch(&self) -> bool {
        match &self.state {
            FetchState::Idle => true,
            FetchState::Pending { .. } => false,
            FetchState::Done { at } => match self.refetch_interval {
                Some(interval) => client_time_ms() - at >= interval,
                None => false,
            },
        }
    }

    pub fn request_if_needed(&mut self, request_action: impl FnOnce()) {
        if self.needs_fetch() {
            self.state = FetchState::Pending {
                since: client_time_ms(),
            };
            request_action();
        }
    }

    pub fn set_value(&mut self, value: T) {
        self.value = Some(value);
        self.state = FetchState::Done {
            at: client_time_ms(),
        };
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.state, FetchState::Pending { .. })
    }

    pub fn state(&self) -> &FetchState {
        &self.state
    }
}
