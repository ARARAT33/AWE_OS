#![no_std]

use core::sync::atomic::{AtomicBool, Ordering};

pub struct SpinLock {
    locked: AtomicBool,
}

impl SpinLock {
    pub const fn new() -> Self { Self { locked: AtomicBool::new(false) } }

    pub fn try_lock(&self) -> bool {
        self.locked.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok()
    }

    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

// A minimal primitive only; preemption-aware guards belong in the scheduler layer.
