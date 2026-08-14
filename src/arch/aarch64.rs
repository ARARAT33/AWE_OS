use super::Architecture;

pub struct AArch64;

impl Architecture for AArch64 {
    #[inline(always)]
    fn halt() -> ! {
        loop { unsafe { core::arch::asm!("wfi", options(nomem, nostack, preserves_flags)); } }
    }

    #[inline(always)]
    fn irq_save() -> usize {
        let daif: usize;
        unsafe { core::arch::asm!("mrs {}, daif", out(reg) daif, options(nomem, preserves_flags)); }
        unsafe { core::arch::asm!("msr daifset, #2", options(nomem, nostack, preserves_flags)); }
        daif
    }

    #[inline(always)]
    unsafe fn irq_restore(state: usize) {
        core::arch::asm!("msr daif, {}", in(reg) state, options(nomem, nostack, preserves_flags));
    }
}
