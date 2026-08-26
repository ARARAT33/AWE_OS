#![no_std]

use super::idt::Idt;

pub const TIMER_VECTOR: u8 = 32;
pub const KEYBOARD_VECTOR: u8 = 33;
pub const MOUSE_VECTOR: u8 = 44;

pub unsafe fn init_pic() {
    unsafe {
        use super::{io_in8, io_out8};
        let mask1 = io_in8(0x21);
        let mask2 = io_in8(0xA1);
        io_out8(0x20, 0x11);
        io_out8(0xA0, 0x11);
        io_out8(0x21, 0x20);
        io_out8(0xA1, 0x28);
        io_out8(0x21, 0x04);
        io_out8(0xA1, 0x02);
        io_out8(0x21, 0x01);
        io_out8(0xA1, 0x01);
        // Enable IRQ0 timer, IRQ1 keyboard, and IRQ2 cascade for slave IRQ12 mouse.
        io_out8(0x21, mask1 & !0x07);
        io_out8(0xA1, mask2 & !0x10);
    }
}

pub unsafe fn pic_send_eoi(irq: u8) {
    unsafe {
        use super::io_out8;
        if irq >= 8 { io_out8(0xA0, 0x20); }
        io_out8(0x20, 0x20);
    }
}

pub fn install_early_interrupts(idt: &mut Idt, code_selector: u16, timer_handler: u64) {
    idt.set_handler(TIMER_VECTOR, timer_handler, code_selector);
}

#[cfg(test)]
mod tests {
    use super::*;
    extern "C" fn timer() {}
    #[test]
    fn vectors_match_pic_remap() {
        assert_eq!(TIMER_VECTOR, 32);
        assert_eq!(KEYBOARD_VECTOR, 33);
        assert_eq!(MOUSE_VECTOR, 44);
    }
    #[test]
    fn timer_vector_is_installed() {
        let mut idt = Idt::new();
        install_early_interrupts(&mut idt, 0x08, timer as *const () as usize as u64);
        assert!(idt.is_present(TIMER_VECTOR));
        assert!(!idt.is_present(KEYBOARD_VECTOR));
    }
}