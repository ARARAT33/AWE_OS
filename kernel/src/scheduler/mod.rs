#![no_std]

use super::process::ProcessId;

pub const MAX_RUNNABLE: usize = 256;

/// Bounded FIFO scheduler queue. It deliberately performs no allocation and
/// refuses both overflow and duplicate runnable entries.
pub struct Scheduler {
    queue: [Option<ProcessId>; MAX_RUNNABLE],
    head: usize,
    tail: usize,
    len: usize,
}

impl Scheduler {
    pub const fn new() -> Self { Self { queue: [None; MAX_RUNNABLE], head: 0, tail: 0, len: 0 } }

    pub fn enqueue(&mut self, process: ProcessId) -> bool {
        if self.len == MAX_RUNNABLE || self.contains(process) { return false; }
        self.queue[self.tail] = Some(process);
        self.tail = (self.tail + 1) % MAX_RUNNABLE;
        self.len += 1;
        true
    }

    pub fn dequeue(&mut self) -> Option<ProcessId> {
        if self.len == 0 { return None; }
        let process = self.queue[self.head].take();
        self.head = (self.head + 1) % MAX_RUNNABLE;
        self.len -= 1;
        process
    }

    pub const fn len(&self) -> usize { self.len }
    pub const fn is_empty(&self) -> bool { self.len == 0 }
    pub const fn is_full(&self) -> bool { self.len == MAX_RUNNABLE }

    fn contains(&self, process: ProcessId) -> bool {
        let mut i = 0;
        while i < self.len {
            let index = (self.head + i) % MAX_RUNNABLE;
            if let Some(candidate) = self.queue[index] {
                if candidate == process { return true; }
            }
            i += 1;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifo_order_and_duplicate_protection() {
        let mut scheduler = Scheduler::new();
        let a = ProcessId(10);
        let b = ProcessId(11);
        assert!(scheduler.enqueue(a));
        assert!(scheduler.enqueue(b));
        assert!(!scheduler.enqueue(a));
        assert_eq!(scheduler.dequeue(), Some(a));
        assert_eq!(scheduler.dequeue(), Some(b));
        assert!(scheduler.is_empty());
    }

    #[test]
    fn queue_never_exceeds_bound() {
        let mut scheduler = Scheduler::new();
        for id in 0..MAX_RUNNABLE as u64 {
            assert!(scheduler.enqueue(ProcessId(id + 1)));
        }
        assert!(scheduler.is_full());
        assert!(!scheduler.enqueue(ProcessId(MAX_RUNNABLE as u64 + 1)));
    }
}
