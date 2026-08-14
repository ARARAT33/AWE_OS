#![no_std]

#[repr(C, packed)]
pub struct GdtDescriptor {
    pub limit: u16,
    pub base: u64,
}

#[inline(always)]
pub unsafe fn load_gdt(descriptor: &GdtDescriptor) {
    core::arch::asm!("lgdt [{}]", in(reg) descriptor, options(readonly, nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn disable_interrupts() {
    core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn enable_interrupts() {
    core::arch::asm!("sti", options(nomem, nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn halt() -> ! {
    loop { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)); }
}
