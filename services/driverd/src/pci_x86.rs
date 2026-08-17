//! x86_64 PCI Configuration Mechanism #1 backend.
//! This backend is intentionally isolated from the kernel and can be replaced
//! by ECAM/firmware-mediated access on platforms that do not expose CF8/CFC.

use crate::pci::{PciConfigAccess, PciLocation};

#[cfg(target_arch = "x86_64")]
#[derive(Clone, Copy, Debug, Default)]
pub struct X86CfgIo;

#[cfg(target_arch = "x86_64")]
impl X86CfgIo {
    #[inline(always)]
    unsafe fn out32(port: u16, value: u32) {
        unsafe {
            core::arch::asm!(
                "out dx, eax",
                in("dx") port,
                in("eax") value,
                options(nostack, preserves_flags)
            );
        }
    }

    #[inline(always)]
    unsafe fn in32(port: u16) -> u32 {
        let value: u32;
        unsafe {
            core::arch::asm!(
                "in eax, dx",
                in("dx") port,
                out("eax") value,
                options(nostack, preserves_flags)
            );
        }
        value
    }
}

#[cfg(target_arch = "x86_64")]
impl PciConfigAccess for X86CfgIo {
    fn read_u32(&self, location: PciLocation, offset: u8) -> u32 {
        let aligned = offset & 0xFC;
        let address = 0x8000_0000u32
            | ((location.bus as u32) << 16)
            | ((location.device as u32) << 11)
            | ((location.function as u32) << 8)
            | aligned as u32;
        unsafe {
            Self::out32(0xCF8, address);
            Self::in32(0xCFC)
        }
    }
}
