#![no_std]

use awe_boot_protocol::BootInfo;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Frame {
    pub start: u64,
}

pub const PAGE_SIZE: u64 = 4096;

pub const fn frame_at(address: u64) -> Frame {
    Frame { start: address & !(PAGE_SIZE - 1) }
}

pub const fn frame_end(frame: Frame) -> u64 {
    frame.start + PAGE_SIZE
}

/// A deterministic physical-frame cursor backed directly by the loader's
/// memory map. Only Multiboot2/AWE regions marked usable (`kind == 1`) are
/// exposed. The first page is never returned.
pub struct PhysicalFrameAllocator {
    regions: *const awe_boot_protocol::MemoryRegion,
    region_count: u32,
    region_index: u32,
    cursor: u64,
    end: u64,
}

impl PhysicalFrameAllocator {
    pub const fn empty() -> Self {
        Self {
            regions: core::ptr::null(),
            region_count: 0,
            region_index: 0,
            cursor: 0,
            end: 0,
        }
    }

    /// # Safety
    /// The memory-region array referenced by `info` must remain valid for the
    /// lifetime of this allocator and must not be concurrently mutated.
    pub unsafe fn from_boot_info(info: &BootInfo) -> Self {
        let mut allocator = Self {
            regions: info.memory_regions,
            region_count: info.memory_region_count,
            region_index: 0,
            cursor: 0,
            end: 0,
        };
        allocator.select_next_region();
        allocator
    }

    fn select_next_region(&mut self) {
        while self.region_index < self.region_count {
            let region = unsafe { core::ptr::read(self.regions.add(self.region_index as usize)) };
            self.region_index += 1;
            if region.kind != 1 || region.length < PAGE_SIZE { continue; }

            let start = region.base.max(PAGE_SIZE).saturating_add(PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
            let end = region.base.saturating_add(region.length) & !(PAGE_SIZE - 1);
            if start < end {
                self.cursor = start;
                self.end = end;
                return;
            }
        }
        self.cursor = 0;
        self.end = 0;
    }

    /// Returns the next available physical 4 KiB frame, or `None` when the
    /// loader-provided usable memory map is exhausted.
    pub fn allocate(&mut self) -> Option<Frame> {
        if self.cursor == 0 || self.cursor >= self.end {
            self.select_next_region();
        }
        if self.cursor == 0 { return None; }
        let frame = Frame { start: self.cursor };
        self.cursor = self.cursor.saturating_add(PAGE_SIZE);
        Some(frame)
    }

    pub fn remaining_in_current_region(&self) -> u64 {
        self.end.saturating_sub(self.cursor) / PAGE_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use awe_boot_protocol::{Architecture, MemoryRegion};

    #[test]
    fn frame_alignment_is_stable() {
        assert_eq!(frame_at(0x1234).start, 0x1000);
        assert_eq!(frame_end(frame_at(0x1234)), 0x2000);
    }

    #[test]
    fn allocator_skips_reserved_memory_and_page_zero() {
        let regions = [
            MemoryRegion { base: 0, length: 0x2000, kind: 1, reserved: 0 },
            MemoryRegion { base: 0x2000, length: 0x2000, kind: 2, reserved: 0 },
            MemoryRegion { base: 0x4000, length: 0x3000, kind: 1, reserved: 0 },
        ];
        let info = BootInfo {
            magic: awe_boot_protocol::AWE_BOOT_MAGIC,
            version: awe_boot_protocol::AWE_BOOT_VERSION,
            size: core::mem::size_of::<BootInfo>() as u32,
            architecture: Architecture::X86_64,
            cpu_count: 1,
            memory_regions: regions.as_ptr(),
            memory_region_count: regions.len() as u32,
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
        let mut allocator = unsafe { PhysicalFrameAllocator::from_boot_info(&info) };
        assert_eq!(allocator.allocate().unwrap().start, 0x1000);
        assert_eq!(allocator.allocate().unwrap().start, 0x4000);
        assert_eq!(allocator.allocate().unwrap().start, 0x5000);
        assert_eq!(allocator.allocate().unwrap().start, 0x6000);
        assert!(allocator.allocate().is_none());
    }
}
