#![no_std]

use super::process::ProcessId;

pub const MAX_RUNNABLE: usize = 256;

pub struct Scheduler {
    queue: [Option<ProcessId>; MAX_RUNNABLE],
    head: usize,
    tail: usize,
    len: usize,
}

impl Scheduler {
    pub const fn new() -> Self { Self { queue: [None; MAX_RUNNABLE], head: 0, tail: 0, len: 0 } }

    pub fn enqueue(&mut self, process: ProcessId) -> bool {
        if self.len == MAX_RUNNABLE { return false; }
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
}
