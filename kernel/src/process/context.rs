#![no_std]

/// CPU context owned by a schedulable process.
/// Architecture-specific switching remains separate from validation.
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct CpuContext {
    pub rip: u64,
    pub rsp: u64,
    pub rflags: u64,
    pub cr3: u64,
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

impl CpuContext {
    pub const REQUIRED_RFLAGS: u64 = 1 << 1;
    pub const ALLOWED_RFLAGS: u64 = 0x0000_0000_003f_fffd;

    pub const fn kernel_entry(rip: u64, rsp: u64, cr3: u64) -> Self {
        Self { rip, rsp, rflags: Self::REQUIRED_RFLAGS, cr3, ..Self::default() }
    }

    pub const fn validate(&self) -> bool {
        self.rip != 0 && self.rsp != 0
            && self.rip >> 48 == 0 && self.rsp >> 48 == 0
            && self.cr3 & 0xfff == 0
            && self.rflags & Self::REQUIRED_RFLAGS != 0
            && self.rflags & !Self::ALLOWED_RFLAGS == 0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ProcessContext {
    pub process_id: super::ProcessId,
    pub cpu: CpuContext,
}

impl ProcessContext {
    pub const fn new(process_id: super::ProcessId, cpu: CpuContext) -> Self { Self { process_id, cpu } }
    pub const fn is_valid(&self) -> bool { self.cpu.validate() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valid_context_is_accepted() {
        assert!(CpuContext::kernel_entry(0x0040_0000, 0x0080_0000, 0x0010_0000).validate());
    }
    #[test]
    fn null_context_is_rejected() { assert!(!CpuContext::default().validate()); }
    #[test]
    fn unaligned_cr3_is_rejected() {
        assert!(!CpuContext::kernel_entry(0x0040_0000, 0x0080_0000, 0x0010_0001).validate());
    }
}
