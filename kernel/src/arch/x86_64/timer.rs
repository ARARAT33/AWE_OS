#![no_std]

use core::sync::atomic::{AtomicU64, Ordering};

pub const DEFAULT_HZ: u32 = 1000;

pub struct Timer {
    ticks: AtomicU64,
    frequency_hz: u32,
}

impl Timer {
    pub const fn new(frequency_hz: u32) -> Self {
        Self {
            ticks: AtomicU64::new(0),
            frequency_hz,
        }
    }
    pub const fn frequency_hz(&self) -> u32 {
        self.frequency_hz
    }
    #[inline]
    pub fn tick(&self) -> u64 {
        let mut current = self.ticks.load(Ordering::Relaxed);
        loop {
            if current == u64::MAX {
                return current;
            }
            match self.ticks.compare_exchange_weak(
                current,
                current + 1,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(value) => return value + 1,
                Err(observed) => current = observed,
            }
        }
    }
    pub fn ticks(&self) -> u64 {
        self.ticks.load(Ordering::Acquire)
    }
    pub fn elapsed_ms(&self) -> u64 {
        if self.frequency_hz == 0 {
            return 0;
        }
        self.ticks().saturating_mul(1000) / self.frequency_hz as u64
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Quantum {
    remaining: u32,
    original: u32,
}

impl Quantum {
    pub const fn new(ticks: u32) -> Self {
        Self {
            remaining: ticks,
            original: ticks,
        }
    }
    pub const fn remaining(&self) -> u32 {
        self.remaining
    }
    pub const fn expired(&self) -> bool {
        self.remaining == 0
    }
    pub fn consume_tick(&mut self) -> bool {
        if self.remaining == 0 {
            return true;
        }
        self.remaining -= 1;
        self.remaining == 0
    }
    pub fn reset(&mut self) {
        self.remaining = self.original;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TickDecision {
    Continue,
    RequestReschedule,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TimerScheduler {
    quantum: Quantum,
    pending: bool,
}

impl TimerScheduler {
    pub const fn new(quantum_ticks: u32) -> Self {
        Self {
            quantum: Quantum::new(quantum_ticks),
            pending: false,
        }
    }
    pub const fn pending(&self) -> bool {
        self.pending
    }
    pub const fn remaining(&self) -> u32 {
        self.quantum.remaining()
    }
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
    pub fn reset_quantum(&mut self) {
        self.quantum.reset();
    }
}

/// Deterministic bridge used by the timer ISR. The ISR calls this function
/// after acknowledging hardware; it performs only atomic accounting and
/// scheduler-state transition, never a context switch.
static TIMER_TICKS: AtomicU64 = AtomicU64::new(0);
static RESCHEDULE_REQUESTS: AtomicU64 = AtomicU64::new(0);

pub fn interrupt_tick() -> TickDecision {
    TIMER_TICKS.fetch_add(1, Ordering::Relaxed);
    if RESCHEDULE_REQUESTS.load(Ordering::Relaxed) == u64::MAX {
        return TickDecision::RequestReschedule;
    }
    let previous = RESCHEDULE_REQUESTS.fetch_add(1, Ordering::Relaxed);
    if previous == 0 {
        TickDecision::Continue
    } else {
        TickDecision::RequestReschedule
    }
}

pub fn interrupt_ticks() -> u64 {
    TIMER_TICKS.load(Ordering::Acquire)
}
pub fn pending_interrupt_requests() -> u64 {
    RESCHEDULE_REQUESTS.load(Ordering::Acquire)
}
pub fn consume_interrupt_request() -> bool {
    let mut current = RESCHEDULE_REQUESTS.load(Ordering::Acquire);
    loop {
        if current == 0 {
            return false;
        }
        match RESCHEDULE_REQUESTS.compare_exchange_weak(
            current,
            current - 1,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => return true,
            Err(observed) => current = observed,
        }
    }
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
    fn timer_scheduler_requests_then_consumes() {
        let mut s = TimerScheduler::new(2);
        assert_eq!(s.on_tick(), TickDecision::Continue);
        assert_eq!(s.on_tick(), TickDecision::RequestReschedule);
        assert!(s.pending());
        assert!(s.take_reschedule());
        assert!(!s.pending());
    }
    #[test]
    fn zero_quantum_requests_safely() {
        let mut s = TimerScheduler::new(0);
        assert_eq!(s.on_tick(), TickDecision::RequestReschedule);
        assert!(s.take_reschedule());
    }
    #[test]
    fn interrupt_bridge_records_ticks() {
        let before = interrupt_ticks();
        let _ = interrupt_tick();
        assert_eq!(interrupt_ticks(), before + 1);
        assert!(pending_interrupt_requests() > 0);
        assert!(consume_interrupt_request());
    }
    #[test]
    fn zero_frequency_is_safe() {
        let timer = Timer::new(0);
        timer.tick();
        assert_eq!(timer.elapsed_ms(), 0);
    }
}
