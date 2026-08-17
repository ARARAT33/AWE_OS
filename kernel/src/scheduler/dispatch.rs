#![no_std]

use super::RunQueue;
use crate::process::ProcessId;

/// Deterministic scheduler dispatcher for the early kernel. It keeps the
/// current process separate from the runnable queue and never allocates.
pub struct Dispatcher<const N: usize> {
    queue: RunQueue<N>,
    current: Option<ProcessId>,
}

impl<const N: usize> Default for Dispatcher<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Dispatcher<N> {
    pub const fn new() -> Self {
        Self {
            queue: RunQueue::new(),
            current: None,
        }
    }
    pub const fn current(&self) -> Option<ProcessId> {
        self.current
    }
    pub const fn runnable(&self) -> usize {
        self.queue.len()
    }

    pub fn enqueue(&mut self, id: ProcessId) -> bool {
        self.queue.push(id)
    }

    /// Performs one deterministic scheduling decision. The previously running
    /// process is requeued only when the caller explicitly requests it; this
    /// makes context-switch policy visible rather than implicit.
    pub fn schedule(&mut self, requeue_current: bool) -> Option<ProcessId> {
        if requeue_current && let Some(id) = self.current {
            let _ = self.queue.push(id);
        }
        self.current = self.queue.pop();
        self.current
    }

    pub fn yield_current(&mut self) -> Option<ProcessId> {
        let id = self.current.take()?;
        let _ = self.queue.push(id);
        self.current = self.queue.pop();
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduler_rotates_without_allocations() {
        let mut d: Dispatcher<4> = Dispatcher::new();
        assert!(d.enqueue(ProcessId(1)));
        assert!(d.enqueue(ProcessId(2)));
        assert_eq!(d.schedule(false), Some(ProcessId(1)));
        assert_eq!(d.schedule(true), Some(ProcessId(2)));
        assert_eq!(d.yield_current(), Some(ProcessId(1)));
    }

    #[test]
    fn empty_scheduler_has_no_current_process() {
        let mut d: Dispatcher<2> = Dispatcher::new();
        assert_eq!(d.schedule(false), None);
        assert_eq!(d.current(), None);
    }
}
