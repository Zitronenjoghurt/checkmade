use crate::client_time_ms;
use crate::event::{AppEvent, ReconnectedEvent};
use crate::ws::Ws;
use chrono::{DateTime, TimeZone, Utc};
use std::collections::VecDeque;
use web_time::Duration;

const SAMPLE_COUNT: usize = 18;
const HEARTBEAT_INTERVAL_MS: u64 = 5_000;
const HEARTBEAT_TIMEOUT_MS: u64 = 30_000;

#[derive(Debug, Copy, Clone)]
struct SyncSample {
    offset: i64,
    rtt: u64,
}

pub struct ServerTime {
    samples: VecDeque<SyncSample>,
    offset: i64,
    rtt: u64,
    pending_since: Option<u64>,
    last_pong: u64,
    last_ping: u64,
}

impl Default for ServerTime {
    fn default() -> Self {
        Self {
            samples: VecDeque::with_capacity(SAMPLE_COUNT),
            offset: 0,
            rtt: 0,
            pending_since: None,
            last_pong: client_time_ms(),
            last_ping: 0,
        }
    }
}

impl ServerTime {
    pub fn update(&mut self, ctx: &egui::Context, ws: &mut Ws) {
        if !ws.is_connected() {
            return;
        }

        if ReconnectedEvent::fired(ctx) {
            *self = Default::default();
        }

        let now = client_time_ms();
        if self.last_ping == 0 || now.saturating_sub(self.last_ping) >= HEARTBEAT_INTERVAL_MS {
            self.last_ping = now;
            self.pending_since = Some(now);
            ws.ping();
        }
    }

    pub fn handle_pong(&mut self, client_time: u64, server_time: u64) {
        let now = client_time_ms();
        self.last_pong = now;
        self.pending_since = None;

        let rtt = now.saturating_sub(client_time);
        let offset = (server_time as i64) - (client_time as i64) - ((rtt / 2) as i64);

        if self.samples.len() >= SAMPLE_COUNT {
            self.samples.pop_front();
        }
        self.samples.push_back(SyncSample { offset, rtt });

        self.recalculate();
    }

    fn recalculate(&mut self) {
        if self.samples.is_empty() {
            return;
        }

        let mut sorted: Vec<_> = self.samples.iter().copied().collect();
        sorted.sort_by_key(|s| s.rtt);
        sorted.truncate((SAMPLE_COUNT / 3).max(1));

        let len = sorted.len() as i64;
        let new_offset = sorted.iter().map(|s| s.offset).sum::<i64>() / len;
        let new_rtt = sorted.iter().map(|s| s.rtt).sum::<u64>() / (len as u64);

        if self.samples.len() <= 3 {
            self.offset = new_offset;
            self.rtt = new_rtt;
        } else {
            self.offset += (new_offset - self.offset) * 3 / 10;

            let rtt_diff = (new_rtt as i64) - (self.rtt as i64);
            self.rtt = (self.rtt as i64 + (rtt_diff * 3 / 10)) as u64;
        }
    }

    pub fn ready(&self) -> bool {
        !self.samples.is_empty()
    }

    pub fn is_timed_out(&self) -> bool {
        self.ready() && client_time_ms().saturating_sub(self.last_pong) >= HEARTBEAT_TIMEOUT_MS
    }

    pub fn to_local(&self, server_time: u64) -> u64 {
        (server_time as i64 - self.offset).max(0) as u64
    }

    pub fn to_local_datetime(&self, server_time: u64) -> DateTime<Utc> {
        Utc.timestamp_millis_opt(self.to_local(server_time) as i64)
            .unwrap()
    }

    pub fn now(&self) -> u64 {
        (client_time_ms() as i64 + self.offset).max(0) as u64
    }

    pub fn server_datetime(&self) -> DateTime<Utc> {
        let ms = self.now();
        Utc.timestamp_millis_opt(ms as i64).unwrap()
    }

    pub fn elapsed_since(&self, server_time: u64) -> Duration {
        let current_server_now = self.now();
        let elapsed_ms = current_server_now.saturating_sub(server_time);
        Duration::from_millis(elapsed_ms)
    }

    pub fn latency(&self) -> Duration {
        Duration::from_millis(self.rtt)
    }
}
