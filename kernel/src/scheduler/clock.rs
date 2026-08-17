#![no_std]

/// Deterministic tick clock used by the scheduler until a platform timer is
/// attached. Saturating arithmetic prevents time accounting from wrapping.
#[derive(Clone, Copy, Default)]
pub struct TickClock {
    ticks: u64,
}

impl TickClock {
    pub const fn new() -> Self {
        Self { ticks: 0 }
    }
    pub const fn now(&self) -> u64 {
        self.ticks
    }
    pub fn advance(&mut self, ticks: u64) {
        self.ticks = self.ticks.saturating_add(ticks);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_never_wraps() {
        let mut c = TickClock::new();
        c.advance(u64::MAX);
        c.advance(1);
        assert_eq!(c.now(), u64::MAX);
    }
}
