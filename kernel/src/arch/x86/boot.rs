#![no_std]

#[inline(always)]
pub unsafe fn disable_interrupts() {
    core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn enable_interrupts() {
    core::arch::asm!("sti", options(nomem, nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn wait() -> ! {
    loop {
        core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
    }
}
