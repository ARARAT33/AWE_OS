#![no_std]

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Duration(pub u64);

impl Instant {
    pub const fn after(self, duration: Duration) -> Self {
        Self(self.0.saturating_add(duration.0))
    }

    pub const fn elapsed_since(self, earlier: Instant) -> Duration {
        Duration(self.0.saturating_sub(earlier.0))
    }
}

/// Monotonic kernel clock primitive. Hardware timer drivers advance it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonotonicClock {
    ticks: u64,
}

impl Default for MonotonicClock {
    fn default() -> Self {
        Self::new()
    }
}

impl MonotonicClock {
    pub const fn new() -> Self {
        Self { ticks: 0 }
    }
    pub const fn now(&self) -> Instant {
        Instant(self.ticks)
    }

    pub fn advance(&mut self, ticks: u64) {
        self.ticks = self.ticks.saturating_add(ticks);
    }

    pub fn reset_for_test(&mut self) {
        self.ticks = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_is_monotonic_and_saturating() {
        let mut clock = MonotonicClock::new();
        clock.advance(10);
        let start = clock.now();
        clock.advance(5);
        assert_eq!(clock.now().elapsed_since(start), Duration(5));
        clock.advance(u64::MAX);
        assert_eq!(clock.now(), Instant(u64::MAX));
    }
}
