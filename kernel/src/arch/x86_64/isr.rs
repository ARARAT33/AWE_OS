#![no_std]

/// CPU state captured at an interrupt boundary. The layout is intentionally
/// explicit so the eventual assembly ISR stubs have one stable Rust ABI.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct InterruptFrame {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rbp: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub vector: u64,
    pub error_code: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

impl InterruptFrame {
    pub const fn empty() -> Self {
        Self {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            r10: 0,
            r9: 0,
            r8: 0,
            rdi: 0,
            rsi: 0,
            rbp: 0,
            rdx: 0,
            rcx: 0,
            rbx: 0,
            rax: 0,
            vector: 0,
            error_code: 0,
            rip: 0,
            cs: 0,
            rflags: 0,
            rsp: 0,
            ss: 0,
        }
    }

    pub const fn instruction_pointer_valid(&self) -> bool {
        let upper = self.rip >> 48;
        (upper == 0 || upper == 0xffff) && self.rip != 0
    }

    pub const fn stack_pointer_valid(&self) -> bool {
        let upper = self.rsp >> 48;
        (upper == 0 || upper == 0xffff) && (self.rsp & 0x7) == 0
    }

    pub const fn rflags_valid(&self) -> bool {
        // Bit 1 is architecturally fixed to one; VM is not valid for a normal
        // 64-bit kernel return frame.
        self.rflags & 0x2 != 0 && self.rflags & (1 << 17) == 0
    }

    pub const fn validate(&self) -> bool {
        self.instruction_pointer_valid() && self.stack_pointer_valid() && self.rflags_valid()
    }
}

pub const fn is_exception_with_error_code(vector: u8) -> bool {
    matches!(vector, 8 | 10 | 11 | 12 | 13 | 14 | 17 | 21 | 29 | 30)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_validation_accepts_canonical_state() {
        let mut f = InterruptFrame::empty();
        f.rip = 0x0000_0000_0040_0000;
        f.rsp = 0x0000_0000_0080_0000;
        f.rflags = 0x202;
        assert!(f.validate());
    }

    #[test]
    fn frame_validation_rejects_bad_state() {
        let mut f = InterruptFrame::empty();
        f.rip = 0x0001_0000_0000_0000;
        f.rsp = 0x1001;
        f.rflags = 0;
        assert!(!f.validate());
    }

    #[test]
    fn error_code_vectors_are_explicit() {
        assert!(is_exception_with_error_code(14));
        assert!(is_exception_with_error_code(13));
        assert!(!is_exception_with_error_code(32));
    }
}
