#![no_std]

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Lock-free bump allocator used for the earliest kernel heap phase.
///
/// It is intentionally monotonic: individual frees are ignored until a
/// later allocator implementation can safely reclaim pages. The allocator
/// is bounded, alignment-aware and safe under concurrent allocation calls.
pub struct BumpAllocator {
    start: AtomicUsize,
    end: AtomicUsize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            start: AtomicUsize::new(0),
            end: AtomicUsize::new(0),
        }
    }

    /// # Safety
    /// The `[start, start + size)` region must be exclusively reserved for
    /// this allocator and remain valid for its entire lifetime.
    pub unsafe fn init(&self, start: usize, size: usize) {
        self.start.store(start, Ordering::Release);
        self.end.store(start.saturating_add(size), Ordering::Release);
    }

    pub fn bounds(&self) -> (usize, usize) {
        (
            self.start.load(Ordering::Acquire),
            self.end.load(Ordering::Acquire),
        )
    }

    pub fn capacity(&self) -> usize {
        let (start, end) = self.bounds();
        end.saturating_sub(start)
    }

    fn alloc_inner(&self, layout: Layout) -> *mut u8 {
        if layout.size() == 0 || !layout.align().is_power_of_two() {
            return null_mut();
        }

        let align = layout.align();
        let size = layout.size();
        loop {
            let current = self.start.load(Ordering::Acquire);
            let aligned = match current.checked_add(align - 1) {
                Some(v) => v & !(align - 1),
                None => return null_mut(),
            };
            let next = match aligned.checked_add(size) {
                Some(v) => v,
                None => return null_mut(),
            };
            if next > self.end.load(Ordering::Acquire) {
                return null_mut();
            }
            if self
                .start
                .compare_exchange_weak(current, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return aligned as *mut u8;
            }
        }
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.alloc_inner(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocator_respects_alignment_and_bounds() {
        let allocator = BumpAllocator::new();
        let mut storage = [0u8; 128];
        unsafe { allocator.init(storage.as_mut_ptr() as usize, storage.len()); }

        let a = allocator.alloc_inner(Layout::from_size_align(1, 1).unwrap());
        let b = allocator.alloc_inner(Layout::from_size_align(16, 16).unwrap());
        assert!(!a.is_null());
        assert!(!b.is_null());
        assert_eq!((b as usize) % 16, 0);
        assert_eq!(allocator.capacity(), 128);
    }

    #[test]
    fn zero_sized_allocations_are_rejected() {
        let allocator = BumpAllocator::new();
        let layout = Layout::from_size_align(0, 8).unwrap();
        assert!(allocator.alloc_inner(layout).is_null());
    }
}
