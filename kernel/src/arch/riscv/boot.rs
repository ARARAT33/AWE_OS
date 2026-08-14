#![no_std]

#[inline(always)]
pub unsafe fn read_hart_id() -> usize {
    let hart: usize;
    core::arch::asm!("mv {}, tp", out(reg) hart, options(nomem, nostack, preserves_flags));
    hart
}

#[inline(always)]
pub unsafe fn wait() -> ! {
    loop { core::arch::asm!("wfi", options(nomem, nostack, preserves_flags)); }
}
