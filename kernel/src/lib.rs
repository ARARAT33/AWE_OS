#![no_std]
#![allow(dead_code)]
#![allow(unused_attributes)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::module_inception)]

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

struct EarlyAllocator;
const EARLY_HEAP_SIZE: usize = 2 * 1024 * 1024;
static mut EARLY_HEAP: [u8; EARLY_HEAP_SIZE] = [0; EARLY_HEAP_SIZE];
static EARLY_NEXT: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for EarlyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mask = layout.align().saturating_sub(1);
        loop {
            let current = EARLY_NEXT.load(Ordering::Relaxed);
            let aligned = match current.checked_add(mask) { Some(v) => v & !mask, None => return core::ptr::null_mut() };
            let end = match aligned.checked_add(layout.size()) { Some(v) => v, None => return core::ptr::null_mut() };
            if end > EARLY_HEAP_SIZE { return core::ptr::null_mut(); }
            if EARLY_NEXT.compare_exchange_weak(current, end, Ordering::AcqRel, Ordering::Relaxed).is_ok() {
                return unsafe { core::ptr::addr_of_mut!(EARLY_HEAP) as *mut u8 }.wrapping_add(aligned);
            }
        }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static GLOBAL_ALLOCATOR: EarlyAllocator = EarlyAllocator;

#[alloc_error_handler]
fn allocation_error(_: Layout) -> ! { loop { core::hint::spin_loop(); } }

// CellKernel is intentionally hardware-driver free.
// All hardware discovery, driver lifecycle, compatibility adapters and
// VirtIO/Linux/Windows/Android driver execution live in services/driverd.
// The kernel owns only minimal process/IPC/capability primitives needed to
// isolate and communicate with those services.
pub mod ac_boot_gate;
pub mod ac_completion;
pub mod ac_runtime;
pub mod ai;
pub mod aosin;
pub mod arch;
pub mod boot_guard;
pub mod boot_phase;
pub mod boot_state;
pub mod compat;
pub mod continuum;
pub mod device;
pub mod drivers;
pub mod engineering_contracts;
pub mod entry;
pub mod execution_core;
pub mod formats;
pub mod interrupts;
pub mod ipc;
pub mod logging;
pub mod memory;
pub mod net;
pub mod platform;
pub mod process;
pub mod runtime;
pub mod scheduler;
pub mod security;
pub mod service;
pub mod service_registry;
pub mod storage;
pub mod syscall;
pub mod system_contract;
pub mod time;
