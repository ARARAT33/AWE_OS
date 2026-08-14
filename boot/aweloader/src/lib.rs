#![no_std]

use awe_boot_protocol::{validate, Architecture, BootInfo};

pub struct LoaderState {
    pub info: BootInfo,
}

impl LoaderState {
    pub const fn new(architecture: Architecture) -> Self {
        Self { info: BootInfo::empty(architecture) }
    }

    pub fn ready(&self) -> bool {
        validate(&self.info)
    }
}

/// Architecture-neutral handoff. Platform entry code must populate BootInfo
/// before transferring control to the kernel.
///
/// # Safety
/// `kernel_entry` must point to a valid AWEOS kernel entry and `info` must be
/// valid for the lifetime required by the kernel.
pub unsafe fn handoff(kernel_entry: extern "C" fn(*const BootInfo) -> !, info: &BootInfo) -> ! {
    kernel_entry(info as *const BootInfo)
}
