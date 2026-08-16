#![no_std]

/// Allocation-free token bucket for bounding repeated privileged operations.
pub struct TokenBucket {
    capacity: u64,
    tokens: u64,
    refill_per_tick: u64,
    last_tick: u64,
}

impl TokenBucket {
    pub const fn new(capacity: u64, refill_per_tick: u64) -> Self {
        Self { capacity, tokens: capacity, refill_per_tick, last_tick: 0 }
    }

    pub fn allow(&mut self, now: u64, cost: u64) -> bool {
        if now > self.last_tick {
            let elapsed = now - self.last_tick;
            let refill = elapsed.saturating_mul(self.refill_per_tick);
            self.tokens = self.tokens.saturating_add(refill).min(self.capacity);
            self.last_tick = now;
        }
        if cost > self.tokens { return false; }
        self.tokens -= cost;
        true
    }

    pub const fn available(&self) -> u64 { self.tokens }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_limits_bursts_and_refills() {
        let mut b = TokenBucket::new(2, 1);
        assert!(b.allow(0, 1));
        assert!(b.allow(0, 1));
        assert!(!b.allow(0, 1));
        assert!(b.allow(1, 1));
        assert_eq!(b.available(), 0);
    }
}
