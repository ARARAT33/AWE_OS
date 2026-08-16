#![no_std]

use super::{ProcessDescriptor, ProcessId, ProcessState};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SchedulerError { QueueFull, Empty, NotRunnable }

/// Deterministic bounded round-robin scheduler core.
pub struct Scheduler<const N: usize> {
    queue: [Option<ProcessId>; N],
    head: usize,
    len: usize,
    ticks: u64,
}

impl<const N: usize> Scheduler<N> {
    pub const fn new() -> Self { Self { queue: [None; N], head: 0, len: 0, ticks: 0 } }
    pub const fn is_empty(&self) -> bool { self.len == 0 }
    pub const fn len(&self) -> usize { self.len }
    pub const fn ticks(&self) -> u64 { self.ticks }

    pub fn enqueue(&mut self, process: &ProcessDescriptor) -> Result<(), SchedulerError> {
        if !matches!(process.state, ProcessState::Runnable | ProcessState::Running) {
            return Err(SchedulerError::NotRunnable);
        }
        if self.len == N { return Err(SchedulerError::QueueFull); }
        let index = (self.head + self.len) % N;
        self.queue[index] = Some(process.id);
        self.len += 1;
        Ok(())
    }

    pub fn dequeue(&mut self) -> Result<ProcessId, SchedulerError> {
        if self.len == 0 { return Err(SchedulerError::Empty); }
        let id = self.queue[self.head].take().ok_or(SchedulerError::Empty)?;
        self.head = (self.head + 1) % N;
        self.len -= 1;
        self.ticks = self.ticks.wrapping_add(1);
        Ok(id)
    }

    pub fn yield_process(&mut self, process: &ProcessDescriptor) -> Result<(), SchedulerError> {
        self.enqueue(process)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn process(id: u64, state: ProcessState) -> ProcessDescriptor {
        ProcessDescriptor { id: ProcessId(id), state, budget: super::super::ResourceBudget::unlimited() }
    }
    #[test]
    fn round_robin_order_is_deterministic() {
        let mut s = Scheduler::<4>::new();
        s.enqueue(&process(1, ProcessState::Runnable)).unwrap();
        s.enqueue(&process(2, ProcessState::Runnable)).unwrap();
        s.enqueue(&process(3, ProcessState::Runnable)).unwrap();
        assert_eq!(s.dequeue().unwrap(), ProcessId(1));
        assert_eq!(s.dequeue().unwrap(), ProcessId(2));
        assert_eq!(s.dequeue().unwrap(), ProcessId(3));
        assert_eq!(s.ticks(), 3);
    }
    #[test]
    fn blocked_process_never_enters_queue() {
        let mut s = Scheduler::<2>::new();
        assert_eq!(s.enqueue(&process(1, ProcessState::Blocked)), Err(SchedulerError::NotRunnable));
        assert!(s.is_empty());
    }
    #[test]
    fn queue_capacity_is_enforced() {
        let mut s = Scheduler::<1>::new();
        s.enqueue(&process(1, ProcessState::Runnable)).unwrap();
        assert_eq!(s.enqueue(&process(2, ProcessState::Runnable)), Err(SchedulerError::QueueFull));
    }
}
