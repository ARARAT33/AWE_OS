#![no_std]

use awe_boot_protocol::{validate, BootInfo};

use crate::boot_phase::{BootPhase, BootProgress};
use crate::memory::PhysicalFrameAllocator;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KernelBootStatus {
    Ready = 0,
    InvalidBootInfo = 1,
    UnsupportedArchitecture = 2,
    NoCpu = 3,
    NoUsableMemory = 4,
}

pub struct KernelContext {
    progress: BootProgress,
}

impl KernelContext {
    pub const fn new() -> Self { Self { progress: BootProgress::new() } }
    pub const fn phase(&self) -> BootPhase { self.progress.phase() }
    pub fn advance(&mut self) -> bool { self.progress.advance() }
}

/// Stable entry contract between AWE Loader and CellKernel.
///
/// The loader owns the lifetime of `BootInfo`; the kernel validates the
/// structure and immediately exercises the physical-frame allocator against
/// the loader-provided memory map before declaring itself ready.
pub fn kernel_entry(info: &BootInfo) -> KernelBootStatus {
    if !validate(info) { return KernelBootStatus::InvalidBootInfo; }
    if !info.architecture.is_supported() { return KernelBootStatus::UnsupportedArchitecture; }
    if info.cpu_count == 0 { return KernelBootStatus::NoCpu; }
    if info.memory_region_count == 0 || info.memory_regions.is_null() { return KernelBootStatus::NoUsableMemory; }

    let mut frames = unsafe { PhysicalFrameAllocator::from_boot_info(info) };
    if frames.allocate().is_none() { return KernelBootStatus::NoUsableMemory; }

    KernelBootStatus::Ready
}

#[cfg(test)]
mod tests {
    use super::*;
    use awe_boot_protocol::{Architecture, MemoryRegion, BootInfo};

    #[test]
    fn accepts_valid_x86_64_handoff_with_memory() {
        let regions = [MemoryRegion { base: 0x1000, length: 0x10000, kind: 1, reserved: 0 }];
        let info = BootInfo {
            magic: awe_boot_protocol::AWE_BOOT_MAGIC,
            version: awe_boot_protocol::AWE_BOOT_VERSION,
            size: core::mem::size_of::<BootInfo>() as u32,
            architecture: Architecture::X86_64,
            cpu_count: 1,
            memory_regions: regions.as_ptr(),
            memory_region_count: 1,
            framebuffer_address: 0,
            framebuffer_size: 0,
            framebuffer_width: 0,
            framebuffer_height: 0,
            framebuffer_pitch: 0,
            acpi_rsdp: 0,
            device_tree: 0,
            kernel_base: 0,
            kernel_size: 0,
        };
        assert_eq!(kernel_entry(&info), KernelBootStatus::Ready);
    }

    #[test]
    fn rejects_invalid_handoff() {
        let mut info = BootInfo::empty(Architecture::X86_64);
        info.magic = 0;
        assert_eq!(kernel_entry(&info), KernelBootStatus::InvalidBootInfo);
    }

    #[test]
    fn rejects_missing_memory_map() {
        let info = BootInfo::empty(Architecture::X86_64);
        assert_eq!(kernel_entry(&info), KernelBootStatus::NoUsableMemory);
    }
}
