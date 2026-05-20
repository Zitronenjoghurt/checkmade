use crate::config::Config;
use checkmade_core::messages::client::ClientMessage;
use std::time::Instant;

pub struct RateLimiter {
    tokens: f64,
    last_refill: Instant,
    violations: u32,
    warned: bool,
}

#[derive(Debug)]
pub enum RateLimitResult {
    Allow,
    Drop,
    Warn,
    Disconnect,
}

impl RateLimiter {
    pub fn new(config: &Config) -> Self {
        Self {
            tokens: config.ws_rate_limit_max_tokens,
            last_refill: Instant::now(),
            violations: 0,
            warned: false,
        }
    }

    fn refill(&mut self, config: &Config) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * config.ws_rate_limit_refill_rate)
            .min(config.ws_rate_limit_max_tokens);
        self.last_refill = now;
    }

    pub fn check(&mut self, config: &Config, message: &ClientMessage) -> RateLimitResult {
        self.refill(config);

        let cost = message.cost();

        if self.tokens >= cost {
            self.tokens -= cost;
            self.violations = self.violations.saturating_sub(1);
            return RateLimitResult::Allow;
        };

        self.violations += 1;

        match self.violations {
            1..=20 => RateLimitResult::Drop,
            _ => {
                if !self.warned {
                    self.warned = true;
                    self.violations = 0;
                    RateLimitResult::Warn
                } else {
                    RateLimitResult::Disconnect
                }
            }
        }
    }
}
