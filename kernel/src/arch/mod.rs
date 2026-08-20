//! Architecture abstractions and hardware boundary definitions for AWEOS.

#[cfg(target_arch = "aarch64")]
pub mod aarch64;
#[cfg(target_arch = "arm")]
pub mod arm;
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
pub mod riscv;
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

/// Target CPU architecture supported by AWEOS kernel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetArch {
    X86_64,
    X86_32,
    AArch64,
    Arm32,
    RiscV64,
    RiscV32,
}

impl TargetArch {
    /// Native architecture of current build target.
    pub const fn current() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            Self::X86_64
        }
        #[cfg(target_arch = "x86")]
        {
            Self::X86_32
        }
        #[cfg(target_arch = "aarch64")]
        {
            Self::AArch64
        }
        #[cfg(target_arch = "arm")]
        {
            Self::Arm32
        }
        #[cfg(target_arch = "riscv64")]
        {
            Self::RiscV64
        }
        #[cfg(target_arch = "riscv32")]
        {
            Self::RiscV32
        }
        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "x86",
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv64",
            target_arch = "riscv32"
        )))]
        {
            Self::X86_64
        }
    }

    pub const fn bit_width(self) -> u8 {
        match self {
            Self::X86_64 | Self::AArch64 | Self::RiscV64 => 64,
            Self::X86_32 | Self::Arm32 | Self::RiscV32 => 32,
        }
    }

    pub const fn page_size(self) -> usize {
        4096
    }
}

/// Generic CPU Execution Context state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ArchCpuState {
    pub arch: TargetArch,
    pub instruction_pointer: u64,
    pub stack_pointer: u64,
    pub flags_register: u64,
    pub cr3_or_page_table: u64,
}

impl ArchCpuState {
    pub const fn new(arch: TargetArch, ip: u64, sp: u64, page_table: u64) -> Self {
        Self {
            arch,
            instruction_pointer: ip,
            stack_pointer: sp,
            flags_register: 0x202, // IF bit set for interrupts
            cr3_or_page_table: page_table,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.instruction_pointer != 0 && self.stack_pointer != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_arch_bit_width() {
        assert_eq!(TargetArch::X86_64.bit_width(), 64);
        assert_eq!(TargetArch::X86_32.bit_width(), 32);
        assert_eq!(TargetArch::AArch64.bit_width(), 64);
        assert_eq!(TargetArch::Arm32.bit_width(), 32);
        assert_eq!(TargetArch::RiscV64.bit_width(), 64);
        assert_eq!(TargetArch::RiscV32.bit_width(), 32);
    }

    #[test]
    fn test_cpu_state_validation() {
        let state = ArchCpuState::new(TargetArch::X86_64, 0x1000, 0x8000, 0x2000);
        assert!(state.is_valid());

        let invalid = ArchCpuState::new(TargetArch::X86_64, 0, 0x8000, 0x2000);
        assert!(!invalid.is_valid());
    }
}
