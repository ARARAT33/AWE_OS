//! AWEOS ARM64 / ARMv7 Bootloader Driver & MMU Handshake Subsystem.

#![no_std]

use awe_boot_protocol::{Architecture, BootInfo, MemoryRegion};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmExceptionLevel {
    EL0,
    EL1,
    EL2,
    EL3,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FdtHeader {
    pub magic: u32,
    pub totalsize: u32,
    pub off_dt_struct: u32,
    pub off_dt_strings: u32,
    pub off_mem_rsvmap: u32,
    pub version: u32,
    pub last_comp_version: u32,
    pub boot_cpuid_phys: u32,
    pub size_dt_strings: u32,
    pub size_dt_struct: u32,
}

pub struct ArmBootManager {
    pub current_el: ArmExceptionLevel,
    pub fdt_pointer: usize,
    pub memory_regions: [MemoryRegion; 32],
    pub memory_region_count: usize,
}

impl ArmBootManager {
    pub fn new(fdt_ptr: usize) -> Self {
        let mut mgr = Self {
            current_el: ArmExceptionLevel::EL2,
            fdt_pointer: fdt_ptr,
            memory_regions: [MemoryRegion {
                base: 0,
                length: 0,
                kind: 1,
                reserved: 0,
            }; 32],
            memory_region_count: 0,
        };
        mgr.parse_fdt_header();
        mgr
    }

    pub fn parse_fdt_header(&mut self) -> bool {
        if self.fdt_pointer == 0 {
            return false;
        }
        let header_ptr = self.fdt_pointer as *const FdtHeader;
        let header = unsafe { &*header_ptr };
        let magic = u32::from_be(header.magic);
        if magic != 0xD00DFEED {
            return false;
        }

        self.memory_regions[0] = MemoryRegion {
            base: 0x4000_0000,
            length: 1024 * 1024 * 1024, // 1 GiB
            kind: 1,
            reserved: 0,
        };
        self.memory_region_count = 1;
        true
    }

    pub fn prepare_handoff(&self) -> BootInfo {
        let mut info = BootInfo::empty(Architecture::Aarch64);
        info.kernel_base = 0x4008_0000;
        info.memory_regions = self.memory_regions.as_ptr();
        info.memory_region_count = self.memory_region_count as u32;
        info.framebuffer_address = 0x3C00_0000;
        info.framebuffer_size = 1024 * 768 * 4;
        info.framebuffer_width = 1024;
        info.framebuffer_height = 768;
        info.framebuffer_pitch = 1024 * 4;
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arm_boot_manager_initialization() {
        let mgr = ArmBootManager::new(0);
        assert_eq!(mgr.current_el, ArmExceptionLevel::EL2);
        let handoff = mgr.prepare_handoff();
        assert!(matches!(handoff.architecture, Architecture::Aarch64));
    }
}
