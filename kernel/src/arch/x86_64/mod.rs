#![no_std]

pub mod boot;

pub const PAGE_TABLE_ENTRIES: usize = 512;
pub const PAGE_SIZE: u64 = 4096;

#[inline(always)]
pub unsafe fn read_cr3() -> u64 {
    let value: u64;
    core::arch::asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}

#[inline(always)]
pub unsafe fn write_cr3(value: u64) {
    core::arch::asm!("mov cr3, {}", in(reg) value, options(nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn read_rflags() -> u64 {
    let value: u64;
    core::arch::asm!("pushfq; pop {}", out(reg) value, options(nomem, preserves_flags));
    value
}
