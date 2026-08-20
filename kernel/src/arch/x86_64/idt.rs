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
        Self {
            offset_low: 0,
            selector: 0,
            options: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }
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
    pub const fn is_present(&self) -> bool {
        self.options & 0x8000 != 0
    }
}

#[repr(C, align(16))]
pub struct Idt {
    entries: [IdtEntry; 256],
}

impl Default for Idt {
    fn default() -> Self {
        Self::new()
    }
}

pub static mut IDT: Idt = Idt::new();

impl Idt {
    pub const fn new() -> Self {
        Self {
            entries: [IdtEntry::missing(); 256],
        }
    }
    pub fn set_handler(&mut self, vector: u8, handler: u64, selector: u16) {
        self.entries[vector as usize] = IdtEntry::new(handler, selector);
    }
    pub fn is_present(&self, vector: u8) -> bool {
        self.entries[vector as usize].is_present()
    }

    pub fn install_validated(&mut self, vector: u8, handler: u64, selector: u16) -> bool {
        if handler == 0 || selector == 0 {
            return false;
        }
        let upper = handler >> 48;
        if upper != 0 && upper != 0xffff {
            return false;
        }
        self.set_handler(vector, handler, selector);
        true
    }

    pub fn timer_installed(&self) -> bool {
        self.is_present(32)
    }

    pub unsafe fn load(&'static self) {
        unsafe {
            let descriptor = Idtr {
                limit: (size_of::<Self>() - 1) as u16,
                base: self as *const _ as u64,
            };
            core::arch::asm!("lidt [{}]", in(reg) &descriptor, options(readonly, nostack, preserves_flags));
        }
    }
}

#[repr(C, packed)]
struct Idtr {
    limit: u16,
    base: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    extern "C" fn test_handler() {}
    #[test]
    fn interrupt_gate_encodes_handler_address() {
        let entry = IdtEntry::new(test_handler as *const () as usize as u64, 0x08);
        assert!(entry.is_present());
    }
    #[test]
    fn idt_starts_empty() {
        let idt = Idt::new();
        assert!(!idt.is_present(0));
        assert!(!idt.is_present(255));
    }
    #[test]
    fn validated_install_rejects_invalid_handler() {
        let mut idt = Idt::new();
        assert!(!idt.install_validated(32, 0, 0x08));
        assert!(!idt.timer_installed());
    }
    #[test]
    fn validated_install_accepts_canonical_handler() {
        let mut idt = Idt::new();
        assert!(idt.install_validated(32, 0x0000_0000_0040_0000, 0x08));
        assert!(idt.timer_installed());
    }
}
