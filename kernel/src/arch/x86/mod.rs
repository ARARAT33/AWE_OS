#![no_std]

pub mod boot;

#[inline(always)]
pub unsafe fn halt() {
    core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
}
