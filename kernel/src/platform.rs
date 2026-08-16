#![no_std]

use awe_boot_protocol::Architecture;

pub mod pit;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PlatformFeatures {
    pub paging: bool,
    pub interrupts: bool,
    pub smp: bool,
    pub framebuffer: bool,
    pub acpi: bool,
    pub device_tree: bool,
}

impl PlatformFeatures {
    pub const fn for_architecture(architecture: Architecture) -> Self {
        match architecture {
            Architecture::X86_64 => Self {
                paging: true,
                interrupts: true,
                smp: true,
                framebuffer: true,
                acpi: true,
                device_tree: false,
            },
            Architecture::Aarch64 => Self {
                paging: true,
                interrupts: true,
                smp: true,
                framebuffer: true,
                acpi: true,
                device_tree: true,
            },
            Architecture::RiscV64 => Self {
                paging: true,
                interrupts: true,
                smp: true,
                framebuffer: true,
                acpi: false,
                device_tree: true,
            },
            Architecture::X86 | Architecture::Arm | Architecture::RiscV32 => Self {
                paging: false,
                interrupts: false,
                smp: false,
                framebuffer: false,
                acpi: false,
                device_tree: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use awe_boot_protocol::Architecture;

    #[test]
    fn x86_64_has_product_baseline_features() {
        let f = PlatformFeatures::for_architecture(Architecture::X86_64);
        assert!(f.paging && f.interrupts && f.smp && f.acpi);
    }

    #[test]
    fn pit_rejects_invalid_rate() {
        assert!(pit::Pit::new(0).is_none());
        assert!(pit::Pit::new(1000).is_some());
    }
}
