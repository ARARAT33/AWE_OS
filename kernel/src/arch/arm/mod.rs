#![no_std]

pub mod boot;

#[inline(always)]
pub unsafe fn read_cpsr() -> u32 {
    let value: u32;
    core::arch::asm!("mrs {}, cpsr", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}
