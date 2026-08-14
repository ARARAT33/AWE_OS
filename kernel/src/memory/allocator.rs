#![no_std]

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct BumpAllocator {
    start: AtomicUsize,
    end: AtomicUsize,
}

impl BumpAllocator {
    pub const fn new() -> Self { Self { start: AtomicUsize::new(0), end: AtomicUsize::new(0) } }

    pub unsafe fn init(&self, start: usize, size: usize) {
        self.start.store(start, Ordering::Release);
        self.end.store(start.saturating_add(size), Ordering::Release);
    }

    fn alloc_inner(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        loop {
            let current = self.start.load(Ordering::Acquire);
            let aligned = match current.checked_add(align - 1) { Some(v) => v & !(align - 1), None => return null_mut() };
            let next = match aligned.checked_add(size) { Some(v) => v, None => return null_mut() };
            if next > self.end.load(Ordering::Acquire) { return null_mut(); }
            if self.start.compare_exchange_weak(current, next, Ordering::AcqRel, Ordering::Acquire).is_ok() {
                return aligned as *mut u8;
            }
        }
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 { self.alloc_inner(layout) }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}
