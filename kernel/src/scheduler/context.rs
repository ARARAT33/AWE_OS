#![no_std]

use crate::process::ProcessId;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Context {
    pub process: ProcessId,
    pub instruction_pointer: u64,
    pub stack_pointer: u64,
    pub flags: u64,
    pub address_space: u64,
}

impl Context {
    pub const fn new(process: ProcessId, instruction_pointer: u64, stack_pointer: u64, flags: u64, address_space: u64) -> Self {
        Self { process, instruction_pointer, stack_pointer, flags, address_space }
    }

    pub const fn is_canonical_address(value: u64) -> bool {
        let upper = value >> 48;
        upper == 0 || upper == 0xffff
    }

    pub const fn validate(self) -> bool {
        Self::is_canonical_address(self.instruction_pointer)
            && Self::is_canonical_address(self.stack_pointer)
            && (self.stack_pointer & 0x7) == 0
            && self.address_space != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_context_is_accepted() {
        let c = Context::new(ProcessId(7), 0x0000_0000_0040_0000, 0x0000_0000_0080_0000, 0x202, 0x1000);
        assert!(c.validate());
    }

    #[test]
    fn invalid_context_is_rejected() {
        let c = Context::new(ProcessId(7), 0x0001_0000_0000_0000, 0x1001, 0, 0);
        assert!(!c.validate());
    }
}
