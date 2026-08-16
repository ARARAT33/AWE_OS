#![no_std]

pub mod clock;
pub mod dispatch;
pub mod priority;

use crate::process::ProcessId;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RunQueue<const N: usize> {
    items: [Option<ProcessId>; N],
    head: usize,
    tail: usize,
    len: usize,
}

impl<const N: usize> RunQueue<N> {
    pub const fn new() -> Self { Self { items: [None; N], head: 0, tail: 0, len: 0 } }
    pub const fn len(&self) -> usize { self.len }
    pub const fn is_empty(&self) -> bool { self.len == 0 }
    pub fn push(&mut self, id: ProcessId) -> bool {
        if N == 0 || self.len == N || self.contains(id) { return false; }
        self.items[self.tail] = Some(id); self.tail = (self.tail + 1) % N; self.len += 1; true
    }
    pub fn pop(&mut self) -> Option<ProcessId> {
        if self.len == 0 { return None; }
        let id = self.items[self.head].take(); self.head = (self.head + 1) % N; self.len -= 1; id
    }
    fn contains(&self, id: ProcessId) -> bool {
        let mut i = 0; while i < self.len { if self.items[(self.head + i) % N] == Some(id) { return true; } i += 1; } false
    }
}

/// Scheduler tick result. `Continue` keeps the current task running; `Reschedule`
/// asks the dispatcher to select another runnable task.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TickAction { Continue, Reschedule }

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SchedulerClock { quantum: u32, remaining: u32 }

impl SchedulerClock {
    pub const fn new(quantum: u32) -> Self { Self { quantum, remaining: quantum } }
    pub const fn quantum(&self) -> u32 { self.quantum }
    pub const fn remaining(&self) -> u32 { self.remaining }
    pub fn tick(&mut self) -> TickAction {
        if self.quantum == 0 { return TickAction::Reschedule; }
        if self.remaining > 0 { self.remaining -= 1; }
        if self.remaining == 0 { self.remaining = self.quantum; TickAction::Reschedule } else { TickAction::Continue }
    }
    pub fn reset(&mut self) { self.remaining = self.quantum; }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn fifo_and_capacity_are_deterministic() {
        let mut q: RunQueue<2> = RunQueue::new();
        assert!(q.push(ProcessId(1))); assert!(q.push(ProcessId(2))); assert!(!q.push(ProcessId(3)));
        assert_eq!(q.pop(), Some(ProcessId(1))); assert!(q.push(ProcessId(3)));
        assert_eq!(q.pop(), Some(ProcessId(2))); assert_eq!(q.pop(), Some(ProcessId(3))); assert!(q.pop().is_none());
    }
    #[test] fn duplicate_is_rejected() { let mut q: RunQueue<2> = RunQueue::new(); assert!(q.push(ProcessId(1))); assert!(!q.push(ProcessId(1))); }
    #[test] fn scheduler_clock_requests_reschedule_at_quantum_boundary() {
        let mut clock = SchedulerClock::new(3);
        assert_eq!(clock.tick(), TickAction::Continue);
        assert_eq!(clock.tick(), TickAction::Continue);
        assert_eq!(clock.tick(), TickAction::Reschedule);
        assert_eq!(clock.remaining(), 3);
    }
    #[test] fn zero_quantum_fails_safe() { let mut clock = SchedulerClock::new(0); assert_eq!(clock.tick(), TickAction::Reschedule); }
}
