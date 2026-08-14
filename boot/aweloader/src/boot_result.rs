#![no_std]

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BootError {
    BadMagic = 1,
    UnsupportedProtocol = 2,
    WrongArchitecture = 3,
    ImageOutOfBounds = 4,
    KernelOutOfBounds = 5,
    InvalidEntry = 6,
    SignatureRequired = 7,
    SignatureInvalid = 8,
    Rollback = 9,
    ForeignImage = 10,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BootFailure {
    pub code: BootError,
    pub detail: u64,
}

pub enum BootDecision {
    Load,
    Halt(BootFailure),
}

impl BootFailure {
    pub const fn new(code: BootError, detail: u64) -> Self { Self { code, detail } }
}
