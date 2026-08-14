#![no_std]

#[cfg(target_arch = "x86_64")]
pub mod x86_64 {
    #[inline(always)]
    pub unsafe fn halt() -> ! {
        loop {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[cfg(target_arch = "x86")]
pub mod x86 {
    #[inline(always)]
    pub unsafe fn halt() -> ! {
        loop {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[cfg(target_arch = "aarch64")]
pub mod aarch64 {
    #[inline(always)]
    pub unsafe fn wait() -> ! {
        loop {
            core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
        }
    }
}

#[cfg(target_arch = "arm")]
pub mod arm {
    #[inline(always)]
    pub unsafe fn wait() -> ! {
        loop {
            core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
        }
    }
}

#[cfg(target_arch = "riscv64")]
pub mod riscv64 {
    #[inline(always)]
    pub unsafe fn wait() -> ! {
        loop {
            core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
        }
    }
}

#[cfg(target_arch = "riscv32")]
pub mod riscv32 {
    #[inline(always)]
    pub unsafe fn wait() -> ! {
        loop {
            core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
        }
    }
}
