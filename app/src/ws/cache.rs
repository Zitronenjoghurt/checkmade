use crate::ws::fetchable::Fetchable;
use egui::ahash::HashMap;
use std::hash::Hash;

pub struct FetchableCache<K, V> {
    entries: HashMap<K, Fetchable<V>>,
    last_fetch: web_time::Instant,
    cooldown: web_time::Duration,
}

impl<K: Eq + Hash, V> Default for FetchableCache<K, V> {
    fn default() -> Self {
        Self {
            entries: HashMap::default(),
            last_fetch: web_time::Instant::now(),
            cooldown: web_time::Duration::from_millis(500),
        }
    }
}

impl<K: Eq + Hash + Copy, V> FetchableCache<K, V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fetch_cooldown(mut self, cooldown: web_time::Duration) -> Self {
        self.cooldown = cooldown;
        self
    }

    pub fn update(&mut self, mut fetcher: impl FnMut(K)) {
        if self.last_fetch.elapsed() < self.cooldown {
            return;
        }
        self.last_fetch = web_time::Instant::now();

        let Some((id, entry)) = self.entries.iter_mut().find(|(_, v)| v.needs_fetch()) else {
            return;
        };

        let id = *id;
        entry.request_if_needed(|| fetcher(id));
    }

    pub fn get(&mut self, key: K) -> Option<&V> {
        self.entries.entry(key).or_default().value.as_ref()
    }

    pub fn set(&mut self, key: K, value: V) {
        self.entries.entry(key).or_default().set_value(value);
    }

    pub fn invalidate(&mut self) {
        for entry in self.entries.values_mut() {
            entry.invalidate();
        }
    }
}
