#![no_std]

use awe_boot_protocol::{validate, BootInfo};

use crate::boot_phase::{BootPhase, BootProgress};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KernelBootStatus {
    Ready = 0,
    InvalidBootInfo = 1,
    UnsupportedArchitecture = 2,
    NoCpu = 3,
}

pub struct KernelContext {
    progress: BootProgress,
}

impl KernelContext {
    pub const fn new() -> Self {
        Self {
            progress: BootProgress::new(),
        }
    }

    pub const fn phase(&self) -> BootPhase {
        self.progress.phase()
    }

    pub fn advance(&mut self) -> bool {
        self.progress.advance()
    }
}

/// Stable entry contract between AWE Loader and CellKernel.
///
/// The loader owns the lifetime of `BootInfo`; the kernel must validate the
/// structure before reading any pointer supplied by firmware.
pub fn kernel_entry(info: &BootInfo) -> KernelBootStatus {
    if !validate(info) {
        return KernelBootStatus::InvalidBootInfo;
    }

    if !info.architecture.is_supported() {
        return KernelBootStatus::UnsupportedArchitecture;
    }

    if info.cpu_count == 0 {
        return KernelBootStatus::NoCpu;
    }

    KernelBootStatus::Ready
}

#[cfg(test)]
mod tests {
    use super::*;
    use awe_boot_protocol::{Architecture, BootInfo};

    #[test]
    fn accepts_valid_x86_64_handoff() {
        let info = BootInfo::empty(Architecture::X86_64);
        assert_eq!(kernel_entry(&info), KernelBootStatus::Ready);
    }

    #[test]
    fn rejects_invalid_handoff() {
        let mut info = BootInfo::empty(Architecture::X86_64);
        info.magic = 0;
        assert_eq!(kernel_entry(&info), KernelBootStatus::InvalidBootInfo);
    }
}
