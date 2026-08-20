//! AWEOS RISC-V 64 (RV64) Bootloader Driver & Supervisor Binary Interface (SBI) Subsystem.

#![no_std]

use awe_boot_protocol::{Architecture, BootInfo, MemoryRegion};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SbiExtension {
    Base = 0x10,
    Timer = 0x54494D45,
    Ipi = 0x735049,
    Rfnc = 0x52464E43,
    Hsm = 0x48534D,
    Srst = 0x53525354,
}

#[derive(Debug, Clone, Copy)]
pub struct SbiRet {
    pub error: isize,
    pub value: usize,
}

pub struct RiscvBootManager {
    pub boot_hart_id: usize,
    pub fdt_pointer: usize,
    pub memory_regions: [MemoryRegion; 32],
    pub memory_region_count: usize,
}

impl RiscvBootManager {
    pub fn new(hart_id: usize, fdt_ptr: usize) -> Self {
        let mut mgr = Self {
            boot_hart_id: hart_id,
            fdt_pointer: fdt_ptr,
            memory_regions: [MemoryRegion {
                base: 0,
                length: 0,
                kind: 1,
                reserved: 0,
            }; 32],
            memory_region_count: 0,
        };
        mgr.init_memory_map();
        mgr
    }

    pub fn init_memory_map(&mut self) {
        self.memory_regions[0] = MemoryRegion {
            base: 0x8000_0000,
            length: 2 * 1024 * 1024 * 1024, // 2 GiB
            kind: 1,
            reserved: 0,
        };
        self.memory_region_count = 1;
    }

    pub fn prepare_handoff(&self) -> BootInfo {
        let mut info = BootInfo::empty(Architecture::RiscV64);
        info.kernel_base = 0x8020_0000;
        info.memory_regions = self.memory_regions.as_ptr();
        info.memory_region_count = self.memory_region_count as u32;
        info.framebuffer_address = 0x1000_0000;
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
    fn test_riscv_boot_manager() {
        let mgr = RiscvBootManager::new(0, 0x8700_0000);
        assert_eq!(mgr.boot_hart_id, 0);
        let handoff = mgr.prepare_handoff();
        assert!(matches!(handoff.architecture, Architecture::RiscV64));
    }
}
