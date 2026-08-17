#![no_std]
use crate::process::ProcessId;
pub const PRIORITY_LEVELS: usize = 8;
pub struct PriorityScheduler<const N: usize> {
    queues: [[Option<ProcessId>; N]; PRIORITY_LEVELS],
    heads: [usize; PRIORITY_LEVELS],
    tails: [usize; PRIORITY_LEVELS],
    lengths: [usize; PRIORITY_LEVELS],
}
impl<const N: usize> PriorityScheduler<N> {
    pub const fn new() -> Self {
        Self {
            queues: [[None; N]; PRIORITY_LEVELS],
            heads: [0; PRIORITY_LEVELS],
            tails: [0; PRIORITY_LEVELS],
            lengths: [0; PRIORITY_LEVELS],
        }
    }
    pub fn enqueue(&mut self, process: ProcessId, priority: u8) -> bool {
        let p = priority as usize;
        if p >= PRIORITY_LEVELS || self.lengths[p] == N || self.contains(process) {
            return false;
        }
        self.queues[p][self.tails[p]] = Some(process);
        self.tails[p] = (self.tails[p] + 1) % N;
        self.lengths[p] += 1;
        true
    }
    pub fn dequeue(&mut self) -> Option<ProcessId> {
        let mut p = PRIORITY_LEVELS;
        while p > 0 {
            p -= 1;
            if self.lengths[p] != 0 {
                let process = self.queues[p][self.heads[p]].take();
                self.heads[p] = (self.heads[p] + 1) % N;
                self.lengths[p] -= 1;
                return process;
            }
        }
        None
    }
    fn contains(&self, process: ProcessId) -> bool {
        let mut p = 0;
        while p < PRIORITY_LEVELS {
            let mut i = 0;
            while i < self.lengths[p] {
                if self.queues[p][(self.heads[p] + i) % N] == Some(process) {
                    return true;
                }
                i += 1
            }
            p += 1
        }
        false
    }
}
