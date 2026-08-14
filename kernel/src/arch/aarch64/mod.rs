#![no_std]

pub mod boot;

#[inline(always)]
pub unsafe fn read_current_el() -> u64 {
    let value: u64;
    core::arch::asm!("mrs {}, CurrentEL", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}

#[inline(always)]
pub unsafe fn read_mpidr() -> u64 {
    let value: u64;
    core::arch::asm!("mrs {}, MPIDR_EL1", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}
