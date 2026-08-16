#![no_std]

use core::sync::atomic::{AtomicU64, Ordering};

pub const DEFAULT_HZ: u32 = 1000;

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

/// State used by the interrupt layer to acknowledge timer progress without
/// directly performing a context switch inside the low-level ISR.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TickDecision { Continue, RequestReschedule }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TimerScheduler {
    quantum: Quantum,
    pending: bool,
}

impl TimerScheduler {
    pub const fn new(quantum_ticks: u32) -> Self {
        Self { quantum: Quantum::new(quantum_ticks), pending: false }
    }
    pub const fn pending(&self) -> bool { self.pending }
    pub const fn remaining(&self) -> u32 { self.quantum.remaining() }

    /// Called exactly once for each timer interrupt after the hardware tick is
    /// acknowledged. It only records scheduling intent; the dispatcher may
    /// consume that intent at a safe context-switch boundary.
    pub fn on_tick(&mut self) -> TickDecision {
        if self.quantum.consume_tick() {
            self.pending = true;
            TickDecision::RequestReschedule
        } else {
            TickDecision::Continue
        }
    }

    pub fn take_reschedule(&mut self) -> bool {
        let was_pending = self.pending;
        self.pending = false;
        was_pending
    }

    pub fn reset_quantum(&mut self) { self.quantum.reset(); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn ticks_are_monotonic() {
        let timer = Timer::new(1000);
        assert_eq!(timer.tick(), 1); assert_eq!(timer.tick(), 2); assert_eq!(timer.elapsed_ms(), 2);
    }
    #[test] fn timer_scheduler_requests_then_consumes_reschedule() {
        let mut s = TimerScheduler::new(2);
        assert_eq!(s.on_tick(), TickDecision::Continue);
        assert_eq!(s.on_tick(), TickDecision::RequestReschedule);
        assert!(s.pending()); assert!(s.take_reschedule()); assert!(!s.pending());
        s.reset_quantum(); assert_eq!(s.remaining(), 2);
    }
    #[test] fn zero_quantum_requests_reschedule_safely() {
        let mut s = TimerScheduler::new(0);
        assert_eq!(s.on_tick(), TickDecision::RequestReschedule);
        assert!(s.take_reschedule());
    }
    #[test] fn zero_frequency_is_safe() {
        let timer = Timer::new(0); timer.tick(); assert_eq!(timer.elapsed_ms(), 0);
    }
}
