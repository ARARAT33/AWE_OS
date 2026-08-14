use super::Architecture;

pub struct X86_64;

impl Architecture for X86_64 {
    #[inline(always)]
    fn halt() -> ! {
        loop {
            unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)); }
        }
    }

    #[inline(always)]
    fn irq_save() -> usize {
        let flags: usize;
        unsafe { core::arch::asm!("pushfq; pop {}", out(reg) flags, options(nomem, preserves_flags)); }
        unsafe { core::arch::asm!("cli", options(nomem, nostack, preserves_flags)); }
        flags
    }

    #[inline(always)]
    unsafe fn irq_restore(state: usize) {
        if state & (1 << 9) != 0 {
            core::arch::asm!("sti", options(nomem, nostack, preserves_flags));
        }
    }
}

#[inline(always)]
pub fn cpu_halt() -> ! { X86_64::halt() }
