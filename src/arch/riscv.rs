use super::Architecture;

pub struct RiscV;

impl Architecture for RiscV {
    #[inline(always)]
    fn halt() -> ! {
        loop { unsafe { core::arch::asm!("wfi", options(nomem, nostack, preserves_flags)); } }
    }

    #[inline(always)]
    fn irq_save() -> usize {
        let value: usize;
        unsafe { core::arch::asm!("csrrc {}, sstatus, {}", out(reg) value, in(reg) (1usize << 1), options(nomem, preserves_flags)); }
        value
    }

    #[inline(always)]
    unsafe fn irq_restore(state: usize) {
        core::arch::asm!("csrw sstatus, {}", in(reg) state, options(nomem, nostack, preserves_flags));
    }
}
