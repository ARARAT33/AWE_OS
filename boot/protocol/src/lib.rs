#![no_std]

pub const AWE_BOOT_MAGIC: u64 = 0x4157_4542_4F4F_5431;
pub const AWE_BOOT_VERSION: u32 = 1;

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86 = 1,
    X86_64 = 2,
    Arm = 3,
    Aarch64 = 4,
    RiscV32 = 5,
    RiscV64 = 6,
}

impl Architecture {
    pub const fn is_64_bit(self) -> bool {
        matches!(self, Self::X86_64 | Self::Aarch64 | Self::RiscV64)
    }

    pub const fn is_supported(self) -> bool {
        matches!(self, Self::X86_64 | Self::Aarch64 | Self::RiscV64)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MemoryRegion {
    pub base: u64,
    pub length: u64,
    pub kind: u32,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BootInfo {
    pub magic: u64,
    pub version: u32,
    pub size: u32,
    pub architecture: Architecture,
    pub cpu_count: u32,
    pub memory_regions: *const MemoryRegion,
    pub memory_region_count: u32,
    pub framebuffer_address: u64,
    pub framebuffer_size: u64,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub framebuffer_pitch: u32,
    pub acpi_rsdp: u64,
    pub device_tree: u64,
    pub kernel_base: u64,
    pub kernel_size: u64,
}

impl BootInfo {
    pub const fn empty(architecture: Architecture) -> Self {
        Self {
            magic: AWE_BOOT_MAGIC,
            version: AWE_BOOT_VERSION,
            size: core::mem::size_of::<Self>() as u32,
            architecture,
            cpu_count: 1,
            memory_regions: core::ptr::null(),
            memory_region_count: 0,
            framebuffer_address: 0,
            framebuffer_size: 0,
            framebuffer_width: 0,
            framebuffer_height: 0,
            framebuffer_pitch: 0,
            acpi_rsdp: 0,
            device_tree: 0,
            kernel_base: 0,
            kernel_size: 0,
        }
    }
}

pub const fn validate(info: &BootInfo) -> bool {
    info.magic == AWE_BOOT_MAGIC
        && info.version == AWE_BOOT_VERSION
        && info.size >= core::mem::size_of::<BootInfo>() as u32
        && info.cpu_count != 0
        && info.architecture.is_supported()
        && (info.memory_region_count == 0 || !info.memory_regions.is_null())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_boot_info_passes() {
        let info = BootInfo::empty(Architecture::X86_64);
        assert!(validate(&info));
    }

    #[test]
    fn invalid_magic_fails() {
        let mut info = BootInfo::empty(Architecture::X86_64);
        info.magic = 0;
        assert!(!validate(&info));
    }

    #[test]
    fn unsupported_architecture_fails() {
        let info = BootInfo::empty(Architecture::X86);
        assert!(!validate(&info));
    }
}
