#![no_std]

pub mod boot;

#[inline(always)]
pub unsafe fn read_sstatus() -> usize {
    let value: usize;
    core::arch::asm!("csrr {}, sstatus", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}

#[inline(always)]
pub unsafe fn read_stvec() -> usize {
    let value: usize;
    core::arch::asm!("csrr {}, stvec", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}
