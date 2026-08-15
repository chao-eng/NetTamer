//! Token-bucket rate limiter used to admit / drop packets per process.

use std::time::{Duration, Instant};

/// A classic token bucket. Tokens are measured in **bytes**; the refill rate is
/// therefore bytes/sec. One token == one byte.
#[derive(Debug, Clone)]
pub struct TokenBucket {
    /// Maximum token balance (== burst capacity, here = 1 second of bytes).
    capacity: f64,
    /// Current token balance.
    tokens: f64,
    /// Refill rate in bytes/sec (= `rate_bps / 8`).
    refill_rate: f64,
    last: Instant,
}

impl TokenBucket {
    /// Create a bucket allowing `rate_bps` bits/sec (`rate_bps / 8` bytes/sec).
    pub fn new(rate_bps: u64) -> Self {
        let rate = rate_bps as f64 / 8.0;
        Self {
            capacity: rate,
            tokens: rate,
            refill_rate: rate,
            last: Instant::now(),
        }
    }

    /// Attempt to consume `n` bytes. Returns `true` if permitted.
    pub fn try_consume(&mut self, n: usize) -> bool {
        self.refill();
        let need = n as f64;
        if self.tokens >= need {
            self.tokens -= need;
            true
        } else {
            false
        }
    }

    /// Update the rate limit, preserving (clamped) tokens proportionally.
    pub fn set_rate(&mut self, rate_bps: u64) {
        self.refill();
        let rate = rate_bps as f64 / 8.0;
        self.refill_rate = rate;
        self.capacity = rate;
        self.tokens = self.tokens.min(rate);
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last).as_secs_f64();
        if elapsed > 0.0 {
            self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
            self.last = now;
        }
    }
}

/// Convenience: convert `Duration` to a refill rate for tests/diagnostics.
#[allow(dead_code)]
pub fn bucket_for_duration(rate_bps: u64, _burst: Duration) -> TokenBucket {
    TokenBucket::new(rate_bps)
}

#[cfg(test)]
mod tests {
    use super::TokenBucket;

    #[test]
    fn allows_within_capacity() {
        let mut b = TokenBucket::new(8_000); // 1000 bytes/sec
        assert!(b.try_consume(500));
        assert!(b.try_consume(500));
        // 1000 bytes consumed; next should fail until refill.
        assert!(!b.try_consume(1));
    }
}
