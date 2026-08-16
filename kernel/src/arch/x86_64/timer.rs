#![no_std]

use core::sync::atomic::{AtomicU64, Ordering};

/// Monotonic kernel tick source. Time saturates instead of wrapping backwards.
pub struct Timer {
    ticks: AtomicU64,
    frequency_hz: u32,
}

impl Timer {
    pub const fn new(frequency_hz: u32) -> Self { Self { ticks: AtomicU64::new(0), frequency_hz } }
    pub const fn frequency_hz(&self) -> u32 { self.frequency_hz }

    #[inline]
    pub fn tick(&self) -> u64 {
        let mut current = self.ticks.load(Ordering::Relaxed);
        loop {
            if current == u64::MAX { return current; }
            match self.ticks.compare_exchange_weak(current, current + 1, Ordering::AcqRel, Ordering::Relaxed) {
                Ok(value) => return value + 1,
                Err(observed) => current = observed,
            }
        }
    }

    pub fn ticks(&self) -> u64 { self.ticks.load(Ordering::Acquire) }

    pub fn elapsed_ms(&self) -> u64 {
        if self.frequency_hz == 0 { return 0; }
        self.ticks().saturating_mul(1000) / self.frequency_hz as u64
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Quantum { remaining: u32, original: u32 }

impl Quantum {
    pub const fn new(ticks: u32) -> Self { Self { remaining: ticks, original: ticks } }
    pub const fn remaining(&self) -> u32 { self.remaining }
    pub const fn expired(&self) -> bool { self.remaining == 0 }
    pub fn consume_tick(&mut self) -> bool {
        if self.remaining == 0 { return true; }
        self.remaining -= 1;
        self.remaining == 0
    }
    pub fn reset(&mut self) { self.remaining = self.original; }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ticks_are_monotonic() {
        let timer = Timer::new(1000);
        assert_eq!(timer.tick(), 1);
        assert_eq!(timer.tick(), 2);
        assert_eq!(timer.elapsed_ms(), 2);
    }
    #[test]
    fn quantum_expires_exactly() {
        let mut q = Quantum::new(3);
        assert!(!q.consume_tick());
        assert!(!q.consume_tick());
        assert!(q.consume_tick());
        assert!(q.expired());
        q.reset();
        assert_eq!(q.remaining(), 3);
    }
    #[test]
    fn zero_frequency_is_safe() {
        let timer = Timer::new(0);
        timer.tick();
        assert_eq!(timer.elapsed_ms(), 0);
    }
}
