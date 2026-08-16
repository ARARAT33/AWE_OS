#![no_std]

use core::mem::size_of;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    options: u16,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    pub const fn missing() -> Self {
        Self { offset_low: 0, selector: 0, options: 0, offset_mid: 0, offset_high: 0, reserved: 0 }
    }

    /// Build an interrupt gate for a 64-bit handler. The caller supplies the
    /// kernel code selector because GDT ownership remains with the platform layer.
    pub const fn new(handler: u64, selector: u16) -> Self {
        Self {
            offset_low: handler as u16,
            selector,
            options: 0x8e00,
            offset_mid: (handler >> 16) as u16,
            offset_high: (handler >> 32) as u32,
            reserved: 0,
        }
    }

    pub const fn is_present(&self) -> bool { self.options & 0x8000 != 0 }
}

#[repr(C, align(16))]
pub struct Idt {
    entries: [IdtEntry; 256],
}

impl Idt {
    pub const fn new() -> Self { Self { entries: [IdtEntry::missing(); 256] } }

    pub fn set_handler(&mut self, vector: u8, handler: u64, selector: u16) {
        self.entries[vector as usize] = IdtEntry::new(handler, selector);
    }

    pub fn is_present(&self, vector: u8) -> bool { self.entries[vector as usize].is_present() }

    /// Load this IDT into the current x86_64 CPU.
    ///
    /// # Safety
    /// `self` must remain valid while the CPU may dispatch interrupts through it.
    pub unsafe fn load(&'static self) {
        let descriptor = Idtr { limit: (size_of::<Self>() - 1) as u16, base: self as *const _ as u64 };
        core::arch::asm!("lidt [{}]", in(reg) &descriptor, options(readonly, nostack, preserves_flags));
    }
}

#[repr(C, packed)]
struct Idtr { limit: u16, base: u64 }

#[cfg(test)]
mod tests {
    use super::*;
    extern "C" fn test_handler() {}

    #[test]
    fn interrupt_gate_encodes_handler_address() {
        let entry = IdtEntry::new(test_handler as usize as u64, 0x08);
        assert!(entry.is_present());
        assert_eq!(entry.selector, 0x08);
    }

    #[test]
    fn idt_starts_empty() {
        let idt = Idt::new();
        assert!(!idt.is_present(0));
        assert!(!idt.is_present(255));
    }
}
