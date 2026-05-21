use crate::client_time_ms;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

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

impl<K: Eq + Hash, V> Fetchable<HashMap<K, V>> {
    pub fn insert(&mut self, key: K, value: V) {
        self.value
            .get_or_insert_with(HashMap::default)
            .insert(key, value);
    }

    pub fn remove(&mut self, key: &K) {
        if let Some(map) = &mut self.value {
            map.remove(key);
        }
    }

    pub fn get_entry(&self, key: &K) -> Option<&V> {
        self.value.as_ref().and_then(|m| m.get(key))
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.value.as_ref().map_or(false, |m| m.contains_key(key))
    }
}

impl<T: Eq + Hash> Fetchable<HashSet<T>> {
    pub fn add(&mut self, item: T) {
        self.value.get_or_insert_with(HashSet::new).insert(item);
    }

    pub fn remove(&mut self, item: &T) {
        if let Some(set) = &mut self.value {
            set.remove(item);
        }
    }

    pub fn contains(&self, item: &T) -> bool {
        self.value.as_ref().map_or(false, |s| s.contains(item))
    }
}
