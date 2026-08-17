#![no_std]

pub mod boot;
pub mod entry;
pub mod idt;
pub mod interrupts;
pub mod isr;
pub mod isr_stubs;
pub mod timer;

pub const PAGE_TABLE_ENTRIES: usize = 512;
pub const PAGE_SIZE: u64 = 4096;

#[inline(always)]
pub unsafe fn read_cr3() -> u64 {
    unsafe {
        let value: u64;
        core::arch::asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
        value
    }
}

#[inline(always)]
pub unsafe fn write_cr3(value: u64) {
    unsafe {
        core::arch::asm!("mov cr3, {}", in(reg) value, options(nostack, preserves_flags));
    }
}

#[inline(always)]
pub unsafe fn read_rflags() -> u64 {
    unsafe {
        let value: u64;
        core::arch::asm!("pushfq; pop {}", out(reg) value, options(nomem, preserves_flags));
        value
    }
}

#[inline(always)]
pub unsafe fn io_out8(port: u16, value: u8) {
    unsafe {
        core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}

#[inline(always)]
pub unsafe fn io_in8(port: u16) -> u8 {
    unsafe {
        let value: u8;
        core::arch::asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
        value
    }
}
