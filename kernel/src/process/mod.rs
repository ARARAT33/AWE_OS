#![no_std]

use core::sync::atomic::{AtomicU64, Ordering};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ProcessId(pub u64);

pub struct ProcessTable {
    next_id: AtomicU64,
}

impl ProcessTable {
    pub const fn new() -> Self { Self { next_id: AtomicU64::new(1) } }

    pub fn allocate_id(&self) -> ProcessId {
        ProcessId(self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}
