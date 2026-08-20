#![no_std]

#[inline(always)]
pub unsafe fn disable_interrupts() {
    core::arch::asm!("cpsid i", options(nomem, nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn enable_interrupts() {
    core::arch::asm!("cpsie i", options(nomem, nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn wait() -> ! {
    loop {
        core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
    }
}
